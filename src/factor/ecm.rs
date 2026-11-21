//! Lenstra's elliptic-curve method (ECM).
//!
//! Each curve is an independent attempt to split `n`: a curve over the
//! integers mod `n` reduces to a curve mod every prime factor `p`, and the
//! attempt succeeds when the order of that curve mod `p` is smooth (all its
//! prime factors are small). Curves are tried in parallel on the rayon pool,
//! and the first one to find a factor stops the others.
//!
//! The curves are Montgomery curves `B·y² = x³ + A·x² + x` built with
//! Suyama's parametrization (group order divisible by 12), and only the `X`
//! and `Z` projective coordinates are tracked. `(A + 2) / 4` is kept as the
//! fraction `a24_num / a24_den`, so no modular inverse is ever needed.

use super::{Montgomery, add_mod, gcd, sub_mod};
use rayon::prelude::*;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// One effort level: stage-1 bound, stage-2 bound and number of curves.
struct Level {
    b1: u64,
    b2: u64,
    curves: u64,
}

/// Effort levels tried in order. The bounds follow GMP-ECM's recommended
/// values for factors of up to 15 and 20 digits; every prime factor that
/// matters here is below 2^64 (about 19.3 digits), since a composite `u128`
/// always has a prime factor at most `sqrt(2^128)`. The last level repeats
/// until a factor is found.
const LEVELS: [Level; 3] = [
    Level {
        b1: 2_000,
        b2: 150_000,
        curves: 30,
    },
    Level {
        b1: 11_000,
        b2: 1_100_000,
        curves: 120,
    },
    Level {
        b1: 50_000,
        b2: 5_000_000,
        curves: u64::MAX,
    },
];

/// Stage-2 giant-step size: baby steps are the `j < D/2` coprime to `D`.
const D: u64 = 2310;

/// Projective point `(X : Z)` on a Montgomery curve, in Montgomery form.
#[derive(Clone, Copy)]
struct Point {
    x: u128,
    z: u128,
}

/// A Montgomery curve mod `n`, through `(A + 2) / 4 = a24_num / a24_den`.
struct Curve {
    mont: Montgomery,
    a24_num: u128,
    a24_den: u128,
}

impl Curve {
    /// Builds the Suyama curve for `sigma >= 6` and its starting point.
    /// Returns `Err(factor)` in the rare case where the construction itself
    /// reveals a factor of `n`.
    fn suyama(mont: Montgomery, sigma: u128) -> Result<(Self, Point), u128> {
        let n = mont.n;
        let m = |a, b| mont.mul(a, b);
        let sigma = mont.encode(sigma);
        let five = mont.encode(5);
        let three = mont.encode(3);
        let four = mont.encode(4);
        let sixteen = mont.encode(16);

        // u = sigma^2 - 5, v = 4 sigma, x0 = u^3, z0 = v^3.
        let u = sub_mod(m(sigma, sigma), five, n);
        let v = m(four, sigma);
        let u3 = m(m(u, u), u);
        let v3 = m(m(v, v), v);

        // (A + 2) / 4 = (v - u)^3 (3u + v) / (16 u^3 v).
        let v_minus_u = sub_mod(v, u, n);
        let a24_num = m(
            m(m(v_minus_u, v_minus_u), v_minus_u),
            add_mod(m(three, u), v, n),
        );
        let a24_den = m(m(sixteen, u3), v);

        let g = gcd(mont.reduce(0, a24_den), n);
        if g != 1 {
            return Err(g);
        }
        Ok((
            Self {
                mont,
                a24_num,
                a24_den,
            },
            Point { x: u3, z: v3 },
        ))
    }

    /// `2P`.
    fn double(&self, p: Point) -> Point {
        let (m, n) = (&self.mont, self.mont.n);
        let sum = add_mod(p.x, p.z, n);
        let diff = sub_mod(p.x, p.z, n);
        let t1 = m.mul(sum, sum);
        let t2 = m.mul(diff, diff);
        let t3 = sub_mod(t1, t2, n);
        Point {
            x: m.mul(m.mul(t1, t2), self.a24_den),
            z: m.mul(
                t3,
                add_mod(m.mul(t2, self.a24_den), m.mul(self.a24_num, t3), n),
            ),
        }
    }

