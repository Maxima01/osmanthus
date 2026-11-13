#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::Result;
use osmanthus_application::{Backend, default_database, sample_bundle};
use osmanthus_domain::{EvidenceBundle, EvidenceSource, FileReport};
use osmanthus_engine::{Analysis, tree_rows};
use osmanthus_store::HistoryEntry;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{
    sync::mpsc::{self, SyncSender},
    thread,
};

slint::include_modules!();

enum Job {
    Sample,
    Import,
    Inspect,
    History,
    Open(String, String),
}
enum Content {
    Session(EvidenceBundle, Analysis),
    File(FileReport),
    None,
}
struct Update {
    content: Content,
    history: Vec<HistoryEntry>,
    status: String,
}

fn main() -> Result<()> {
    startup_trace("Creating desktop interface");
    let window = MainWindow::new()?;
    startup_trace("Desktop interface created");
    let weak = window.as_weak();
    let (sender, receiver) = mpsc::sync_channel(8);
    thread::Builder::new()
        .name("osmanthus-local-backend".into())
        .spawn(move || {
            let mut backend = match Backend::open(&default_database()) {
                Ok(backend) => backend,
                Err(error) => {
                    post(&weak, Err(error));
                    return;
                }
            };
            post(
                &weak,
                backend.history().map(|history| Update {
                    content: Content::None,
                    history,
                    status: "Ready · local storage opened · collectors offline".into(),
                }),
            );
            while let Ok(job) = receiver.recv() {
                post(&weak, execute(&mut backend, job));
            }
        })?;
    let tx = sender.clone();
    let weak = window.as_weak();
    window.on_load_sample(move || submit(&weak, &tx, Job::Sample));
    let tx = sender.clone();
    let weak = window.as_weak();
    window.on_import_recording(move || submit(&weak, &tx, Job::Import));
    let tx = sender.clone();
    let weak = window.as_weak();
    window.on_inspect_file(move || submit(&weak, &tx, Job::Inspect));
    let tx = sender.clone();
    let weak = window.as_weak();
    window.on_refresh_history(move || submit(&weak, &tx, Job::History));
    let weak = window.as_weak();
    window.on_open_saved(move |id, kind| submit(&weak, &sender, Job::Open(id.into(), kind.into())));
    startup_trace("Starting window event loop");
    window.run()?;
    Ok(())
}

fn startup_trace(message: &str) {
    if std::env::var_os("OSMANTHUS_STARTUP_TRACE").is_some() {
        eprintln!("Osmanthus startup: {message}");
    }
}

fn submit(weak: &slint::Weak<MainWindow>, sender: &SyncSender<Job>, job: Job) {
    if let Some(window) = weak.upgrade() {
        window.set_failed(false);
        window.set_busy(true);
        window.set_status("Working…".into());
        if sender.try_send(job).is_err() {
            window.set_busy(false);
            window.set_failed(true);
            window.set_status("Backend unavailable. Close and reopen Osmanthus to retry.".into());
        }
    }
}

fn execute(backend: &mut Backend, job: Job) -> Result<Update> {
    let (content, status) = match job {
        Job::Sample => {
            let bundle = sample_bundle()?;
            let analysis = backend.replay(&bundle)?;
            (
                Content::Session(bundle, analysis),
                "Sample replayed · no host telemetry collected".into(),
            )
        }
        Job::Import => match rfd::FileDialog::new()
            .set_title("Import a recorded evidence bundle")
            .add_filter("Evidence JSON", &["json"])
            .pick_file()
        {
            Some(path) => {
                let (bundle, analysis) = backend.import(&path)?;
                (
                    Content::Session(bundle, analysis),
                    "Recording imported · source is unverified".into(),
                )
            }
            None => (Content::None, "Import canceled".into()),
        },
        Job::Inspect => match rfd::FileDialog::new()
            .set_title("Inspect a file without executing it (up to 64 MiB)")
            .pick_file()
        {
            Some(path) => (
                Content::File(backend.inspect(&path)?),
                "Read-only inspection saved · file was not launched".into(),
            ),
            None => (Content::None, "Inspection canceled".into()),
        },
        Job::History => (Content::None, "Local history refreshed".into()),
        Job::Open(id, kind) => {
            if kind == "file" {
                (
                    Content::File(backend.report(&id)?),
                    "Saved file report opened".into(),
                )
            } else {
                let (bundle, analysis) = backend.session(&id)?;
                (
                    Content::Session(bundle, analysis),
                    "Saved recording opened".into(),
                )
            }
        }
    };
    Ok(Update {
        content,
        history: backend.history()?,
        status,
    })
}

