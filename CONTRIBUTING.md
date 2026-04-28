# Contributing to Ignis

Thank you for your interest in contributing. This document describes the branch
strategy, pull-request process, local build setup and the ADR process used in
this project.

---

## Branch strategy

| Branch | Purpose |
|--------|---------|
| `main` | Always-shippable. CI must be green before any merge. |
| `feature/<short-name>` | New features or refactors. Branch from `main`, merge back via PR. |
| `fix/<short-name>` | Bug fixes. Same flow as feature branches. |
| `docs/<short-name>` | Documentation-only changes. Same flow. |

Direct commits to `main` are only allowed for trivial doc fixes (typos, PROGRESS.md
updates without code changes). Anything touching Rust or TypeScript goes through a PR.

---

## Opening a pull request

1. **Fork or branch** off the latest `main`.
2. Make your changes. Keep each PR focused on one logical unit of work.
3. Ensure CI passes locally before pushing:
   ```powershell
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```
   And for the Tray UI:
   ```powershell
   cd tray
   npm ci
   npm run build
   npx tsc --noEmit
   npx eslint src
   ```
4. Write a clear PR title using Conventional Commits style
   (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`).
5. Fill in the PR description: what changed, why, and how to test it.

### Review checklist

The reviewer checks:

- [ ] `cargo clippy -- -D warnings` clean, `cargo fmt --check` passes.
- [ ] No `unwrap()` in production code (only in `#[cfg(test)]` blocks).
- [ ] New public APIs/endpoints documented in `docs/`.
- [ ] PROGRESS.md updated (Changelog `[Unreleased]` section).
- [ ] Non-trivial decisions captured in a new ADR in `DECISIONS.md`.
- [ ] Tests added for new behaviour. No untested happy paths.
- [ ] Read-only constraint: JSONL files are never written, renamed or deleted.

---

## Local build setup

### Prerequisites

| Tool | Version |
|------|---------|
| Rust | 1.75+ (`rustup show`) |
| Node | 20 LTS (`node --version`) |
| WebView2 | Installed (ships with Windows 11) |
| Tauri CLI | `cargo install tauri-cli` |

### Core library + CLI

```powershell
cargo build
cargo test
cargo run --bin ignis -- daily
```

### HTTP API

```powershell
cargo run --bin ignis-api
# Listens on 127.0.0.1:7337
# API token printed on first run; stored in %APPDATA%\ignis\config.json
```

### Tray app (development)

```powershell
cd tray
npm ci
npm run tauri dev      # hot-reload React + Tauri window
```

### Tray app (release build + installer)

```powershell
cd tray
npm ci
npm run tauri build    # produces MSI + NSIS under tray/src-tauri/target/release/bundle/
```

---

## Code style

- **Rust:** `cargo fmt` (default settings), `thiserror` for error enums, `anyhow` only
  at binary entry points. One module = one responsibility; over 300 lines is a
  refactoring signal.
- **TypeScript:** strict mode, ESLint + Prettier, no `any` without a justifying comment.
- **Commits:** Conventional Commits. One logical change per commit.
- **Comments:** only when the *why* is non-obvious. No block comments explaining *what*
  the code does.

---

## ADR process

Any non-trivial decision (architecture, dependency, schema change, security trade-off)
gets a short ADR appended to `DECISIONS.md`:

```markdown
## ADR-NNN — Short title

- **Datum:** YYYY-MM-DD
- **Status:** Accepted | Proposed | Rejected | Superseded
- **Kontext:** Why is this decision needed?
- **Alternativen:**
  - (A) Option A
  - (B) **Chosen option**
- **Entscheidung:** (B).
- **Begründung:** Why this option.
- **Folgen:** What changes as a result.
```

Superseded ADRs are left in place; the new ADR notes *Superseded by ADR-NNN*.

---

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). Be respectful.
