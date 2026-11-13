//! Pure, deterministic reconstruction. Input order and PID reuse do not define lineage.
use anyhow::Result;
use osmanthus_domain::{BehaviorEvent, EventKind, EvidenceBundle, Finding, ProcessKey};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize)]
pub struct ProcessNode {
    pub key: ProcessKey,
    pub pid: Option<u32>,
    pub image: String,
    pub parent: Option<ProcessKey>,
    pub unresolved: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Analysis {
    pub processes: Vec<ProcessNode>,
    pub timeline: Vec<BehaviorEvent>,
    pub findings: Vec<Finding>,
    pub endpoints: Vec<String>,
    pub files: Vec<String>,
    pub gaps: Vec<String>,
    pub summary: String,
}

pub fn analyze(bundle: &EvidenceBundle) -> Result<Analysis> {
    bundle.validate()?;
    let mut timeline = bundle.events.clone();
    timeline.sort_by_key(|event| (event.timestamp_ms, event.id));
    let mut nodes = BTreeMap::new();
    let mut endpoints = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut findings = Vec::new();
    for event in &timeline {
        if let EventKind::ProcessStarted {
            pid, image, parent, ..
        } = &event.kind
        {
            nodes.insert(
                event.process.clone(),
                ProcessNode {
                    key: event.process.clone(),
                    pid: Some(*pid),
                    image: image.clone(),
                    parent: parent.clone(),
                    unresolved: false,
                },
            );
        }
        match &event.kind {
            EventKind::DnsQuery { domain } => { endpoints.insert(format!("DNS {domain}")); }
            EventKind::NetworkConnection { remote, port, protocol } => {
                endpoints.insert(format!("{protocol} {remote}:{port}"));
            }
            EventKind::FileCreated { path } => { files.insert(path.clone()); }
            EventKind::PersistenceChanged { location, value } => findings.push(Finding {
                rule_id: "review.persistence.v1".into(), title: "Review a persistence change".into(),
                explanation: format!("The recording reports {location} = {value}. Confirm this startup behavior is expected. This is a review signal, not a malware verdict."),
                evidence_ids: vec![event.id],
            }),
            _ => {}
        }
    }
    let mut missing: BTreeSet<ProcessKey> = timeline.iter().map(|e| e.process.clone()).collect();
    missing.extend(nodes.values().filter_map(|n| n.parent.clone()));
    for key in missing {
        nodes.entry(key.clone()).or_insert(ProcessNode {
            key,
            pid: None,
            image: "Unresolved process (start event missing)".into(),
            parent: None,
            unresolved: true,
        });
    }
    let gaps = nodes
        .values()
        .filter(|n| n.unresolved)
        .map(|n| {
            format!(
                "Missing process-start evidence for {}:{}",
                n.key.boot_id, n.key.start_key
            )
        })
        .collect();
    let summary = format!(
        "The recording contains {} process identities. Created files: {}. DNS/connection endpoints: {}. Persistence changes to review: {}. No assurance of safety is inferred.",
        nodes.len(),
        files.len(),
        endpoints.len(),
        findings.len()
    );
    Ok(Analysis {
        processes: nodes.into_values().collect(),
        timeline,
        findings,
        endpoints: endpoints.into_iter().collect(),
        files: files.into_iter().collect(),
        gaps,
        summary,
    })
}

/// Parent-first display order, bounded by validated acyclic lineage.
pub fn tree_rows(analysis: &Analysis) -> Vec<(usize, &ProcessNode)> {
    let mut children: BTreeMap<Option<&ProcessKey>, Vec<&ProcessNode>> = BTreeMap::new();
    for node in &analysis.processes {
        children.entry(node.parent.as_ref()).or_default().push(node);
    }
    let mut stack: Vec<_> = children
        .get(&None)
        .into_iter()
        .flatten()
        .rev()
        .map(|n| (0, *n))
        .collect();
    let mut rows = Vec::new();
    while let Some((depth, node)) = stack.pop() {
        rows.push((depth, node));
        if let Some(nodes) = children.get(&Some(&node.key)) {
            stack.extend(nodes.iter().rev().map(|n| (depth + 1, *n)));
        }
    }
    rows
}
