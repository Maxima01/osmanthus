//! Recording input and nonblocking collector queue. No live Windows collector yet.
use anyhow::{Context, Result, ensure};
use osmanthus_domain::{BehaviorEvent, EvidenceBundle, MAX_BUNDLE_BYTES};
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
};

pub fn read_bundle(path: &Path) -> Result<EvidenceBundle> {
    let file = File::open(path).context("Cannot open the event recording")?;
    ensure!(
        file.metadata()?.is_file(),
        "Select a regular recording file"
    );
    let mut bytes = Vec::new();
    file.take(MAX_BUNDLE_BYTES + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_BUNDLE_BYTES,
        "Recording exceeds the 8 MiB limit"
    );
    let bundle: EvidenceBundle =
        serde_json::from_slice(&bytes).context("Invalid recording JSON")?;
    bundle.validate()?;
    Ok(bundle)
}

#[derive(Clone)]
pub struct EventSink {
    sender: SyncSender<BehaviorEvent>,
    dropped: Arc<AtomicU64>,
}

impl EventSink {
    /// Collector callbacks must copy and try_send; no scanning or database work.
    pub fn try_record(&self, event: BehaviorEvent) -> bool {
        match self.sender.try_send(event) {
            Ok(()) => true,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

pub fn bounded_queue(capacity: usize) -> (EventSink, Receiver<BehaviorEvent>) {
    let (sender, receiver) = mpsc::sync_channel(capacity);
    (
        EventSink {
            sender,
            dropped: Arc::new(AtomicU64::new(0)),
        },
        receiver,
    )
}
