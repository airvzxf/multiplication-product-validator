//! Prime factorization of `u128` values.
//!
//! Small factors are removed by trial division; what remains is split with
//! Pollard's rho (Brent's variant) and checked with Miller–Rabin. All modular
//! arithmetic on the large cofactor uses Montgomery multiplication, so no
//! intermediate value ever needs more than 128 bits plus a carry.

use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// Primes below 1000, used for trial division and as Miller–Rabin bases.
const SMALL_PRIMES: [u16; 168] = small_primes();

/// Trial division covers every factor below this bound.
const TRIAL_LIMIT: u128 = 1000;

/// The first 13 primes are a deterministic Miller–Rabin base set for every
/// `n < 3.317e24` (Sorenson & Webster, 2015).
const DETERMINISTIC_BASES: usize = 13;

/// Bound for which [`DETERMINISTIC_BASES`] is proven correct.
const DETERMINISTIC_BOUND: u128 = 3_317_044_064_679_887_385_961_981;

/// Bases used above [`DETERMINISTIC_BOUND`]: the first 25 primes. No proven
/// base set covers all of `u128`; no composite below 2^128 passing these 25
/// bases is known.
const EXTENDED_BASES: usize = 25;

const fn small_primes() -> [u16; 168] {
    let mut primes = [0u16; 168];
    let mut count = 0;
    let mut candidate = 2u16;
    while candidate < 1000 {
        let mut is_prime = true;
        let mut d = 2u16;
        while d * d <= candidate {
            if candidate % d == 0 {
                is_prime = false;
                break;
            }
            d += 1;
        }
        if is_prime {
            primes[count] = candidate;
            count += 1;
        }
        candidate += 1;
    }
    primes
}

/// Full 256-bit product of two `u128` values, as `(high, low)`.
fn mul_wide(a: u128, b: u128) -> (u128, u128) {
    const MASK: u128 = u64::MAX as u128;
    let (a_hi, a_lo) = (a >> 64, a & MASK);
    let (b_hi, b_lo) = (b >> 64, b & MASK);

    let lo_lo = a_lo * b_lo;
    let lo_hi = a_lo * b_hi;
    let hi_lo = a_hi * b_lo;
    let hi_hi = a_hi * b_hi;

    let middle = (lo_lo >> 64) + (lo_hi & MASK) + (hi_lo & MASK);
    let low = (middle << 64) | (lo_lo & MASK);
    let high = hi_hi + (lo_hi >> 64) + (hi_lo >> 64) + (middle >> 64);
    (high, low)
}

/// Montgomery arithmetic modulo an odd `n > 1`, with `R = 2^128`.
#[derive(Clone, Copy)]
struct Montgomery {
    n: u128,
    /// `-n^-1 mod 2^128`.
    n_neg_inv: u128,
    /// `R^2 mod n`, used to enter Montgomery form.
    r2: u128,
    /// `R mod n`, the Montgomery form of 1.
    one: u128,
}

impl Montgomery {
    fn new(n: u128) -> Self {
        debug_assert!(n > 1 && n % 2 == 1);
        // Newton iteration: each step doubles the number of correct low bits
        // (n * n ≡ 1 mod 8 gives three to start with).
        let mut inv = n;
        for _ in 0..6 {
            inv = inv.wrapping_mul(2u128.wrapping_sub(n.wrapping_mul(inv)));
        }
        let one = (u128::MAX % n + 1) % n;
        let mut r2 = one;
        for _ in 0..128 {
            r2 = add_mod(r2, r2, n);
        }
        Self {
            n,
            n_neg_inv: inv.wrapping_neg(),
            r2,
            one,
        }
    }

