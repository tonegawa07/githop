# Contributing

Contributions to githop are welcome!

## Development Setup

```bash
git clone https://github.com/tonegawa07/githop.git
cd githop
cargo build
```

## How to Contribute

1. Open an issue to discuss what you'd like to do
2. Fork the repo and create a branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push and open a Pull Request

## Code Quality

Please run the following before submitting a PR:

```bash
cargo test
cargo clippy
cargo fmt --check
```

## Manual Testing

Since githop is a TUI app, automated testing of the UI is limited. Please verify the following manually before submitting a PR:

1. `j` `k` / `↑` `↓` to move cursor
2. `Enter` to switch branch (and exit)
3. `y` to copy branch name to clipboard
4. `d` to delete branch (confirm merged / force unmerged)
5. `n` to create a new branch
6. `r` to rename a branch
7. `/` to filter branches, `Esc` to clear
8. `q` / `Esc` to quit

```bash
cargo run
```

## Releasing (for maintainers)

### Version numbering

This project follows [Semantic Versioning](https://semver.org/) (`MAJOR.MINOR.PATCH`):

- **PATCH** (e.g. 0.1.1 → 0.1.2): Bug fixes, docs, internal improvements
- **MINOR** (e.g. 0.1.2 → 0.2.0): New features (backwards compatible)
- **MAJOR** (e.g. 0.x → 1.0.0): Breaking changes or stable release declaration

### Release steps

```bash
# 1. Update CHANGELOG.md: move [Unreleased] items to [x.y.z] - YYYY-MM-DD
# 2. Update version in Cargo.toml
# 3. Update Cargo.lock
cargo check
# 4. Commit
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "Release vx.y.z"
# 5. Tag and push
git tag vx.y.z
git push && git push --tags
```

CI will automatically:
- Verify that the tag, `Cargo.toml`, and `CHANGELOG.md` versions match
- Publish to crates.io
- Build binaries for macOS (x86_64, aarch64) and Linux (x86_64)
- Create a GitHub Release with the binaries
- Update the Homebrew formula with new checksums

## Bug Reports

Please file a [bug report](https://github.com/tonegawa07/githop/issues/new?template=bug_report.md) with reproduction steps.

## Feature Requests

Please file a [feature request](https://github.com/tonegawa07/githop/issues/new?template=feature_request.md) describing your use case.
