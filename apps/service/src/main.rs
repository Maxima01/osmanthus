//! Intentionally non-operational service shell. No installation or elevation.
fn main() {
    println!(
        "Osmanthus service shell | protocol {}",
        osmanthus_ipc::PROTOCOL_VERSION
    );
    println!(
        "Not installed or collecting. SCM lifecycle, ETW, watchdog, and authenticated named-pipe transport are deferred."
    );
}
