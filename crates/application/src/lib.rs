//! Local development backend, shared by desktop and CLI.
//! The production desktop will replace this adapter with authenticated IPC.
use anyhow::Result;
use osmanthus_domain::{EvidenceBundle, FileReport};
use osmanthus_engine::{Analysis, analyze};
use osmanthus_store::{HistoryEntry, Store};
use std::path::{Path, PathBuf};

pub struct Backend {
    store: Store,
}

pub fn default_database() -> PathBuf {
    std::env::var_os("OSMANTHUS_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
        .join("evidence.db")
}

pub fn sample_bundle() -> Result<EvidenceBundle> {
    let bundle: EvidenceBundle =
        serde_json::from_str(include_str!("../../../tests/fixtures/sample-install.json"))?;
    bundle.validate()?;
    Ok(bundle)
}

impl Backend {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            store: Store::open(path)?,
        })
    }
    pub fn replay(&mut self, bundle: &EvidenceBundle) -> Result<Analysis> {
        let analysis = analyze(bundle)?;
        self.store.save_bundle(bundle)?;
        Ok(analysis)
    }
    pub fn import(&mut self, path: &Path) -> Result<(EvidenceBundle, Analysis)> {
        let bundle = osmanthus_telemetry::read_bundle(path)?;
        let analysis = self.replay(&bundle)?;
        Ok((bundle, analysis))
    }
    pub fn inspect(&self, path: &Path) -> Result<FileReport> {
        let report = osmanthus_static::inspect(path)?;
        self.store.save_report(&report)?;
        Ok(report)
    }
    pub fn history(&self) -> Result<Vec<HistoryEntry>> {
        self.store.history()
    }
    pub fn session(&self, id: &str) -> Result<(EvidenceBundle, Analysis)> {
        let bundle = self.store.load_bundle(id)?;
        let analysis = analyze(&bundle)?;
        Ok((bundle, analysis))
    }
    pub fn report(&self, id: &str) -> Result<FileReport> {
        self.store.load_report(id)
    }
}
