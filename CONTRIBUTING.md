# Contributing to Bromcom Timetable Formatter

Thanks for helping make the timetable easier to read! Please review the guidance below before opening a pull request.

## Development Environment

- Install the latest stable [Rust toolchain](https://www.rust-lang.org/tools/install) with `rustup`.
- Add the targets needed for testing cross-compilation work locally:
  - `rustup target add x86_64-apple-darwin aarch64-apple-darwin`
  - `rustup target add aarch64-unknown-linux-gnu riscv64gc-unknown-linux-gnu x86_64-unknown-freebsd`
- Install [`cross`](https://github.com/cross-rs/cross) if you plan to build non-native GNU targets locally: `cargo install cross` (or `cargo binstall cross`).
- Optional OS packages:
  - Ubuntu: `sudo apt-get install gcc-riscv64-linux-gnu`
  - macOS: install Xcode Command Line Tools (`xcode-select --install`).

## Workflow Expectations

1. **Fork and branch** from `main`. Use meaningful branch names (e.g. `feat/parser-lint`).
2. **Keep changes focused**. Separate unrelated fixes into their own PRs.
3. **GPG-signed conventional commits** are mandatory.
   - Sign every commit with your configured GPG key (`git commit -S ...`).
   - Follow [Conventional Commits](https://www.conventionalcommits.org/) (e.g. `feat(core): add overrides audit`).
4. **Formatting & linting**
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
5. **Tests**
   - `cargo test --workspace`
   - For parser/render changes, include targeted unit tests when possible.
6. **Release workflow validation**
   - If you edit `.github/workflows/release.yml`, the PR will automatically run the build matrix (including macOS Arm runners). Expect longer CI time.

## Pull Request Checklist

Before opening a PR, ensure:

- [ ] All commits are GPG signed and use Conventional Commit messages.
- [ ] `cargo fmt`, `cargo clippy`, and `cargo test` pass locally.
- [ ] Documentation/config examples updated when you add or change behavior.
- [ ] Large features include relevant screenshots or SVG snippets when helpful.
- [ ] You filled out the PR template (see `.github/pull_request_template.md`).

## Testing Cross-compiled Targets Locally

- Use `cross build --target <triple> --release` for Linux/FreeBSD targets.
- macOS binaries are produced on GitHub-hosted runners (Intel: `macos-15-intel`, Arm: `macos-latest`). Testing locally requires the respective hardware.
- Windows targets rely on MSVC toolchains; run `cargo build --target x86_64-pc-windows-msvc` on Windows or let CI cover them.

## Reporting Issues

Open GitHub issues with detailed repro steps, anonymized PDF samples if possible, and the configuration you used. Use the issue templates to help triage faster.

## Code of Conduct

Be respectful and inclusive. The project follows the [GitHub Community Guidelines](https://docs.github.com/site-policy/github-terms/github-community-guidelines).

Thanks again for contributing!
