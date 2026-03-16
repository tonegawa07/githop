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

## Releasing (for maintainers)

1. Update `CHANGELOG.md`: move items from `[Unreleased]` to `[x.y.z] - YYYY-MM-DD`
2. Update `version` in `Cargo.toml`
3. Commit: `git commit -m "Release vx.y.z"`
4. Tag and push: `git tag vx.y.z && git push && git push --tags`
5. CI will verify version consistency and publish to crates.io

## Bug Reports

Please file a [bug report](https://github.com/tonegawa07/githop/issues/new?template=bug_report.md) with reproduction steps.

## Feature Requests

Please file a [feature request](https://github.com/tonegawa07/githop/issues/new?template=feature_request.md) describing your use case.