fn post(weak: &slint::Weak<MainWindow>, result: Result<Update>) {
    let weak = weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(window) = weak.upgrade() {
            window.set_busy(false);
            match result {
                Ok(update) => {
                    window.set_failed(false);
                    window.set_status(update.status.into());
                    window.set_history(ModelRc::new(VecModel::from(
                        update
                            .history
                            .into_iter()
                            .map(|h| SavedRow {
                                id: h.id.into(),
                                label: short_label(&h.label, 96).into(),
                                kind: h.kind.into(),
                            })
                            .collect::<Vec<_>>(),
                    )));
                    match update.content {
                        Content::Session(bundle, analysis) => {
                            show_session(&window, &bundle, &analysis)
                        }
                        Content::File(report) => {
                            window.set_file_report(format_report(&report).into());
                            window.set_active_tab(2);
                        }
                        Content::None => {}
                    }
                }
                Err(error) => {
                    window.set_failed(true);
                    window.set_status(format!("{error:#} · choose another input or retry").into());
                }
            }
        }
    });
}

fn show_session(window: &MainWindow, bundle: &EvidenceBundle, analysis: &Analysis) {
    window.set_session_title(short_label(&bundle.session.label, 72).into());
    window.set_provenance(
        match bundle.session.source {
            EvidenceSource::Synthetic => {
                "SYNTHETIC SAMPLE · illustrative events · no host activity observed"
            }
            EvidenceSource::ImportedRecording => {
                "IMPORTED RECORDING · unverified source · collectors offline"
            }
        }
        .into(),
    );
    window.set_summary(
        format!(
            "{} Unresolved process identities: {}.",
            analysis.summary,
            analysis.gaps.len()
        )
        .into(),
    );
    let rows: Vec<_> = tree_rows(analysis)
        .into_iter()
        .map(|(depth, node)| {
            let name = node.image.rsplit(['\\', '/']).next().unwrap_or(&node.image);
            let label = format!(
                "{}{}{}  [PID {}]",
                "  ".repeat(depth.min(4)),
                if node.unresolved {
                    "Unresolved: "
                } else if depth == 0 {
                    "Root: "
                } else {
                    "Child: "
                },
                short_label(name, 24),
                node.pid
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "?".into())
            );
            let events = analysis
                .timeline
                .iter()
                .filter(|e| e.process == node.key)
                .map(|e| {
                    format!(
                        "+{} ms  {}\n{}\nEvidence: {}\n",
                        e.timestamp_ms,
                        e.kind.label(),
                        e.kind.detail(),
                        e.id
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let pid = node
                .pid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unavailable (start event missing)".into());
            let parent = node
                .parent
                .as_ref()
                .map(|key| {
                    let name = analysis
                        .processes
                        .iter()
                        .find(|p| &p.key == key)
                        .map(|p| p.image.as_str())
                        .unwrap_or("Unresolved parent");
                    format!("{name}\nParent identity: {}:{}", key.boot_id, key.start_key)
                })
                .unwrap_or_else(|| {
                    if node.unresolved {
                        "Unavailable (start event missing)".into()
                    } else {
                        "No parent in recording (root)".into()
                    }
                });
            EvidenceRow {
                label: label.into(),
                detail: format!(
                    "{}\n\nPID: {}\nBoot ID: {}\nStart key: {}\nParent: {}\n\n{}",
                    node.image, pid, node.key.boot_id, node.key.start_key, parent, events
                )
                .into(),
            }
        })
        .collect();
    window.set_selected_process(if rows.is_empty() { -1 } else { 0 });
    window.set_process_detail(
        rows.first()
            .map(|r| r.detail.clone())
            .unwrap_or_else(|| "No process evidence in this recording.".into()),
    );
    window.set_processes(ModelRc::new(VecModel::from(rows)));
    window.set_timeline(ModelRc::new(VecModel::from(analysis.timeline.iter().map(|e| EvidenceRow {
        label: format!("+{:>5} ms   {}", e.timestamp_ms, e.kind.label()).into(),
        detail: format!("{}\n\n{}\n\nTime: +{} ms\nEvent ID: {}\nBoot ID: {}\nStart key: {}\n\nRecording: {}\nSource: {:?} (unverified)", e.kind.label(), e.kind.detail(), e.timestamp_ms, e.id, e.process.boot_id, e.process.start_key, bundle.session.label, bundle.session.source).into(),
    }).collect::<Vec<_>>())));
    window.set_findings(ModelRc::new(VecModel::from(
        analysis
            .findings
            .iter()
            .map(|f| {
                let evidence = analysis
                    .timeline
                    .iter()
                    .filter(|e| f.evidence_ids.contains(&e.id))
                    .map(|e| format!("{} | {}\n{}", e.id, e.kind.label(), e.kind.detail()))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                EvidenceRow {
                    label: f.title.clone().into(),
                    detail: format!(
                        "{}\n\n{}\n\nRule: {}\n\nSource evidence\n{}",
                        f.title, f.explanation, f.rule_id, evidence
                    )
                    .into(),
                }
            })
            .collect::<Vec<_>>(),
    )));
    window.set_event_detail(
        "Select an event or review signal to inspect its source evidence.".into(),
    );
    window.set_active_tab(0);
}

fn format_report(report: &FileReport) -> String {
    format!(
        "{}\n\nFormat: {}\nSize: {} bytes\nSHA-256:\n{}\n\nPE imports ({})\n{}\n\nEmbedded URL clues ({})\n{}\n\nUnresolved / limitations\n{}",
        report.path,
        report.format,
        report.size,
        report.sha256,
        report.imports.len(),
        if report.imports.is_empty() {
            "No imports discovered.".into()
        } else {
            report.imports.join("\n")
        },
        report.embedded_urls.len(),
        if report.embedded_urls.is_empty() {
            "No ASCII URL clues discovered.".into()
        } else {
            report.embedded_urls.join("\n")
        },
        report.limitations.join("\n\n")
    )
}

fn short_label(value: &str, limit: usize) -> String {
    let mut chars = value.chars().map(|c| if c.is_control() { ' ' } else { c });
    let mut text: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        text.push('…');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::{
        Model, Rgb8Pixel,
        platform::{
            Platform, WindowAdapter,
            software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        },
    };
    use std::{io::Write, path::Path, rc::Rc};

    struct PreviewPlatform {
        window: Rc<MinimalSoftwareWindow>,
    }
    impl Platform for PreviewPlatform {
        fn create_window_adapter(
            &self,
        ) -> std::result::Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
            Ok(self.window.clone())
        }
    }

    fn capture(adapter: &MinimalSoftwareWindow, path: &Path, width: u32, height: u32) {
        adapter.set_size(slint::PhysicalSize::new(width, height));
        adapter.request_redraw();
        slint::platform::update_timers_and_animations();
        let mut pixels = vec![Rgb8Pixel::default(); (width * height) as usize];
        assert!(adapter.draw_if_needed(|renderer| {
            renderer.render(&mut pixels, width as usize);
        }));
        assert!(
            pixels
                .iter()
                .any(|p| p.r != pixels[0].r || p.g != pixels[0].g || p.b != pixels[0].b)
        );
        let mut output = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
        write!(output, "P6\n{width} {height}\n255\n").unwrap();
        for pixel in pixels {
            output.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
        }
    }

    #[test]
    fn desktop_renders_empty_sample_details_and_history_at_supported_sizes() {
        let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        slint::platform::set_platform(Box::new(PreviewPlatform {
            window: adapter.clone(),
        }))
        .unwrap();
        let window = MainWindow::new().unwrap();
        window.set_status("Ready · local storage opened · collectors offline".into());
        window.show().unwrap();
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp/qa");
        std::fs::create_dir_all(&directory).unwrap();
        capture(&adapter, &directory.join("empty.ppm"), 1180, 820);
        let bundle = sample_bundle().unwrap();
        let analysis = osmanthus_engine::analyze(&bundle).unwrap();
        show_session(&window, &bundle, &analysis);
        assert_eq!(window.get_processes().row_count(), 3);
        assert_eq!(window.get_timeline().row_count(), 9);
        assert_eq!(window.get_findings().row_count(), 1);
        capture(&adapter, &directory.join("behavior.ppm"), 1180, 820);
        capture(&adapter, &directory.join("behavior-small.ppm"), 900, 680);
        window.set_active_tab(1);
        window.set_event_detail(window.get_findings().row_data(0).unwrap().detail);
        capture(&adapter, &directory.join("timeline.ppm"), 1180, 820);
        window.set_active_tab(2);
        let report = osmanthus_static_report();
        window.set_file_report(format_report(&report).into());
        capture(&adapter, &directory.join("inspection.ppm"), 1180, 820);
        window.set_active_tab(3);
        window.set_history(ModelRc::new(VecModel::from(vec![SavedRow {
            id: bundle.session.id.to_string().into(),
            label: bundle.session.label.into(),
            kind: "recording".into(),
        }])));
        capture(&adapter, &directory.join("history.ppm"), 1180, 820);
        window.set_active_tab(4);
        capture(&adapter, &directory.join("later.ppm"), 1180, 820);
        window.hide().unwrap();
    }

    fn osmanthus_static_report() -> FileReport {
        FileReport {
            path: "Illustrative file inspection (render fixture)".into(),
            size: 3,
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
            format: "Non-PE file".into(),
            imports: vec![],
            embedded_urls: vec![],
            limitations: vec![
                "Static evidence does not establish safety. No file was executed.".into(),
            ],
        }
    }
}
