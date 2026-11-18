//! Long-running checks, ignored by default. The quality-nightly workflow
//! runs them with `cargo test --release -- --ignored`.

use multiplication_product_validator::{factor_pairs, factor_pairs_trial, factorize, is_prime};

/// Deterministic xorshift generator, so a failure is reproducible.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn check_factorization(n: u128) {
    let factors = factorize(n);
    let mut product = 1u128;
    for &(prime, exponent) in &factors {
        assert!(is_prime(prime), "n={n}: {prime} is not prime");
        product *= prime.pow(exponent);
    }
    assert_eq!(product, n, "n={n}: {factors:?}");
    assert!(factors.windows(2).all(|w| w[0].0 < w[1].0), "n={n}");
}

#[test]
#[ignore = "slow; run by the quality-nightly workflow"]
fn both_methods_agree_below_one_million() {
    for n in 0..1_000_000 {
        assert_eq!(factor_pairs(n), factor_pairs_trial(n), "n={n}");
    }
}

#[test]
#[ignore = "slow; run by the quality-nightly workflow"]
fn random_products_of_two_u64_values() {
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    for _ in 0..2_000 {
        // Keep one side below 2^40 so Pollard's rho stays fast.
        let a = u128::from(rng.next() >> 24).max(1);
        let b = u128::from(rng.next()).max(1);
        check_factorization(a * b);
    }
}

#[test]
#[ignore = "slow; run by the quality-nightly workflow"]
fn random_products_of_three_values_up_to_2_pow_40() {
    let mut rng = XorShift(0xD1B5_4A32_D192_ED03);
    for _ in 0..2_000 {
        let n = (0..3)
            .map(|_| u128::from(rng.next() >> (24 + rng.next() % 32)).max(1))
            .product::<u128>();
        check_factorization(n);
    }
}

/// Smallest prime at or above `n`.
fn next_prime(mut n: u128) -> u128 {
    while !is_prime(n) {
        n += 1;
    }
    n
}

#[test]
#[ignore = "slow; run by the quality-nightly workflow"]
fn random_semiprimes_of_two_large_primes() {
    // The hardest inputs for the factorization: two primes of 48 to 64 bits
    // each, so neither trial division nor a short rho run can split them.
    let mut rng = XorShift(0x2545_F491_4F6C_DD1D);
    for _ in 0..200 {
        let p = next_prime(u128::from(rng.next() >> (rng.next() % 17)) | 1 << 47);
        let q = next_prime(u128::from(rng.next() >> (rng.next() % 17)) | 1 << 47);
        let mut expected = vec![(p.min(q), 1), (p.max(q), 1)];
        if p == q {
            expected = vec![(p, 2)];
        }
        assert_eq!(factorize(p * q), expected, "p={p} q={q}");
    }
}