    /// Montgomery reduction of the 256-bit value `(high, low)`, which must be
    /// below `n * R`. Returns `(high, low) * R^-1 mod n`.
    fn reduce(&self, high: u128, low: u128) -> u128 {
        let m = low.wrapping_mul(self.n_neg_inv);
        let (mn_high, mn_low) = mul_wide(m, self.n);
        // low + mn_low is 0 mod 2^128 by construction; only its carry matters.
        let carry = u128::from(low.overflowing_add(mn_low).1);
        let (sum, overflow_a) = high.overflowing_add(mn_high);
        let (sum, overflow_b) = sum.overflowing_add(carry);
        if overflow_a || overflow_b || sum >= self.n {
            sum.wrapping_sub(self.n)
        } else {
            sum
        }
    }

    fn mul(&self, a: u128, b: u128) -> u128 {
        let (high, low) = mul_wide(a, b);
        self.reduce(high, low)
    }

    fn encode(&self, a: u128) -> u128 {
        self.mul(a % self.n, self.r2)
    }

    fn pow(&self, base: u128, mut exp: u128) -> u128 {
        let mut result = self.one;
        let mut base = base;
        while exp > 0 {
            if exp & 1 == 1 {
                result = self.mul(result, base);
            }
            base = self.mul(base, base);
            exp >>= 1;
        }
        result
    }
}

fn add_mod(a: u128, b: u128, n: u128) -> u128 {
    let (sum, overflow) = a.overflowing_add(b);
    if overflow || sum >= n {
        sum.wrapping_sub(n)
    } else {
        sum
    }
}

fn sub_mod(a: u128, b: u128, n: u128) -> u128 {
    if a >= b {
        a - b
    } else {
        a.wrapping_sub(b).wrapping_add(n)
    }
}

const fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Returns whether `n` is prime.
///
/// Deterministic for `n < 3.317e24`. Above that bound it runs Miller–Rabin
/// with the first 25 prime bases; no composite below 2^128 that passes all
/// of them is known.
pub fn is_prime(n: u128) -> bool {
    if n < 2 {
        return false;
    }
    for &p in &SMALL_PRIMES {
        let p = u128::from(p);
        if n == p {
            return true;
        }
        if n % p == 0 {
            return false;
        }
    }
    if n < TRIAL_LIMIT * TRIAL_LIMIT {
        return true;
    }

    let mont = Montgomery::new(n);
    let n_minus_one = n - 1;
    let shift = n_minus_one.trailing_zeros();
    let odd = n_minus_one >> shift;
    let minus_one = mont.encode(n_minus_one);

    let bases = if n < DETERMINISTIC_BOUND {
        DETERMINISTIC_BASES
    } else {
        EXTENDED_BASES
    };
    SMALL_PRIMES[..bases].iter().all(|&base| {
        let mut x = mont.pow(mont.encode(u128::from(base)), odd);
        if x == mont.one || x == minus_one {
            return true;
        }
        for _ in 1..shift {
            x = mont.mul(x, x);
            if x == minus_one {
                return true;
            }
        }
        false
    })
}

/// Pollard's rho with Brent's cycle detection on the odd composite `n`,
/// using `f(x) = x^2 + c`. Returns a non-trivial factor, or `None` if this
/// `c` failed or `stop` was raised by another worker.
fn pollard_brent(n: u128, c: u128, stop: &AtomicBool) -> Option<u128> {
    /// Differences multiplied together before each gcd.
    const BATCH: u64 = 128;

    let mont = Montgomery::new(n);
    let c = mont.encode(c);
    let step = |x: u128| add_mod(mont.mul(x, x), c, n);

    let mut y = mont.encode(2);
    let mut x = y;
    let mut ys = y;
    let mut q = mont.one;
    let mut g = 1;
    let mut r: u64 = 1;

    while g == 1 {
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        x = y;
        for _ in 0..r {
            y = step(y);
        }
        let mut k = 0;
        while k < r && g == 1 {
            ys = y;
            for _ in 0..BATCH.min(r - k) {
                y = step(y);
                q = mont.mul(q, sub_mod(x, y, n));
            }
            g = gcd(q, n);
            k += BATCH;
        }
        r *= 2;
    }

    if g == n {
        // The batch overshot: replay it one step at a time.
        loop {
            ys = step(ys);
            g = gcd(sub_mod(x, ys, n), n);
            if g != 1 {
                break;
            }
        }
    }
    (g != n).then_some(g)
}

