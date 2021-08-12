//! Supporting randomized Hash Functions

use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use primes::is_prime;
use rand::{thread_rng, Rng};
use std::{fmt::Debug, usize};

/// Describes a Hashing Function from n bits to l bits
///
/// HashFunction the trait provides no guarantee for implementation.
/// As a result, universality of the functions are not consistent across different implementations.
pub trait HashFunction: Debug {
    /// Initialize a new hash function. This should
    fn init(n: u64, l: u64) -> Self;
    /// Computes the value of h(x), where h is the current hash function
    fn compute(&self, x: u64) -> usize;
    /// Computes the boolean value of h(x) = *0*, where h is the current hash function
    fn is_zero(&self, x: u64) -> bool {
        self.compute(x) == 0
    }
}

// TODO: We might need to add some sort of bijection here, arbitrary bijection mapping input to x.
/// A Hash function of the format:
///
/// f(x) = ax + b
/// h(x) = leftmost `l` bits of f(x)
///
/// ```latex
/// a := {0,1}^n
/// b :- {0,1}^n
/// ```
///
/// Computations are all performed in F_{2^n}.
///
/// a and b are initialized uniformly at random upon initializing the function.
///
/// Storage:
/// - a (log(n) bits)
/// - b (log(n) bits)
/// - order (log(n) bits)
/// - 64 bits (constant)
/// Total = O(log(n)) bits
#[derive(Debug)]
pub struct FieldHasher {
    a: u64,
    b: u64,
    n: u64,
    mask: u64,
}

impl FieldHasher {
    fn init_a_b(n: u64, a: u64, b: u64, l: u64) -> Self {
        if !is_prime(n) {
            panic!("Hash Function domain MUST be prime: {}", n)
        }
        if !l.is_power_of_two() {
            panic!("Hash Function range MUST be a power of two: {}", l)
        }
        let mask = {
            let mut mask = BigUint::zero();
            for i in 0..(l as f32).log2() as u64 {
                mask.set_bit(i, true)
            }
            mask.to_u64().unwrap()
        };

        Self { a, b, n, mask }
    }
}

impl HashFunction for FieldHasher {
    fn init(n: u64, l: u64) -> Self {
        let mut rng = thread_rng();
        let a = rng.gen_range(0..n);
        let b = rng.gen_range(0..n);

        Self::init_a_b(n, a, b, l)
    }

    fn compute(&self, x: u64) -> usize {
        let Self { a, b, n, mask, .. } = self;

        ((((a * x) + b) % n) & mask) as usize
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use itertools::Itertools;
    use num_traits::Pow;
    use primes::PrimeSet;

    use super::*;

    fn two_universal(n: u64, l: u64) -> (f32, f32) {
        let mut ones_twos = HashMap::<(usize, usize), f32>::new();

        let mut pset = primes::Sieve::new();

        let (_, n) = pset.find(n);
        let l = n.next_power_of_two();

        (0..n)
            .into_iter()
            .cartesian_product((0..n).into_iter())
            .into_iter()
            .for_each(|(a, b)| {
                let hasher = FieldHasher::init_a_b(n, a, b, l);
                let one = hasher.compute(1);
                let two = hasher.compute(2);

                *ones_twos.entry((one, two)).or_default() += 1.0;
            });

        let avg: f32 = ones_twos.values().into_iter().sum::<f32>() / ones_twos.len() as f32;

        let standard_deviation = (ones_twos
            .values()
            .into_iter()
            .fold(0_f32, |red, v| red + (*v as f32 - avg).pow(2))
            / ones_twos.len() as f32)
            .sqrt();

        (avg, standard_deviation)
    }

    #[test]
    fn close() {
        let (_, std) = two_universal(17, 6);
        println!("{:?}", std);

        assert!(std == 0.0);
    }
}
