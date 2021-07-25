//! Supporting randomized Hash Functions

use num_bigint::BigUint;
use num_primes::Generator;
use num_traits::pow::Pow;

/// Describes a Hashing Function from n bits to l bits
///
/// HashFunction the trait provides no guarantee for implementation.
/// As a result, universality of the functions are not consistent across different implementations.
pub trait HashFunction {
    /// Initialize a new hash function. This should
    fn init(n: u64, l: u64) -> Self;
    /// Computes the value of h(x), where h is the current hash function
    fn compute(&self, x: u64) -> Vec<u8>;
    /// Computes the boolean value of h(x) = *0*, where h is the current hash function
    fn is_zero(&self, x: u64) -> bool {
        self.compute(x).iter().find(|x| **x > 0).is_none()
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
pub struct FieldHasher {
    a: BigUint,
    b: BigUint,
    bits: BigUint,
    l: u64,
}

impl HashFunction for FieldHasher {
    fn init(n: u64, l: u64) -> Self {
        let a = Generator::new_uint(n);
        let b = Generator::new_uint(n);

        Self {
            a,
            b,
            bits: Pow::pow(BigUint::from(2u32), BigUint::from(n)),
            l,
        }
    }

    fn compute(&self, x: u64) -> Vec<u8> {
        let Self { a, b, bits, .. } = self;
        let x: BigUint = x.into();

        let computed = (((a * x) % bits) + b) % bits;

        computed
            .to_radix_be(2)
            .into_iter()
            .rev()
            .take(self.l as usize)
            .collect()
    }

    #[cfg(test)]
    fn init_test() -> Self {
        Self {
            a: 5u32.into(),
            b: 2u32.into(),
            bits: 3u32.into(),
            l: 2,
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
#[derive(Debug, Clone)]
pub struct MatrixHasher {
    a: Vec<BigUint>,
    b: BigUint,
}

impl HashFunction for MatrixHasher {
    fn init(n: u64, l: u64) -> Self {
        let a = (0..l).into_iter().map(|_| Generator::new_uint(n)).collect();
        let b = Generator::new_uint(l);

        Self { a, b }
    }

    fn compute(&self, x: u64) -> Vec<u8> {
        let x: BigUint = x.into();
        let a: Vec<u8> = self
            .a
            .iter()
            .map(|a| ((a & &x).count_ones() % 2) as u8)
            .collect();

        (BigUint::from_radix_be(&a, 2).unwrap() ^ &self.b).to_radix_be(2)
    }

    #[cfg(test)]
    fn init_test() -> Self {
        let a = vec![BigUint::new(vec![1]), BigUint::new(vec![7])];
        let b = BigUint::new(vec![3]);

        Self { a, b }
    }
}
