//! Supporting randomized Hash Functions

use num_bigint::BigUint;
use num_primes::Generator;
use num_traits::{ToPrimitive, Zero};
use rand::Rng;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::Debug,
    time::Instant,
    usize,
};

use crate::printdur;

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

    #[cfg(test)]
    fn init_test() -> Self;
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
pub struct FieldHasher {
    a: BigUint,
    b: BigUint,
    order: BigUint,
    mask: BigUint,
}

impl Debug for FieldHasher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field Hasher - (a = {}, b = {})", self.a, self.b)
    }
}

impl HashFunction for FieldHasher {
    fn init(n: u64, l: u64) -> Self {
        let domain = ((n as f64).log2() + 1.0) as u64;
        let a = Generator::new_uint(domain);
        let b = Generator::new_uint(domain);
        let range = ((l as f64).log2()) as u64;

        let mut mask = BigUint::zero();
        for i in 0..range {
            mask.set_bit(i, true)
        }

        Self {
            a,
            b,
            order: n.into(),
            mask,
        }
    }

    fn compute(&self, x: u64) -> usize {
        let Self {
            a, b, order, mask, ..
        } = self;

        let x: BigUint = x.into();

        let product = (a * x) % order;
        let computed = (product + b) % order;

        println!("{:?}", mask);

        (&computed & mask).to_isize().unwrap() as usize
    }

    #[cfg(test)]
    fn init_test() -> Self {
        Self {
            a: 838u32.into(),
            b: 208u32.into(),
            order: 1024u32.into(),
            mask: BigUint::new(vec![15]),
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
#[deprecated]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn field_hash() {
        let hasher = FieldHasher::init_test();

        let mut roots = vec![];
        for i in 0..1024 {
            let val = hasher.compute(i);
            if val == 0 {
                roots.push(i);
            }
        }
        println!("{:?}", roots);
    }
}
