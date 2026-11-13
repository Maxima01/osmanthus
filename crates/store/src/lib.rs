//! One connection owned by the application worker. Local development evidence only.
use anyhow::{Context, Result, ensure};
use osmanthus_domain::{EvidenceBundle, FileReport};
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub struct Store {
    connection: Connection,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub label: String,
    pub kind: String,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let connection =
            Connection::open(path).context("Cannot open the local evidence database")?;
        connection.busy_timeout(Duration::from_secs(3))?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")?;
        let version: u32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(
            version <= 1,
            "Database was created by a newer Osmanthus version"
        );
        if version == 0 {
            connection.execute_batch(include_str!("../migrations/001.sql"))?;
        }
        Ok(Self { connection })
    }

    pub fn save_bundle(&mut self, bundle: &EvidenceBundle) -> Result<()> {
        bundle.validate()?;
        let json = serde_json::to_string(bundle)?;
        let tx = self.connection.transaction()?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT bundle_json FROM sessions WHERE id=?1",
                [bundle.session.id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            ensure!(
                existing == json,
                "Session ID already exists with different evidence; import was refused"
            );
            return Ok(());
        }
        tx.execute(
            "INSERT INTO sessions(id,label,source,bundle_json,saved_ms) VALUES(?1,?2,?3,?4,?5)",
            params![
                bundle.session.id.to_string(),
                bundle.session.label,
                serde_json::to_string(&bundle.session.source)?,
                json,
                now_ms()
            ],
        )?;
        for event in &bundle.events {
            tx.execute("INSERT INTO events(session_id,event_id,timestamp_ms,event_json) VALUES(?1,?2,?3,?4)",
                params![bundle.session.id.to_string(), event.id.to_string(), event.timestamp_ms as i64,
                    serde_json::to_string(event)?])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_bundle(&self, id: &str) -> Result<EvidenceBundle> {
        let json: String = self.connection.query_row(
            "SELECT bundle_json FROM sessions WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        let bundle: EvidenceBundle = serde_json::from_str(&json)?;
        bundle.validate()?;
        Ok(bundle)
    }

    pub fn save_report(&self, report: &FileReport) -> Result<()> {
        self.connection.execute(
            "INSERT INTO file_reports(id,path,report_json,saved_ms) VALUES(?1,?2,?3,?4)",
            params![
                Uuid::new_v4().to_string(),
                report.path,
                serde_json::to_string(report)?,
                now_ms()
            ],
        )?;
        Ok(())
    }

    pub fn load_report(&self, id: &str) -> Result<FileReport> {
        let json: String = self.connection.query_row(
            "SELECT report_json FROM file_reports WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn history(&self) -> Result<Vec<HistoryEntry>> {
        let mut statement = self.connection.prepare(
            "SELECT id,label,kind FROM (SELECT id,label,'recording' AS kind,saved_ms FROM sessions
             UNION ALL SELECT id,path AS label,'file' AS kind,saved_ms FROM file_reports)
             ORDER BY saved_ms DESC,id LIMIT 100",
        )?;
        let rows = statement.query_map([], |r| {
            Ok(HistoryEntry {
                id: r.get(0)?,
                label: r.get(1)?,
                kind: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}
