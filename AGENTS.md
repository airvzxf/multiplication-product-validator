# AGENTS.md

Guidance for AI coding agents and contributors working on this project.

## What the program does

Given a positive integer `n` up to 2^128 − 1, the CLI prints every pair
`a * b = n` with `a <= b`, in ascending order of `a`, plus the prime
factorization of `n`, and writes the pairs to a CSV file. The answer must
be exact for every input in that range and fast even for the hardest
inputs (products of two large primes).

## Layout

- `src/factor.rs`: Montgomery arithmetic on `u128`, Miller–Rabin
  (`is_prime`), Pollard's rho, perfect-power detection and `factorize`.
- `src/factor/ecm.rs`: the elliptic-curve method, used for factors that
  rho does not find quickly.
- `src/lib.rs`: divisor generation, factor pairs, the trial-division
  reference method and the thread pool.
- `src/main.rs`: command-line interface (clap) and CSV output.
- `tests/cli.rs`: end-to-end tests of the binary.
- `tests/exhaustive.rs`: long-running tests, `#[ignore]`d by default.

## Commands

```bash
cargo test                                # unit and CLI tests
cargo test --release -- --include-ignored # plus the long-running tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo deny check
```

All of these must pass before a change is ready.

## Rules

- **Correctness first.** Every arithmetic step must be exact on `u128`.
  Use widening or modular helpers (`mul_wide`, `Montgomery`) instead of
  operations that can overflow. Never use floating point for a bound or
  a result.
- **Two methods, one answer.** `factorize` is the fast path, while
  `factor_pairs_trial` is the slow reference. Any change to factoring
  must keep both methods giving identical output; the tests compare them.
- **Output is stable.** The CLI text and the CSV columns
  (`Multiplier,Multiplicand,Product`) are covered by `tests/cli.rs`.
  Change them only on purpose, and update the tests and README with them.
- **Performance claims need measurements.** Measure with a release build
  over random inputs of several sizes, including products of two primes
  of about 64 bits, and with `-j 1`. Also run the long tests, which catch
  slowdowns that a single benchmark misses.
- **MSRV is Rust 1.85** (`rust-version` in `Cargo.toml`). Do not use
  standard library APIs that are newer than that.
- **No `unsafe`** and no new dependencies without a clear need; the
  program depends only on `rayon` and `clap`.
- **Tests come with the change.** Use known values (primes, pseudoprimes,
  semiprimes) whose properties are verified independently, not assumed.
- Keep the README "How it works" section in sync with the algorithms.
- Commits and pull request titles follow
  [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `perf:`, `docs:`, `test:`, `ci:`, …).
