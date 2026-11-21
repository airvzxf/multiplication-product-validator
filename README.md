# Multiplication: Product Validator

[![ci](https://github.com/airvzxf/multiplication-product-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/airvzxf/multiplication-product-validator/actions/workflows/ci.yml)
[![CodeQL](https://github.com/airvzxf/multiplication-product-validator/actions/workflows/codeql.yml/badge.svg)](https://github.com/airvzxf/multiplication-product-validator/actions/workflows/codeql.yml)

Given a number (the product), this application finds and displays every pair
of two factors that multiply to produce that number, along with its prime
factorization. It accepts any positive integer up to 2^128 − 1.

## For Users

### Install

Download the archive for your platform from the
[Releases](https://github.com/airvzxf/multiplication-product-validator/releases)
page, check it against `SHA256SUMS`, and optionally verify its build
provenance:

```bash
gh attestation verify <archive> --repo airvzxf/multiplication-product-validator
```

Or build it from source with Rust 1.85 or newer
([rustup.rs](https://rustup.rs/)):

```bash
cargo install --locked --git https://github.com/airvzxf/multiplication-product-validator
```

### Usage

```bash
multiplication-product-validator 12
```

```
Found 3 multiplications for the product 12.
Prime factorization: 2^2 * 3
1 * 12 = 12
2 * 6 = 12
3 * 4 = 12
CSV file generated: product-validated.csv
```

| Option | Description |
|---|---|
| `--name <NAME>` | Name of the CSV file, without extension (default `product-validated`). |
| `--no-csv` | Do not write the CSV file. |
| `-j, --threads <N>` | Worker threads (default: logical cores minus one). |
| `--method <METHOD>` | `factorize` (default) or `trial`. |

The CSV file has the columns `Multiplier,Multiplicand,Product`.

### How it works

- **`factorize`** (default): factorizes the number into primes and builds
  every divisor from the prime factors. Factors below 1000 are removed by
  trial division and perfect powers are detected directly. Each remaining
  composite goes through a short run of Pollard's rho, which finds small
  factors quickly. If that fails, rho keeps running on one thread while the
  elliptic-curve method (ECM) tries independent curves on all the others,
  and the first factor found stops the rest. Miller–Rabin tests primality.
  Any number up to 2^128 − 1 takes well under a second. The hardest case,
  the product of two primes of about 19 digits each, takes about 0.1 s on
  8 threads.
- **`trial`**: tests every candidate up to the square root, which is only
  practical up to about 10^20. Kept for comparison.

Both methods run on a pool of worker threads that splits the work into
small batches; an idle worker takes pending batches from a busy one, so no
core sits idle while work remains. Results are always in ascending order.

Primality is proven for numbers below 3.317 × 10^24. Above that, the
Miller–Rabin test uses the first 25 primes as bases; no composite below
2^128 is known to pass them all.

## For Developers

```bash
cargo build --release
cargo test                                # unit and CLI tests
cargo test --release -- --include-ignored # plus the long-running tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo deny check
```

`rust-toolchain.toml` pins the toolchain used for formatting, linting and
release builds, so local output matches CI.

### Project structure

```
.
├── Cargo.toml
├── src
│   ├── factor.rs   # Primality test, Pollard's rho, prime factorization
│   ├── factor/
│   │   └── ecm.rs  # Elliptic-curve method
│   ├── lib.rs      # Factor pairs, divisor generation, thread pool
│   └── main.rs     # Command-line interface and CSV output
└── tests
    ├── cli.rs        # End-to-end tests of the binary
    └── exhaustive.rs # Long-running tests (ignored by default)
```

### Continuous integration

| Workflow | When | What |
|---|---|---|
| `ci.yml` | push / PR to `main` | fmt, clippy, tests on Linux/macOS/Windows, rustdoc, release build, cargo-deny, MSRV |
| `quality-nightly.yml` | daily | long-running tests, beta and nightly toolchains, advisory scan |
| `codeql.yml` | push / PR / weekly | CodeQL for Rust and the workflows |
| `release-plz.yml` | push to `main` | opens the release PR |
| `release.yml` | signed `vX.Y.Z` tag | builds, attests and publishes the release |

Dependabot proposes Cargo and GitHub Actions updates weekly.

### Releasing

Commits follow [Conventional Commits](https://www.conventionalcommits.org/).

1. Merge the release PR opened by release-plz. It bumps the version in
   `Cargo.toml` and adds the `CHANGELOG.md` entries.
2. Sign and push the tag on the merge commit:

   ```bash
   git switch main && git pull
   git tag -s vX.Y.Z -m 'release: vX.Y.Z'
   git push origin vX.Y.Z
   ```

3. `release.yml` checks that the tag is signed by a key in
   `.github/trusted-signers`, is on `main` and matches `Cargo.toml`, then
   publishes the GitHub Release.

### Contributing

1. Fork the repository and create a branch (`git switch -c feat/your-feature`).
2. Make your changes, with tests.
3. Commit using Conventional Commits and open a pull request.

## License

[AGPL-3.0-only](LICENSE)
