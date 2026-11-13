# Project Osmanthus

A preliminary Rust desktop application for evidence-based application auditing.
This increment turns the earlier architecture discussion into a working local
foundation. **It is a replay and static-inspection tool, not a live security monitor.**

## Run

On this workspace, a local Rust/C toolchain was installed under `tmp/toolchain`
without changing the system PATH. Double-click **start-osmanthus.cmd** in the
project folder, or run it from Command Prompt or PowerShell:

```bat
.\start-osmanthus.cmd
```

This launcher opens the already-built desktop executable from the project root.
If the executable is missing, it builds it with the local toolchain. It does not
require Cargo on your system PATH or PowerShell script execution. After changing
Rust or Slint code, use the Cargo helper below to rebuild and run the new source.

For other Cargo commands from **Command Prompt or PowerShell**, use:

```bat
.\scripts\cargo-local.cmd run -p osmanthus-desktop
.\scripts\cargo-local.cmd run -p osmanthus-cli -- demo
```

The original helper is for a **PowerShell terminal only**:

```powershell
.\scripts\cargo-local.ps1 run -p osmanthus-desktop
```

If `cargo` is not recognized, use one of the local-toolchain launchers above.
Running a `.ps1` path directly from Command Prompt does not invoke PowerShell.

With your own Rust toolchain and matching C compiler:

```powershell
cargo run -p osmanthus-desktop
cargo run -p osmanthus-cli -- demo
cargo run -p osmanthus-cli -- inspect "C:\path\to\application.exe"
cargo run -p osmanthus-cli -- replay tests/fixtures/sample-install.json
cargo run -p osmanthus-cli -- history
```

The default database is `data/evidence.db`, relative to your working directory.
Set `OSMANTHUS_DATA_DIR` to select a different data directory. Run from the project
root for a consistent development database. No account or administrator rights
are needed. File contents are read locally; the tool does not execute inspected
files, contact extracted URLs, or upload evidence.

## Try the foundation

1. Click **Load sample**. Its synthetic provenance remains visible.
2. Select `setup.exe`, its `updater.exe` child, or the unrelated process that
   reuses the installer's PID. Each has its own stable identity and activity.
3. Click **Review a persistence change** to see its exact evidence ID and
   recorded registry location. A review signal is not a malware verdict.
4. Click **Inspect file** to read a local file up to 64 MiB. View SHA-256,
   PE imports, ASCII URL clues, and explicit limitations.
5. Reopen a recording or inspection from **Local history**.
6. Use **Import recording** with a version-1 JSON bundle, such as the sample
   fixture. Imports have no authenticity guarantee. Nothing in a bundle runs.

## Workspace

| Component | Implemented responsibility |
| --- | --- |
| `domain` | Versioned events, boot/start-key identities, evidence and input validation |
| `engine` | Deterministic process tree, timeline, gaps, evidence-linked persistence review |
| `static-analysis` | Read-only SHA-256, PE imports, bounded ASCII URL extraction |
| `store` | SQLite migration, transactional replay, immutable session IDs, report history |
| `telemetry` | Bounded replay import and nonblocking queue with loss counters |
| `ipc` | Read-only query contracts and size/version-checked frame codec; no transport |
| `application` | Local backend shared by desktop and CLI |
| `desktop` | Unprivileged Slint GUI; one background backend worker |
| `cli` | Demo, replay, inspect, history |
| `service` | Explicitly inactive composition shell; no SCM registration or collection |

The local development adapter owns SQLite on its worker. In production, the service
must own the database and the GUI must query through authenticated IPC. Do not
elevate this desktop or expose its current adapter as a privileged service.

## Evidence format and limits

See `tests/fixtures/sample-install.json` for the full format. `schema_version` is
1. `source` is `synthetic` or `imported_recording`; source claims in imported
files are unverified. Every event has an ID, relative `timestamp_ms`, a boot UUID
and start key, and a typed payload. Current event variants are process start/exit,
file create, module load, DNS, connection, and persistence change.

- Recording limit: 8 MiB and 10,000 events; strings up to 8 KiB.
- Reject unknown fields/schema versions, duplicate event or process-start IDs,
  invalid keys, cross-boot parents, and cyclic lineage.
- Missing process starts become visible unresolved nodes, not invented lineage.
- Identical session replay is idempotent; changed evidence under an existing
  session ID is rejected. Stored evidence is not cryptographically authenticated.
- Inspection limit: 64 MiB, up to 512 unique imports and 64 ASCII URL strings.
  The SHA-256 is over the same byte snapshot that is parsed. Concurrent file
  mutation is not prevented by this preliminary reader.
- History displays the newest 100 entries. Long-term retention, encryption,
  service ACLs, provenance signing, and automatic installation history are deferred.
- Domain fields are project-specific; OCSF conformance has not been claimed or validated.

## Verification

```powershell
.\scripts\cargo-local.ps1 build --workspace --locked
.\scripts\cargo-local.ps1 test --workspace --locked
```

Tests cover deterministic replay, PID reuse, missing starts, cycles, duplicate IDs,
database reopen/idempotence/conflicts, known SHA-256 bytes, malformed PE, input
limits, malformed imports, queue loss accounting, and malformed IPC frames.

See [docs/BUILD_STATUS.md](docs/BUILD_STATUS.md) for the requested built/later list.

# osmanthus
