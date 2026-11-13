# Project Osmanthus

<!-- impeccable:product-schema 1 -->

## Platform
Windows desktop, built in Rust.

## Stack
The earlier project conversation establishes a Cargo workspace, Slint desktop,
pure behavior engine, SQLite evidence store, and a separate Windows service.
This preliminary increment implements a local replay backend, not a service.

## Users and Purpose
A computer owner or security investigator reviews an application's files,
process lineage, outbound endpoints, and persistence evidence to understand
what it might do and what was actually observed.

## Confirmed Constraints
- Static evidence predicts capabilities; runtime evidence records observed behavior.
- A complete behavior tree cannot be known before execution.
- Display evidence and uncertainty rather than promise full protection.
- Keep the GUI unprivileged. Future privileged actions belong to the service.
- Quarantine must be reversible; deletion must be explicitly approved and manifest scoped.
- Complex live collection, detection, containment, and kernel work can wait.

## Evidence on Hand
- Earlier chat: Read project reference document, including the architecture discussion.
- Reference: C:/Users/AlaizaCorp/Downloads/Rust Behavioral Endpoint Monitor Feasibility.pdf.
- No application implementation existed before this increment.
- Sample event recordings must be explicitly identified as synthetic.

## Preliminary Scope
An inspectable Rust foundation: desktop shell, read-only static file inspection,
recording import/replay, process tree, evidence timeline, evidence-linked review
findings, local history, and a CLI. No host execution or automatic remediation.

## Open Decisions
Production deployment, distribution license, retention policy, and the detailed
technical document for later increments remain open. The unrelated Mini Twitter
Phase 1 document is not a project reference.
