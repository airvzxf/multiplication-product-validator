//! Find every pair of two factors whose product is a given number.

mod factor;

pub use factor::{factorize, is_prime};
use rayon::prelude::*;

/// A pair `(multiplier, multiplicand)` with `multiplier * multiplicand == n`
/// and `multiplier <= multiplicand`.
pub type FactorPair = (u128, u128);

/// Number of worker threads used when none is requested: every logical
/// core except one, which stays free for the rest of the system.
pub fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map_or(1, |n| n.get().saturating_sub(1))
        .max(1)
}

/// Configures the global rayon pool with `threads` workers.
///
/// Rayon splits the work into small batches and lets an idle worker steal
/// pending batches from a busy one, so no core waits while work remains.
/// Must be called before any parallel work starts.
pub fn init_thread_pool(threads: usize) -> Result<(), rayon::ThreadPoolBuildError> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
}

/// Returns every factor pair of `n`, ordered by ascending multiplier.
///
/// Factorizes `n` into primes and builds the divisors from them, so the
/// cost depends on the size of the prime factors rather than on `n`.
/// Returns an empty vector for `n == 0`.
pub fn factor_pairs(n: u128) -> Vec<FactorPair> {
    factor_pairs_from(n, &factorize(n))
}

/// Builds the factor pairs of `n` from its prime factorization, as returned
/// by [`factorize`]. Returns an empty vector for `n == 0`.
pub fn factor_pairs_from(n: u128, factors: &[(u128, u32)]) -> Vec<FactorPair> {
    if n == 0 {
        return Vec::new();
    }
    let root = n.isqrt();
    let mut divisors = vec![1u128];
    for &(prime, exponent) in factors {
        let previous = divisors.len();
        let mut power = 1u128;
        for _ in 0..exponent {
            power *= prime;
            for i in 0..previous {
                // Every divisor is at most n, so this cannot overflow.
                divisors.push(divisors[i] * power);
            }
        }
    }
    divisors.retain(|&d| d <= root);
    divisors.par_sort_unstable();
    divisors.into_par_iter().map(|d| (d, n / d)).collect()
}

/// Formats a prime factorization as `2^4 * 3^2 * 5`, or `1` when empty.
pub fn format_factorization(factors: &[(u128, u32)]) -> String {
    if factors.is_empty() {
        return "1".to_owned();
    }
    factors
        .iter()
        .map(|&(p, e)| {
            if e == 1 {
                p.to_string()
            } else {
                format!("{p}^{e}")
            }
        })
        .collect::<Vec<_>>()
        .join(" * ")
}

/// Returns every factor pair of `n` by testing each candidate multiplier in
/// `1..=isqrt(n)` in parallel.
///
/// Same result as [`factor_pairs`], but the cost grows with `sqrt(n)`, so it
/// is only practical up to roughly `10^20`. Kept as a reference
/// implementation.
pub fn factor_pairs_trial(n: u128) -> Vec<FactorPair> {
    if n == 0 {
        return Vec::new();
    }
    (1..=n.isqrt())
        .into_par_iter()
        .filter(|&multiplier| n % multiplier == 0)
        .map(|multiplier| (multiplier, n / multiplier))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threads_is_at_least_one() {
        assert!(default_threads() >= 1);
    }

    #[test]
    fn zero_has_no_pairs() {
        assert!(factor_pairs(0).is_empty());
    }

    #[test]
    fn one() {
        assert_eq!(factor_pairs(1), vec![(1, 1)]);
    }

    #[test]
    fn twelve() {
        assert_eq!(factor_pairs(12), vec![(1, 12), (2, 6), (3, 4)]);
    }

    #[test]
    fn perfect_square_lists_the_root_once() {
        assert_eq!(
            factor_pairs(36),
            vec![(1, 36), (2, 18), (3, 12), (4, 9), (6, 6)]
        );
    }

    #[test]
    fn prime() {
        assert_eq!(factor_pairs(1_000_000_007), vec![(1, 1_000_000_007)]);
    }

    #[test]
    fn pairs_are_sorted_and_valid() {
        let n = 720_720;
        let pairs = factor_pairs(n);
        assert_eq!(pairs.len(), 120); // 720720 has 240 divisors.
        assert!(pairs.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(pairs.iter().all(|&(a, b)| a <= b && a * b == n));
    }

    #[test]
    fn trial_and_factorization_agree() {
        for n in 0..20_000 {
            assert_eq!(factor_pairs(n), factor_pairs_trial(n), "n={n}");
        }
        for n in [
            1_000_003u128 * 1_000_003,
            600_851_475_143,
            1 << 40,
            963_761_198_400, // 6720 divisors
        ] {
            assert_eq!(factor_pairs(n), factor_pairs_trial(n), "n={n}");
        }
    }

    #[test]
    fn large_highly_composite_number() {
        // 2^10 * 3^6 * 5^4 * 7^3 * 11^2 * 13^2 * 17 * 19 * 23 * 29 * 31 * 37
        let n: u128 = 1024 * 729 * 625 * 343 * 121 * 169 * 17 * 19 * 23 * 29 * 31 * 37;
        let pairs = factor_pairs(n);
        // (10+1)(6+1)(4+1)(3+1)(2+1)(2+1)(1+1)^6 = 887040 divisors.
        assert_eq!(pairs.len(), 887_040 / 2);
        assert!(pairs.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(pairs.iter().all(|&(a, b)| a <= b && a * b == n));
    }

    #[test]
    fn formats_factorizations() {
        assert_eq!(format_factorization(&[]), "1");
        assert_eq!(format_factorization(&factorize(720)), "2^4 * 3^2 * 5");
    }

    #[test]
    fn u128_max() {
        let pairs = factor_pairs(u128::MAX);
        assert_eq!(pairs.len(), 256); // 9 distinct primes: 2^9 / 2.
        assert!(pairs.iter().all(|&(a, b)| a * b == u128::MAX));
    }
}
