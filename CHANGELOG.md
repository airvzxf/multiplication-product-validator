# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/airvzxf/multiplication-product-validator/releases/tag/v0.1.0) - 2026-11-17

First release.

### Added

- Find every pair of two factors whose product is a given positive integer
  up to 2^128 − 1, printed in ascending order and written to a CSV file
  (`--name` sets its name, `--no-csv` skips it).
- Prime factorization (trial division, Miller–Rabin, Pollard's rho) to find
  the factor pairs; the CLI prints the factorization.
- `--method trial` to run the square-root search for comparison.
- `-j/--threads`, defaulting to the logical cores minus one.
- Prebuilt binaries for Linux (x86_64, aarch64), macOS (aarch64) and
  Windows (x86_64), with `SHA256SUMS` and build-provenance attestations.
  ([#2](https://github.com/airvzxf/multiplication-product-validator/pull/2))

### Fixed

- The square-root bound used an `f64` approximation that overshoots for
  large perfect squares; it now uses the exact integer square root.
- `0` is rejected with a non-zero exit code, and a CSV write failure
  exits with an error.

### Other

- Relicensed under AGPL-3.0-only.
- The crate lives at the repository root; `Cargo.lock` is tracked.