/// Finds a non-trivial factor of the odd composite `n`, trying several rho
/// constants in parallel and keeping the first one that succeeds.
fn find_factor(n: u128) -> u128 {
    let width = rayon::current_num_threads().max(1) as u128;
    let stop = AtomicBool::new(false);
    let mut first = 1;
    loop {
        let found = (first..first + width).into_par_iter().find_map_any(|c| {
            let factor = pollard_brent(n, c, &stop)?;
            stop.store(true, Ordering::Relaxed);
            Some(factor)
        });
        if let Some(factor) = found {
            return factor;
        }
        first += width;
    }
}

/// Largest `r` with `r^k <= n`, for `k >= 2`.
fn integer_root(n: u128, k: u32) -> u128 {
    if k == 2 {
        return n.isqrt();
    }
    // The f64 estimate is within a few units for k >= 3; correct it exactly.
    let mut r = (n as f64).powf(1.0 / f64::from(k)) as u128;
    while r > 0 && r.checked_pow(k).is_none_or(|power| power > n) {
        r -= 1;
    }
    while (r + 1).checked_pow(k).is_some_and(|power| power <= n) {
        r += 1;
    }
    r
}

/// If `n = r^k` for some `k >= 2`, returns `(r, k)` with the largest such `k`.
///
/// Only called once every factor below 1000 is gone, so `r >= 1000` and
/// `k <= log_1000(2^128) < 13`.
fn perfect_power(n: u128) -> Option<(u128, u32)> {
    (2..=13).rev().find_map(|k| {
        let r = integer_root(n, k);
        (r.checked_pow(k) == Some(n)).then_some((r, k))
    })
}

/// Appends the prime factors of `n` (with repetition, unsorted) to `out`.
fn split(n: u128, out: &mut Vec<u128>) {
    if n == 1 {
        return;
    }
    if is_prime(n) {
        out.push(n);
        return;
    }
    // Pollard's rho needs about sqrt(p) steps to find a repeated prime p in
    // p^k, so powers are detected directly.
    if let Some((root, k)) = perfect_power(n) {
        for _ in 0..k {
            split(root, out);
        }
        return;
    }
    let factor = find_factor(n);
    split(factor, out);
    split(n / factor, out);
}

