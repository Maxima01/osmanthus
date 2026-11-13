# Preliminary build status

## Built in this increment

- Rust Cargo workspace with separated domain, engine, static analysis, storage,
  telemetry, protocol, application backend, desktop, CLI, and service shell.
- Slint desktop with Behavior, Timeline, File inspection, Local history, and Later
  work views, background processing, file pickers, and loading/error/empty states.
- Explicitly labeled synthetic installer walkthrough, including a child process,
  created file, loaded module, DNS query, connection, startup-registry evidence,
  and an unrelated process that reuses a PID.
- Validated JSON recording import/replay, deterministic event ordering, stable
  boot/start-key process trees, unresolved-node reporting, and source event IDs.
- A basic deterministic persistence review signal with a plain-language
  explanation and linked evidence. This is not YARA/Sigma or malware classification.
- Read-only inspection of local files: SHA-256, PE format/imports, ASCII URL clues,
  bounded parsing, and explicit unknowns. Inspected files are never executed.
- SQLite migration and transactions, idempotent replay, session-ID conflict
  refusal, recording/report history, and reopening saved analysis.
- CLI commands for demo, replay, inspect, and history.
- Nonblocking event-queue primitive with dropped-event counters.
- Versioned, bounded, read-only IPC frame contracts. No named-pipe server exists.
- Inactive service entry point. No service installation or privileged collection.
- Replay/storage/parser/limit/queue/protocol regression tests.

## For later — next observation increment

- Windows service installation and SCM lifecycle, dedicated service identity,
  shutdown supervision, structured operational logging, and recovery.
- Live ETW process, image-load, file, registry, DNS, and TCP/UDP collectors.
- Reliable Windows boot/start-key extraction, raw event retention, normalized
  schema mapping, clock calibration, event deduplication, and start/exit state.
- Collector health, provider watchdog, real ETW loss metrics, load testing, and
  bounded worker scheduling. The current queue is only a tested primitive.
- Hardened named-pipe transport: DACLs, first-instance protection, remote-client
  rejection, client/server authentication, authorization, rate limits, and tests.
- Move database ownership from the local development adapter to the service;
  the production GUI must only use authenticated IPC.

## For later — analysis and evidence

- Installed-application discovery, installer sessions/manifests, USN Journal
  correlation, file identity, dependency graphs, and dropped-file relationships.
- Authenticode chain/validity verification, signature provenance, deeper PE
  resources/exports, packer/entropy checks, dynamic imports, embedded payloads,
  script analysis, and optional reputation feeds.
- YARA-X and Sigma integrations with rule management and replay validation.
- Service/task/startup/WMI persistence detection; the current UI only reviews an
  explicitly supplied persistence-change event.
- Process-attributed DNS/network callback evidence, connection outcome and TLS
  metadata, and a clear split between attempted and successful connections.
- OCSF validation, signed evidence export/import, provenance, history search,
  retention policy, privacy controls, tamper resistance, and durable migrations.

## For later — response and isolated execution

- Reversible quarantine vault, restore, receipts, crash recovery, and user review.
- Process-tree response with fresh identity checks and propagation limitations.
- Explicitly confirmed manifest-scoped deletion/uninstall; no guessed-directory deletion.
- Disposable snapshot-protected VM agent, offline and controlled-egress passes,
  snapshot differences, and evidence-bundle transfer.
- Kernel minifilter research only in an isolated test-signing VM, after the
  user-mode observation and response layers are stable.
- Packaging, distribution licensing, signing, installer/updater, accessibility
  review, and broader Windows compatibility/performance/security testing.

## Boundary

This is increment 0 (foundation), not completion of the full Phase 1 telemetry
MVP. Sample events are synthetic. Imported recordings are unverified. Static
analysis cannot reveal a complete future behavior tree, and absence of review
signals does not establish that an application is safe.

## Verification completed

- Workspace build succeeded on Windows with the local Rust toolchain.
- All 12 tests passed: 11 core regression tests and one desktop rendering test
  covering seven captures across normal and minimum supported window sizes.
- CLI replay, real PE inspection, local history, and inactive service shell were
  exercised. The compiled service PE yielded 190 imports during inspection.
- The three desktop review fixes were scored resolved: explicit lineage,
  compact-window working space, and readable PID/count labels.
- Native file-picker interaction and installation on another Windows machine
  have not been manually exercised; deployment remains later work.

