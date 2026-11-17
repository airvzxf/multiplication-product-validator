# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/airvzxf/multiplication-product-validator/releases/tag/v0.1.0) - 2026-11-17

### Added

- prime factorization, thread control, CI and release flow ([#2](https://github.com/airvzxf/multiplication-product-validator/pull/2))
- Add argument for the CSV name generator.
- Add automatic generation for the CSV file with the multiplications.
- Add parallelism to optimize the multiplications to find the product.
- To optimize the loop of the multiplications.
- Add the validator of the multiplication's product without optimizations.
- Added a structure for the project in Rust with Rust Rover.

### Other

- relicense to AGPL-3.0
- Add ignore file with a rule for environment file.
- Update the Read Me file with documentation of the application.
- Added the basic Gemini CLI file.
- Update Read Me file.
- Initial commit

### Added

- Prime factorization (trial division, Miller–Rabin, Pollard's rho) to find
  the factor pairs; the CLI prints the factorization.
- `--method trial` to run the previous square-root search for comparison.
- `-j/--threads`, defaulting to the logical cores minus one.
- `--no-csv` to skip writing the CSV file.
- Unit, CLI and long-running tests; CI, CodeQL, Dependabot, nightly
  quality checks, release-plz and a signed-tag release workflow.

### Fixed

- The square-root bound used an `f64` approximation that overshoots for
  large perfect squares; it now uses the exact integer square root.
- `0` is rejected with a non-zero exit code, and a CSV write failure
  exits with an error.

### Changed

- The crate moved to the repository root; `Cargo.lock` is tracked.