    /// `P + Q`, given `P - Q`.
    fn add(&self, p: Point, q: Point, difference: Point) -> Point {
        let (m, n) = (&self.mont, self.mont.n);
        let u = m.mul(sub_mod(p.x, p.z, n), add_mod(q.x, q.z, n));
        let v = m.mul(add_mod(p.x, p.z, n), sub_mod(q.x, q.z, n));
        let sum = add_mod(u, v, n);
        let diff = sub_mod(u, v, n);
        Point {
            x: m.mul(difference.z, m.mul(sum, sum)),
            z: m.mul(difference.x, m.mul(diff, diff)),
        }
    }

    /// `k·P` for `k >= 1`, with the Montgomery ladder.
    fn multiply(&self, k: u64, p: Point) -> Point {
        debug_assert!(k >= 1);
        let mut r0 = p;
        let mut r1 = self.double(p);
        for bit in (0..63 - k.leading_zeros()).rev() {
            if (k >> bit) & 1 == 1 {
                r0 = self.add(r1, r0, p);
                r1 = self.double(r1);
            } else {
                r1 = self.add(r0, r1, p);
                r0 = self.double(r0);
            }
        }
        r0
    }
}

/// Prime sieve for each level, up to its `b2 + D`, as a bitset. Each one is
/// built the first time its level runs, so a factor found at the first
/// level never pays for the larger sieves.
fn sieve(level: usize) -> &'static [u64] {
    static SIEVES: [OnceLock<Vec<u64>>; LEVELS.len()] = [const { OnceLock::new() }; LEVELS.len()];
    SIEVES[level].get_or_init(|| {
        let limit = (LEVELS[level].b2 + D) as usize;
        let mut bits = vec![u64::MAX; limit / 64 + 1];
        bits[0] &= !0b11; // 0 and 1 are not prime.
        let mut i = 2;
        while i * i <= limit {
            if bits[i / 64] >> (i % 64) & 1 == 1 {
                for multiple in (i * i..=limit).step_by(i) {
                    bits[multiple / 64] &= !(1 << (multiple % 64));
                }
            }
            i += 1;
        }
        bits
    })
}

fn is_small_prime(sieve: &[u64], k: u64) -> bool {
    let k = k as usize;
    sieve[k / 64] >> (k % 64) & 1 == 1
}

/// Runs one curve. Returns a non-trivial factor of `n`, or `None` if this
/// curve failed or `stop` was raised by another worker.
fn run_curve(n: u128, sigma: u128, level_index: usize, stop: &AtomicBool) -> Option<u128> {
    let level = &LEVELS[level_index];
    let mont = Montgomery::new(n);
    let (curve, mut q) = match Curve::suyama(mont, sigma) {
        Ok(curve) => curve,
        Err(factor) => return (factor != n).then_some(factor),
    };
    let primes = sieve(level_index);

    // Stage 1: Q = (product of every prime power up to B1) · P.
    for (index, p) in (2..=level.b1)
        .filter(|&p| is_small_prime(primes, p))
        .enumerate()
    {
        let mut power = p;
        while power <= level.b1 / p {
            power *= p;
        }
        q = curve.multiply(power, q);
        if index % 8 == 7 && stop.load(Ordering::Relaxed) {
            return None;
        }
    }
    let g = gcd(mont.reduce(0, q.z), n);
    if g != 1 {
        return (g != n).then_some(g);
    }

    // Stage 2 (baby-step giant-step): catch an order of the form s·r with s
    // B1-smooth and one prime r in (B1, B2]. For r = k·D ± j,
    // x(k·D·Q) = x(j·Q) mod p exactly when r·Q vanishes mod p, so the cross
    // product X_G·Z_j − X_j·Z_G collects that factor of p.
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    let babies: Vec<(u64, Point)> = (1..D / 2)
        .filter(|&j| gcd(u128::from(j), u128::from(D)) == 1)
        .map(|j| (j, curve.multiply(j, q)))
        .collect();

    let step = curve.multiply(D, q);
    let first = (level.b1 / D).max(1);
    // `previous` is k·D·Q and `giant` is (k + 1)·D·Q.
    let mut previous = curve.multiply(first * D, q);
    let mut giant = curve.multiply((first + 1) * D, q);
    let mut accumulator = mont.one;
    let mut k = first;
    while k * D <= level.b2 + D {
        let current = previous;
        for &(j, baby) in &babies {
            let low = k * D - j;
            let high = k * D + j;
            let hit = |r: u64| r > level.b1 && r <= level.b2 && is_small_prime(primes, r);
            if hit(low) || hit(high) {
                let cross = sub_mod(mont.mul(current.x, baby.z), mont.mul(baby.x, current.z), n);
                accumulator = mont.mul(accumulator, cross);
            }
        }
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        // (k + 2)·D·Q = (k + 1)·D·Q + D·Q, whose difference is k·D·Q.
        let next = curve.add(giant, step, previous);
        previous = giant;
        giant = next;
        k += 1;
    }
    let g = gcd(mont.reduce(0, accumulator), n);
    (g != 1 && g != n).then_some(g)
}

