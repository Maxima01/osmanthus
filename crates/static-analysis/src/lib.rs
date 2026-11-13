//! Bounded read-only analysis of a single snapshot. Never launches the input.
use anyhow::{Context, Result, ensure};
use goblin::pe::PE;
use osmanthus_domain::FileReport;
use regex::bytes::Regex;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs::File, io::Read, path::Path};

pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

pub fn inspect(path: &Path) -> Result<FileReport> {
    let canonical = path
        .canonicalize()
        .context("Cannot resolve the file path")?;
    let file = File::open(&canonical).context("Cannot open the selected file")?;
    let metadata = file.metadata()?;
    ensure!(metadata.is_file(), "Select a regular file");
    ensure!(
        metadata.len() <= MAX_FILE_BYTES,
        "File exceeds the preliminary 64 MiB inspection limit"
    );
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_FILE_BYTES,
        "File grew beyond the inspection limit"
    );
    let mut report = FileReport {
        path: canonical.to_string_lossy().into_owned(), size: bytes.len() as u64,
        sha256: format!("{:x}", Sha256::digest(&bytes)), format: "Non-PE file".into(),
        imports: vec![], embedded_urls: vec![],
        limitations: vec![
            "Static inspection predicts capabilities; it does not prove behavior or safety.".into(),
            "Authenticode, reputation, YARA, embedded payload extraction, and dynamic imports are not evaluated.".into(),
            "No file was executed. Network callbacks and persistence require runtime evidence.".into(),
        ],
    };
    if bytes.starts_with(b"MZ") {
        match PE::parse(&bytes) {
            Ok(pe) => {
                report.format = format!(
                    "PE{} (machine 0x{:04x})",
                    if pe.is_64 { "32+" } else { "32" },
                    pe.header.coff_header.machine
                );
                let imports: BTreeSet<_> = pe
                    .imports
                    .iter()
                    .map(|i| format!("{}!{}", i.dll, i.name))
                    .collect();
                if imports.len() > 512 {
                    report
                        .limitations
                        .push("Import display truncated to 512 entries.".into());
                }
                report.imports = imports.into_iter().take(512).collect();
            }
            Err(error) => {
                report.format = "Malformed or unsupported PE".into();
                report
                    .limitations
                    .push(format!("PE parser could not inspect this file: {error}"));
            }
        }
    }
    let pattern = Regex::new(r#"https?://[A-Za-z0-9][A-Za-z0-9._~:/?#\[\]@!$&'()*+,;=%-]{0,510}"#)?;
    let urls: BTreeSet<_> = pattern
        .find_iter(&bytes)
        .take(1024)
        .map(|m| String::from_utf8_lossy(m.as_bytes()).into_owned())
        .collect();
    report.embedded_urls = urls.into_iter().take(64).collect();
    report.limitations.push("URL extraction is limited to 64 ASCII strings and may include dead code or false positives; URLs are never contacted.".into());
    Ok(report)
}