/// Prime factorization of `n` as `(prime, exponent)` pairs in ascending
/// order. Returns an empty vector for `0` and `1`.
pub fn factorize(mut n: u128) -> Vec<(u128, u32)> {
    let mut factors = Vec::new();
    if n == 0 {
        return factors;
    }

    for &p in &SMALL_PRIMES {
        let p = u128::from(p);
        if p * p > n {
            break;
        }
        let mut exponent = 0;
        while n % p == 0 {
            n /= p;
            exponent += 1;
        }
        if exponent > 0 {
            factors.push((p, exponent));
        }
    }

    let mut large = Vec::new();
    split(n, &mut large);
    large.sort_unstable();
    for p in large {
        match factors.last_mut() {
            Some((last, exponent)) if *last == p => *exponent += 1,
            _ => factors.push((p, 1)),
        }
    }
    factors
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference primality test by trial division.
    fn is_prime_naive(n: u128) -> bool {
        n >= 2 && (2..).take_while(|d| d * d <= n).all(|d| n % d != 0)
    }

    #[test]
    fn mul_wide_matches_known_products() {
        assert_eq!(mul_wide(u128::MAX, u128::MAX), (u128::MAX - 1, 1));
        assert_eq!(mul_wide(1 << 64, 1 << 64), (1, 0));
        assert_eq!(mul_wide(12345, 67890), (0, 12345 * 67890));
    }

    #[test]
    fn montgomery_mul_matches_u128_for_small_moduli() {
        for n in [3u128, 1_000_003, (1 << 61) - 1, u64::MAX as u128] {
            let mont = Montgomery::new(n);
            for (a, b) in [(0u128, 5u128), (2, n - 1), (n - 1, n - 1), (123_456, 789)] {
                let (a, b) = (a % n, b % n);
                let product = mont.reduce(0, mont.mul(mont.encode(a), mont.encode(b)));
                assert_eq!(product, a * b % n, "n={n} a={a} b={b}");
            }
        }
    }

    #[test]
    fn integer_root_is_exact() {
        assert_eq!(integer_root(26, 3), 2);
        assert_eq!(integer_root(27, 3), 3);
        assert_eq!(integer_root(u128::MAX, 2), u64::MAX as u128);
        assert_eq!(integer_root(u128::MAX, 3), 6_981_463_658_331);
        assert_eq!(integer_root(1 << 120, 12), 1 << 10);
    }

    #[test]
    fn perfect_power_detection() {
        assert_eq!(perfect_power(1009 * 1009 * 1009), Some((1009, 3)));
        assert_eq!(perfect_power(1009 * 1013), None);
        let p = u64::MAX as u128 - 58;
        assert_eq!(perfect_power(p * p), Some((p, 2)));
    }

    #[test]
    fn small_primes_table() {
        assert_eq!(SMALL_PRIMES[0], 2);
        assert_eq!(SMALL_PRIMES[167], 997);
    }

    #[test]
    fn is_prime_agrees_with_trial_division() {
        for n in 0..50_000 {
            assert_eq!(is_prime(n), is_prime_naive(n), "n={n}");
        }
    }

    #[test]
    fn is_prime_on_large_values() {
        assert!(is_prime(u64::MAX as u128 - 58)); // 2^64 - 59
        assert!(is_prime(u128::MAX - 158)); // 2^128 - 159
        assert!(is_prime((1 << 127) - 1)); // Mersenne prime M127
        assert!(!is_prime(u128::MAX));
        // Strong pseudoprime to every base up to 37.
        assert!(!is_prime(318_665_857_834_031_151_167_461));
        // Carmichael number.
        assert!(!is_prime(3_825_123_056_546_413_051));
    }

    #[test]
    fn factorize_small_values() {
        assert!(factorize(0).is_empty());
        assert!(factorize(1).is_empty());
        assert_eq!(factorize(2), vec![(2, 1)]);
        assert_eq!(
            factorize(720_720),
            vec![(2, 4), (3, 2), (5, 1), (7, 1), (11, 1), (13, 1)]
        );
    }

    #[test]
    fn factorize_semiprime_of_two_32_bit_primes() {
        let (p, q) = (4_294_967_291u128, 4_294_967_279u128);
        assert_eq!(factorize(p * q), vec![(q, 1), (p, 1)]);
    }

    #[test]
    fn factorize_square_of_a_64_bit_prime() {
        let p = u64::MAX as u128 - 58;
        assert_eq!(factorize(p * p), vec![(p, 2)]);
    }

    #[test]
    fn factorize_mixed_powers_of_large_primes() {
        let (p, q) = (1_000_003u128, 1_000_033u128);
        assert_eq!(factorize(p * p * p * q * q), vec![(p, 3), (q, 2)]);
    }

    #[test]
    fn factorize_u128_max() {
        // 2^128 - 1 = 3 * 5 * 17 * 257 * 641 * 65537 * 274177
        //             * 6700417 * 67280421310721
        assert_eq!(
            factorize(u128::MAX),
            vec![
                (3, 1),
                (5, 1),
                (17, 1),
                (257, 1),
                (641, 1),
                (65_537, 1),
                (274_177, 1),
                (6_700_417, 1),
                (67_280_421_310_721, 1),
            ]
        );
    }
}
