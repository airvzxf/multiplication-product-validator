//! Find every pair of two factors whose product is a given number.

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
/// Each candidate multiplier in `1..=isqrt(n)` is tested in parallel.
/// Returns an empty vector for `n == 0`.
pub fn factor_pairs(n: u128) -> Vec<FactorPair> {
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
}
