//! Supporting randomized Hash Functions

use num_bigint::BigUint;
use num_primes::Generator;
use num_traits::{ToPrimitive, Zero};
use std::{fmt::Debug, time::Instant};

use crate::printdur;

/// Describes a Hashing Function from n bits to l bits
///
/// HashFunction the trait provides no guarantee for implementation.
/// As a result, universality of the functions are not consistent across different implementations.
pub trait HashFunction {
    /// Initialize a new hash function. This should
    fn init(n: u64, l: u64) -> Self;
    /// Computes the value of h(x), where h is the current hash function
    fn compute(&self, x: u64) -> usize;
    /// Computes the boolean value of h(x) = *0*, where h is the current hash function
    fn is_zero(&self, x: u64) -> bool {
        self.compute(x) == 0
    }

    #[cfg(test)]
    fn init_test() -> Self;
}

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
/// - order (n bits)
/// - 64 bits (constant)
pub struct FieldHasher {
    a: BigUint,
    b: BigUint,
    order: BigUint,
    domain: u64,
    range: u64,
}

impl Debug for FieldHasher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field Hasher - [{}] -> [{}]", self.domain, self.range)
    }
}

impl HashFunction for FieldHasher {
    fn init(n: u64, l: u64) -> Self {
        let domain = (n as f64).log2().ceil() as u64;
        let a = Generator::new_uint(domain);
        let b = Generator::new_uint(domain);
        let range = (l as f64).log2().ceil() as u64;

        Self {
            a,
            b,
            order: n.into(),
            domain,
            range,
        }
    }

    fn compute(&self, x: u64) -> usize {
        let Self {
            domain,
            a,
            b,
            order,
            range,
            ..
        } = self;

        let x: BigUint = x.into();

        let product = (a * x) % (order - 1_u32);

        let computed: BigUint = (product ^ b) % (order - 1_u32);

        (computed >> (domain - range)).to_isize().unwrap() as usize
    }

    #[cfg(test)]
    fn init_test() -> Self {
        Self {
            domain: 3,
            a: 5u32.into(),
            b: 2u32.into(),
            order: 3u32.into(),
            range: 2,
        }
    }
}

/// A Hash function of the format:
///
/// h(x) = Ax + b
///
/// A := {0,1}^{n,l}
/// b :- {0,1}^l
///
/// A and b are initialized uniformly at random upon initializing the function.
///
/// This is much much slower to initialize than the FieldHasher
///
/// Storage:
/// - a (l * n bits)
/// - b (l bits)
/// - [l constant]
#[derive(Debug, Clone)]
pub struct MatrixHasher {
    a: Vec<BigUint>,
    b: BigUint,
}

impl HashFunction for MatrixHasher {
    fn init(n: u64, l: u64) -> Self {
        let start = Instant::now();
        println!("Initializing {:?} n bit numbers", l);
        let a = (0..l)
            .into_iter()
            .map(|_| {
                let s = Instant::now();
                let res = Generator::new_uint(n);
                printdur!("an n bit number", s);
                res
            })
            .collect();
        let b = Generator::new_uint(l);

        printdur!("Matrix Hasher initialization", start);

        Self { a, b }
    }

    fn compute(&self, x: u64) -> usize {
        let x: BigUint = x.into();
        let a: Vec<u8> = self
            .a
            .iter()
            .map(|a| ((a & &x).count_ones() % 2) as u8)
            .collect();

        (BigUint::from_radix_be(&a, 2).unwrap() ^ &self.b).count_ones() as usize
    }

    #[cfg(test)]
    fn init_test() -> Self {
        let a = vec![BigUint::new(vec![1]), BigUint::new(vec![7])];
        let b = BigUint::new(vec![3]);

        Self { a, b }
    }
}
