---
name: cleaning-up-codebases
description: Use when cleaning a codebase for cruft, dead code, anti-patterns, or scope creep; when reviewing or refactoring; or when making routine edits—apply a "removal over refactoring" lens. Especially for Rust/Qt workspaces, vibe-coded or rapidly developed projects.
---

# Cleaning Up Codebases

## Overview

**Core principle:** Removal over refactoring. Simplification over restructuring. Ask "should this exist?" before "how can I improve this?" The main failure mode is refactoring code that should be deleted, or adding abstractions to an already over-abstracted codebase.

**Apply this lens routinely:** When adding features, refactoring, or reviewing—not only during explicit "cleanup" requests. One question or one quick scan often prevents cruft.

## When to Use

- User asks for cleanup, dead code removal, or "cruft" review
- Reviewing or refactoring existing code
- **Routine edits:** Before adding a new file/feature, ask "does this belong here?"; when touching a module, consider quick dead-code checks
- Vibe-coded or organically grown project; suspected scope creep or architectural drift
- Pre-refactor audit to decide what to keep

**When NOT to use:** Greenfield with nothing to clean; single targeted bug fix; pure performance work (different methodology).

## Process (Full Cleanup)

1. **Understand intent** — Read project docs first: `CLAUDE.md`, README, design docs; then recent git history and dependency manifests (`Cargo.toml`, `package.json`). Identify the gap between stated intent and actual code.
2. **Survey with scans** — Use **Grep** and **SemanticSearch** across the repo. **Count** findings (e.g. "47 `unwrap()` calls") so scope is concrete.
3. **Question existence** — For each major feature/module: Does it align with stated purpose? Is it on a call path from entry points? Could it be a separate project? Is it half-finished? **Never default to "refactor to be better"; first ask "should this exist?"**
4. **Classify** — T1: Safe deletes (dead code, unused files). T2: Quick fixes (isolated improvements). T3: Focused refactors. T4: Architectural changes. Do T1 before T2 before T3; T4 may be a separate effort.
5. **Negotiate scope** — Present findings to the user before planning. Ask what to keep and what to delete. Do not assume what the owner values.
6. **Verify baseline** — Before any changes: `cargo build` / `npm run build`, then `cargo test` / `npm test`. If it does not build, fix that first.
7. **Execute safe-to-dangerous** — T1 → T2 → T3 → T4. After **each** change: build and test; if broken, revert and investigate.
8. **Verify after each change** — Build passes, tests pass, no new warnings. Commit working state.

## Cursor Workflow

- **Grep**: Unused symbols, `unwrap()`/`expect()`, TODO/FIXME/HACK, duplicate patterns, module use vs declaration.
- **SemanticSearch**: "Where is X used?", "What calls this function?", entry points and call graphs.
- **Read**: `CLAUDE.md`, `Cargo.toml`, `qml.qrc`, `build.rs` (cxx-qt file lists) to align with stated architecture and registrations.
- **List_dir / Glob**: Orphan files (e.g. QML not in `qml.qrc`, Rust modules not in `lib.rs` or `build.rs`).
- **Terminal**: `cargo build --release`, `cargo test -p <crate>`, project-specific test commands from docs.

## Project-Aware Signals (Rust / Qt / MyMe-like)

Use these to focus scans; adjust for similar stacks (e.g. other manifests, UI resources).

| Signal | Where to look |
|--------|----------------|
| Dead code | Unused crates in workspace; modules in `lib.rs`/`mod.rs` never referenced; QML files not in `qml.qrc` or never loaded; cxx-qt models in `build.rs` with no QML usage |
| Risk in Rust | `unwrap()`, `expect()` in library/production code (Grep; count them); `block_on()` in UI thread (forbidden in this codebase per CLAUDE.md) |
| Scope creep | Crates or features unrelated to project purpose in README/CLAUDE; dependencies used by a single non-core feature |
| Duplication | Alternate UIs (e.g. `Main.qml` vs `Main-other.qml`); two implementations of the same concern |
| Half-finished | Commented-out blocks; "coming soon" placeholders; feature flags that are always on or off |
| Large files | Files >500 lines; functions >100 lines (Grep or Read to spot) |

## Ongoing Lens (Use During Any Edit)

- **Adding a feature:** Does this belong in this crate/page, or is it scope creep? Is there existing code that already does it?
- **Refactoring:** Am I refactoring something that could be removed entirely? Am I adding new abstractions? Prefer delete over restructure.
- **Reviewing:** Run one quick scan (e.g. Grep for `unwrap()` in the touched crate, or "where is this type used?") and report counts or findings in one sentence.
- **After a change:** Ensure build and tests still pass before moving on.

## Common Mistakes

| Mistake | Do instead |
|---------|------------|
| Refactoring code that should be deleted | Ask "should this exist?" first |
| Adding new abstractions during cleanup | Cleanup = less code, not different code |
| One big multi-phase cleanup plan | Start with T1 deletes; reassess after |
| Flagging as dead without checking usages | Grep/SemanticSearch for all references first |
| Skipping baseline verification | Always confirm build and test pass before starting |
| Deciding scope without user input | Present findings; ask what to keep |

## Red Flags (Cleanup Done Wrong)

- Writing more code than you delete
- Creating new files during a cleanup
- Proposing an "event bus" or "registry pattern" during dead-code removal
- A cleanup plan with 5+ phases spanning weeks
- No confirmation from the owner on what to keep
- Build/test not verified before or after changes