/// Finds a non-trivial factor of the odd composite `n` (not a perfect
/// power, no factor below 1000), running curves in parallel until one
/// succeeds. Returns `None` only if `stop` is raised first.
pub(super) fn find_factor(n: u128, stop: &AtomicBool) -> Option<u128> {
    let width = rayon::current_num_threads().max(1) as u64;
    let mut sigma = 6u128;
    for (level_index, level) in LEVELS.iter().enumerate() {
        let mut tried = 0;
        while tried < level.curves {
            if stop.load(Ordering::Relaxed) {
                return None;
            }
            let batch = width.min(level.curves - tried);
            let found = (0..batch)
                .into_par_iter()
                .find_map_any(|i| run_curve(n, sigma + u128::from(i), level_index, stop));
            if found.is_some() {
                return found;
            }
            sigma += u128::from(batch);
            tried += batch;
        }
    }
    unreachable!("the last ECM level repeats until it finds a factor")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two projective points are equal when X1·Z2 = X2·Z1.
    fn same_point(curve: &Curve, a: Point, b: Point) -> bool {
        curve.mont.mul(a.x, b.z) == curve.mont.mul(b.x, a.z)
    }

    #[test]
    fn ladder_is_consistent_with_doubling_and_composition() {
        // Over a prime field the curve arithmetic is exact.
        let p = u128::from(u64::MAX) - 58;
        let (curve, base) = Curve::suyama(Montgomery::new(p), 11).unwrap();
        assert!(same_point(
            &curve,
            curve.multiply(2, base),
            curve.double(base)
        ));
        let three = curve.add(curve.double(base), base, base);
        assert!(same_point(&curve, curve.multiply(3, base), three));
        for (a, b) in [(7u64, 13u64), (210, 2310), (1023, 4097)] {
            let left = curve.multiply(a * b, base);
            let right = curve.multiply(a, curve.multiply(b, base));
            assert!(same_point(&curve, left, right), "a={a} b={b}");
        }
    }

    #[test]
    fn sieve_marks_primes() {
        let s = sieve(LEVELS.len() - 1);
        let primes: Vec<u64> = (0..30).filter(|&k| is_small_prime(s, k)).collect();
        assert_eq!(primes, [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
        assert!(is_small_prime(s, 1_000_003));
        assert!(!is_small_prime(s, 1_000_001)); // 101 · 9901
        assert!(!is_small_prime(s, LEVELS[LEVELS.len() - 1].b2));
    }

    #[test]
    fn splits_a_semiprime_of_two_64_bit_primes() {
        let (p, q) = (u128::from(u64::MAX) - 58, u128::from(u64::MAX) - 82);
        let factor = find_factor(p * q, &AtomicBool::new(false)).unwrap();
        assert!(factor == p || factor == q, "factor={factor}");
    }

    #[test]
    fn splits_a_semiprime_with_a_small_factor() {
        let (p, q) = (1_000_003u128, u128::from(u64::MAX) - 58);
        let factor = find_factor(p * q, &AtomicBool::new(false)).unwrap();
        assert!(factor == p || factor == q, "factor={factor}");
    }

    #[test]
    fn stops_when_asked() {
        let (p, q) = (u128::from(u64::MAX) - 58, u128::from(u64::MAX) - 82);
        assert_eq!(find_factor(p * q, &AtomicBool::new(true)), None);
    }
}
