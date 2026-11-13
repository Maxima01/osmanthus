use osmanthus_application::{Backend, sample_bundle};
use osmanthus_domain::{EventKind, MAX_BUNDLE_BYTES};
use osmanthus_engine::{analyze, tree_rows};
use osmanthus_ipc::{Frame, MAX_FRAME_BYTES, PROTOCOL_VERSION, Query, read_frame, write_frame};
use std::{fs, io::Write};
use tempfile::tempdir;

#[test]
fn replay_is_deterministic_and_pid_reuse_does_not_change_lineage() {
    let mut bundle = sample_bundle().unwrap();
    let expected = analyze(&bundle).unwrap();
    bundle.events.reverse();
    let actual = analyze(&bundle).unwrap();
    assert_eq!(
        serde_json::to_value(&actual).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert_eq!(actual.processes.len(), 3);
    let reused = actual
        .processes
        .iter()
        .find(|n| n.key.start_key == 3)
        .unwrap();
    assert!(reused.parent.is_none());
    let tree = tree_rows(&actual);
    assert_eq!(
        tree.iter()
            .map(|(depth, n)| (*depth, n.key.start_key))
            .collect::<Vec<_>>(),
        [(0, 1), (1, 2), (0, 3)]
    );
    assert_eq!(
        actual.findings[0].evidence_ids,
        [bundle
            .events
            .iter()
            .find(|e| matches!(e.kind, EventKind::PersistenceChanged { .. }))
            .unwrap()
            .id]
    );
}

#[test]
fn missing_start_is_visible_and_does_not_drop_child_evidence() {
    let mut bundle = sample_bundle().unwrap();
    bundle.events.remove(0);
    let analysis = analyze(&bundle).unwrap();
    assert_eq!(analysis.gaps.len(), 1);
    assert_eq!(tree_rows(&analysis).len(), 3);
    assert_eq!(analysis.timeline.len(), 8);
    assert_eq!(analysis.files.len(), 1);
}

#[test]
fn cycles_duplicates_and_unknown_schema_are_rejected() {
    let mut cyclic = sample_bundle().unwrap();
    let child = cyclic.events[2].process.clone();
    if let EventKind::ProcessStarted { parent, .. } = &mut cyclic.events[0].kind {
        *parent = Some(child);
    }
    assert!(analyze(&cyclic).unwrap_err().to_string().contains("cycle"));
    let mut duplicate = sample_bundle().unwrap();
    duplicate.events.push(duplicate.events[0].clone());
    assert!(duplicate.validate().is_err());
    let mut version = sample_bundle().unwrap();
    version.schema_version = 99;
    assert!(version.validate().is_err());
}

#[test]
fn storage_roundtrip_is_idempotent_and_conflicting_ids_are_refused() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("evidence.db");
    let bundle = sample_bundle().unwrap();
    {
        let mut backend = Backend::open(&path).unwrap();
        backend.replay(&bundle).unwrap();
        backend.replay(&bundle).unwrap();
        assert_eq!(backend.history().unwrap().len(), 1);
        let mut conflicting = bundle.clone();
        conflicting.events[0].timestamp_ms = 99;
        assert!(backend.replay(&conflicting).is_err());
    }
    let backend = Backend::open(&path).unwrap();
    let (loaded, analysis) = backend.session(&bundle.session.id.to_string()).unwrap();
    assert_eq!(
        serde_json::to_string(&loaded).unwrap(),
        serde_json::to_string(&bundle).unwrap()
    );
    assert_eq!(analysis.timeline.len(), 9);
}

#[test]
fn static_inspection_hashes_known_bytes_and_persists_report() {
    let directory = tempdir().unwrap();
    let file = directory.path().join("known.txt");
    fs::write(&file, b"abc").unwrap();
    let backend = Backend::open(&directory.path().join("evidence.db")).unwrap();
    let report = backend.inspect(&file).unwrap();
    assert_eq!(
        report.sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(report.size, 3);
    assert_eq!(fs::read(&file).unwrap(), b"abc");
    let history = backend.history().unwrap();
    let loaded = backend.report(&history[0].id).unwrap();
    assert_eq!(loaded.sha256, report.sha256);
}

#[test]
fn malformed_pe_keeps_hash_and_urls_are_clues_only() {
    let directory = tempdir().unwrap();
    let file = directory.path().join("malformed.exe");
    fs::write(&file, b"MZ\0https://updates.example.test/check\0").unwrap();
    let report = osmanthus_static::inspect(&file).unwrap();
    assert_eq!(report.format, "Malformed or unsupported PE");
    assert_eq!(report.embedded_urls, ["https://updates.example.test/check"]);
    assert_eq!(report.sha256.len(), 64);
}

#[cfg(windows)]
#[test]
fn valid_windows_executable_exposes_pe_imports_without_execution() {
    let report = osmanthus_static::inspect(&std::env::current_exe().unwrap()).unwrap();
    assert!(report.format.starts_with("PE"));
    assert!(!report.imports.is_empty());
    assert!(
        report
            .imports
            .iter()
            .any(|name| name.to_ascii_lowercase().contains("kernel32"))
    );
    assert_eq!(report.sha256.len(), 64);
}

#[test]
fn file_and_recording_limits_reject_before_unbounded_reads() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("large.bin");
    fs::File::create(&path)
        .unwrap()
        .set_len(osmanthus_static::MAX_FILE_BYTES + 1)
        .unwrap();
    assert!(osmanthus_static::inspect(&path).is_err());
    let recording = directory.path().join("large.json");
    fs::File::create(&recording)
        .unwrap()
        .set_len(MAX_BUNDLE_BYTES + 1)
        .unwrap();
    assert!(osmanthus_telemetry::read_bundle(&recording).is_err());
}

#[test]
fn invalid_import_leaves_history_empty() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("broken.json");
    fs::write(&path, b"{\"schema_version\":1").unwrap();
    let mut backend = Backend::open(&directory.path().join("evidence.db")).unwrap();
    assert!(backend.import(&path).is_err());
    assert!(backend.history().unwrap().is_empty());
}

#[test]
fn collector_queue_is_nonblocking_and_counts_lost_events() {
    let event = sample_bundle().unwrap().events.remove(0);
    let (sink, receiver) = osmanthus_telemetry::bounded_queue(1);
    assert!(sink.try_record(event.clone()));
    assert!(!sink.try_record(event.clone()));
    assert_eq!(sink.dropped(), 1);
    assert_eq!(receiver.recv().unwrap().id, event.id);
    drop(receiver);
    assert!(!sink.try_record(event));
    assert_eq!(sink.dropped(), 2);
}

#[test]
fn protocol_rejects_oversize_truncation_and_wrong_versions() {
    let frame = Frame {
        version: PROTOCOL_VERSION,
        request_id: 7,
        request: Query::Health,
    };
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &frame).unwrap();
    assert_eq!(read_frame(&bytes[..]).unwrap(), frame);
    assert!(read_frame(&bytes[..bytes.len() - 1]).is_err());
    assert!(read_frame(&((MAX_FRAME_BYTES + 1) as u32).to_le_bytes()[..]).is_err());
    let mut wrong = Vec::new();
    let body = br#"{"version":99,"request_id":7,"request":{"type":"health"}}"#;
    wrong.write_all(&(body.len() as u32).to_le_bytes()).unwrap();
    wrong.write_all(body).unwrap();
    assert!(read_frame(&wrong[..]).is_err());
}
