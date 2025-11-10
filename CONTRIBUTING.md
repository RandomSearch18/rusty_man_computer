# Contributing guide

PRs are welcome!

This guide is quite barebones - if you have any questions, feel free to open an issue or contact [@MMK21](https://github.com/MMK21Hub/).

## Setup

We use Rust and Cargo, as is standard for most Rust projects.

## Publishing a new release

1. Bump version
   - Ensure Cargo.toml and Cargo.lock are commited
2. Run `build_binaries.sh`
   - See the script for usage details
3. Create a GitHub release
   - Auto-generate release notes from PR titles & add some blurb
