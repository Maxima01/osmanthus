---
name: Osmanthus desktop
description: Native Windows application evidence workspace; preliminary implementation record.
colors:
  evergreen-shell: "#193c32"
  sage-workspace: "#f3f5f1"
  sage-toolbar: "#e5ece4"
  heading-ink: "#18392f"
  body-ink: "#33463d"
  secondary-ink: "#56675e"
  toolbar-ink: "#345347"
  shell-title: "#ffffff"
  shell-support: "#d1e3d7"
  provenance-ground: "#fff2d7"
  provenance-ink: "#654511"
  error-ground: "#fbe8e2"
  error-ink: "#892d20"
typography:
  application-name:
    fontFamily: "Segoe UI"
    fontSize: "25px"
    fontWeight: 600
  view-title:
    fontFamily: "Segoe UI"
    fontSize: "21px"
    fontWeight: 600
  section-title:
    fontFamily: "Segoe UI"
    fontSize: "16px"
    fontWeight: 600
  body:
    fontFamily: "Segoe UI"
    fontSize: "14px"
  shell-caption:
    fontFamily: "Segoe UI"
    fontSize: "13px"
spacing:
  action-gap: "10px"
  inspection-gap: "12px"
  group-gap: "14px"
  pane-gap: "18px"
  view-padding: "20px"
  shell-padding: "24px"
components:
  application-header:
    backgroundColor: "{colors.evergreen-shell}"
    textColor: "{colors.shell-title}"
    height: "86px"
    padding: "{spacing.shell-padding}"
  action-toolbar:
    backgroundColor: "{colors.sage-toolbar}"
    textColor: "{colors.toolbar-ink}"
    height: "60px"
  provenance-band:
    backgroundColor: "{colors.provenance-ground}"
    textColor: "{colors.provenance-ink}"
    height: "44px"
  status-bar:
    backgroundColor: "{colors.sage-toolbar}"
    textColor: "{colors.toolbar-ink}"
    height: "42px"
  status-bar-error:
    backgroundColor: "{colors.error-ground}"
    textColor: "{colors.error-ink}"
    height: "42px"
  process-row:
    height: "44px"
  timeline-row:
    height: "40px"
  history-row:
    height: "46px"
  review-row:
    height: "38px"
---

# Design System: Osmanthus desktop

## Overview

**Creative North Star: "Evidence before verdicts"**

This records the implemented Windows Slint evidence viewer. Evergreen and sage
provide a quiet reading field, with amber reserved for source provenance and
uncertainty. Standard Fluent controls provide familiar desktop interaction.
These are provisional implementation choices, not a user-approved brand system.

The visual signature is continuity between an application, its stable process
identity, and the exact source events behind a review signal. The interface
keeps evidence copyable and distinguishes synthetic, imported, and static evidence.

**Key Characteristics:**
- Native controls and a restrained operational layout.
- Persistent provenance and collector status.
- Explicit Root, Child, and Unresolved process labels.
- Read-only, selectable evidence details.

## Colors

The palette separates a dark shell from pale work areas and uses semantic bands
for evidence source and operation errors. Exact values live in the frontmatter.

### Primary
- **Evergreen shell:** application header and its framing identity.
- **Shell title and shell support:** high-contrast title and softer supporting copy.

### Neutral
- **Sage workspace:** the working field behind standard controls.
- **Sage toolbar:** action toolbar and normal status bar.
- **Heading ink:** view and section titles.
- **Body ink:** recording summaries.
- **Secondary ink:** supporting and empty-state copy.
- **Toolbar ink:** normal operation status and toolbar notes.

### Semantic
- **Provenance ground and ink:** synthetic/imported/static source information.
- **Error ground and ink:** failed operations in the status bar.

**The Provenance Rule.** Amber identifies evidence source or uncertainty; red
identifies a failed operation, not an inferred malware verdict.

Native button, tab, selection, and text-edit colors come from Slint's Fluent
style. Their current theme values are not application-owned palette tokens.

## Typography

**Application and Body Font:** Segoe UI, inherited from the window.

This is a native Windows application type voice. The project does not define a
separate decorative display face, a monospace costume, custom tracking, or custom
line-height tokens.

### Hierarchy
- **Application name:** semibold, largest role; identifies the workspace.
- **View title:** semibold; identifies the current task.
- **Section title:** semibold; labels process and evidence areas.
- **Body:** default text, including selectable evidence.
- **Shell caption:** smaller supporting line below the application name.

**The Readable Identity Rule.** Process IDs, boot IDs, start keys, hashes, and
source-event IDs remain readable and copyable text. Display PID values as plain
numbers or explicit unavailable states.

## Layout

The current desktop has a normal viewport of 1180×820 and a minimum of 900×680.
It does not implement mobile layouts or web breakpoints. A fixed header, action
toolbar, provenance band, and status bar frame the tabbed work area.

Views use the extracted view padding and group gaps. Behavior and Timeline use
side-by-side lists and flexible detail panes. The process list is constrained
between 260 and 340 px; the timeline list between 330 and 430 px. Evidence panes
scroll rather than discard content.

**The Working Space Rule.** Size the review list to its actual rows, capped at
76 px, and collapse it when empty so the process and detail panes retain space.

## Elevation & Depth

Application-owned surfaces use flat tonal layering. The Slint source defines no
custom shadows, blur, simulated material, or application motion vocabulary.
Standard control elevation and feedback belong to the Fluent style.

## Shapes

The shell uses full-width rectangular bands and flexible work regions. Buttons,
tabs, and text editors retain their standard Fluent geometry. The application
does not define a reusable corner-radius scale, so no radius tokens are invented.

## Components

### Buttons and list rows

Standard Slint Buttons handle actions and selectable evidence rows. Process rows
are checkable, with their checked state bound to the selected process. Labels
identify Root, Child, or Unresolved relationships; parent identity is included in
the selected process detail. Native hover, focus, checked, and disabled states
are inherited rather than reimplemented.

### Evidence fields

Read-only, word-wrapped TextEdit controls present process activity, event details,
inspection reports, and later work. Their text can be selected and copied.
The application does not define editable text-entry or custom input styling.

### Navigation

A standard TabWidget separates Behavior, Timeline, File inspection, Local history,
and Later work. The native selected tab communicates the active area. There is no
mobile navigation variant.

### Provenance and status

The provenance band changes its text with the evidence context. The status bar
shows progress, completion, cancellation, or failure; a failure changes its
semantic colors while retaining the last successful evidence.

### Dialogs and asynchronous work

Native file dialogs select a recording or file. Backend actions are disabled
while work runs, and dialogs can be canceled. These behaviors preserve the
standard controls' desktop affordances.

The sidecar records native component metadata. It intentionally contains no
invented HTML/CSS previews, synthesized palette ramps, or unobserved theme tokens;
a web preview panel cannot render these Slint controls from this record.

## Do's and Don'ts

### Do:
- **Do** keep synthetic, imported, and static evidence visibly distinguished.
- **Do** keep stable process identity and source-event IDs with their details.
- **Do** preserve standard Fluent interaction states and keyboard affordances.
- **Do** give the scrolling evidence panes the available working space.

### Don't:
- **Don't** use an empty review list to reserve unused vertical space.
- **Don't** imply that an unresolved process has a known parent.
- **Don't** turn a review signal or an empty finding list into a safety verdict.
- **Don't** treat this provisional record as an approved production brand.
