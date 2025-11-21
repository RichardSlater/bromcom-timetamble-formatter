# v0.1.0 — Initial release

This is the project initial release (v0.1.0).

Summary of initial commits (starting from the repository bootstrap / initial commit):

- Project scaffold and Cargo workspace with two crates:
  - `crates/core` — library containing parser, renderer, processor, configuration and tests
  - `crates/cli` — command-line binary wrapper for the core library

- Features added in initial commit:
  - PDF parsing of Bromcom timetables (parser.rs)
  - SVG renderer for weekly timetables (renderer.rs)
  - Map processor for SVG map highlights (processor.rs)
  - Configuration and mapping support (`config.toml` and `crates/core/src/config.rs`)
  - Unit tests for core behaviors (parsing, processing, rendering)

- Developer tooling and housekeeping:
  - CI workflow for build/test (`.github/workflows/ci.yml`)
  - Release workflow (`.github/workflows/release.yml`)
  - Project guidance and safety instructions in `.github/instructions` and `.github/copilot-instructions.md`
  - Pre-commit and formatting configs

Notes:
- The `v0.1.0` tag has been created and pushed to `origin`.
- If you would like, I can create the official GitHub Release entry using the GitHub API (requires a token) or via the web UI. Currently, this repository has the tag and a local release-notes file committed.

---

Files added by the initial commit (abridged):
- `Cargo.toml`, `README.md`, `LICENSE`, `config.toml`
- `crates/core/src/{config.rs,lib.rs,parser.rs,processor.rs,renderer.rs}`
- `crates/cli/src/main.rs`
- Github workflows and prompt/instructions files

