//! Versioned evidence contracts. No platform, database, or GUI dependencies.
use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_EVENTS: usize = 10_000;
pub const MAX_BUNDLE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessKey {
    pub boot_id: Uuid,
    pub start_key: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Synthetic,
    ImportedRecording,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub id: Uuid,
    pub label: String,
    pub source: EvidenceSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBundle {
    pub schema_version: u32,
    pub session: Session,
    pub events: Vec<BehaviorEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorEvent {
    pub id: Uuid,
    pub timestamp_ms: u64,
    pub process: ProcessKey,
    pub kind: EventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventKind {
    ProcessStarted {
        pid: u32,
        parent: Option<ProcessKey>,
        image: String,
        command_line: String,
    },
    ProcessExited,
    FileCreated {
        path: String,
    },
    ModuleLoaded {
        path: String,
    },
    DnsQuery {
        domain: String,
    },
    NetworkConnection {
        remote: String,
        port: u16,
        protocol: String,
    },
    PersistenceChanged {
        location: String,
        value: String,
    },
}

impl EventKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ProcessStarted { .. } => "Process started",
            Self::ProcessExited => "Process exited",
            Self::FileCreated { .. } => "File created",
            Self::ModuleLoaded { .. } => "Module loaded",
            Self::DnsQuery { .. } => "DNS query",
            Self::NetworkConnection { .. } => "Network connection",
            Self::PersistenceChanged { .. } => "Persistence change",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Self::ProcessStarted {
                image,
                command_line,
                ..
            } => format!("{image} | {command_line}"),
            Self::ProcessExited => "Recorded process exit".into(),
            Self::FileCreated { path } | Self::ModuleLoaded { path } => path.clone(),
            Self::DnsQuery { domain } => domain.clone(),
            Self::NetworkConnection {
                remote,
                port,
                protocol,
            } => format!("{protocol} {remote}:{port}"),
            Self::PersistenceChanged { location, value } => format!("{location} = {value}"),
        }
    }
}

impl EvidenceBundle {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == SCHEMA_VERSION,
            "Unsupported evidence schema {}",
            self.schema_version
        );
        ensure!(!self.session.id.is_nil(), "Session ID must not be nil");
        check_text(&self.session.label)?;
        ensure!(
            !self.session.label.trim().is_empty(),
            "Session label is empty"
        );
        ensure!(
            self.events.len() <= MAX_EVENTS,
            "Recording exceeds {MAX_EVENTS} events"
        );
        let mut ids = BTreeSet::new();
        let mut starts = BTreeSet::new();
        let mut parents = BTreeMap::new();
        for event in &self.events {
            ensure!(
                !event.id.is_nil() && ids.insert(event.id),
                "Duplicate or nil event ID"
            );
            check_key(&event.process)?;
            ensure!(
                event.timestamp_ms <= i64::MAX as u64,
                "Timestamp is out of range"
            );
            match &event.kind {
                EventKind::ProcessStarted {
                    parent,
                    image,
                    command_line,
                    ..
                } => {
                    ensure!(
                        starts.insert(event.process.clone()),
                        "Duplicate process start key"
                    );
                    check_text(image)?;
                    check_text(command_line)?;
                    if let Some(parent) = parent {
                        check_key(parent)?;
                        ensure!(parent != &event.process, "Process cannot parent itself");
                        ensure!(
                            parent.boot_id == event.process.boot_id,
                            "Parent belongs to another boot"
                        );
                        parents.insert(event.process.clone(), parent.clone());
                    }
                }
                EventKind::FileCreated { path } | EventKind::ModuleLoaded { path } => {
                    check_text(path)?
                }
                EventKind::DnsQuery { domain } => check_text(domain)?,
                EventKind::NetworkConnection {
                    remote,
                    protocol,
                    port,
                } => {
                    check_text(remote)?;
                    check_text(protocol)?;
                    ensure!(*port != 0, "Remote port must be nonzero");
                }
                EventKind::PersistenceChanged { location, value } => {
                    check_text(location)?;
                    check_text(value)?;
                }
                EventKind::ProcessExited => {}
            }
        }
        // Detect cycles without recursion. Missing parents are retained as unresolved evidence.
        let mut completed = BTreeSet::new();
        for key in parents.keys() {
            let mut path = BTreeSet::new();
            let mut current = key;
            while !completed.contains(current) {
                if !path.insert(current.clone()) {
                    bail!("Process lineage contains a cycle");
                }
                match parents.get(current) {
                    Some(parent) => current = parent,
                    None => break,
                }
            }
            completed.extend(path);
        }
        Ok(())
    }
}

fn check_key(key: &ProcessKey) -> Result<()> {
    ensure!(
        !key.boot_id.is_nil() && key.start_key != 0,
        "Invalid stable process identity"
    );
    Ok(())
}

fn check_text(value: &str) -> Result<()> {
    ensure!(
        value.len() <= 8192 && !value.contains('\0'),
        "Evidence text exceeds limits or contains NUL"
    );
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub title: String,
    pub explanation: String,
    pub evidence_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReport {
    pub path: String,
    pub size: u64,
    pub sha256: String,
    pub format: String,
    pub imports: Vec<String>,
    pub embedded_urls: Vec<String>,
    pub limitations: Vec<String>,
}
