/// Sparse Recover
///
/// This File is a WIP
///
/// Current includes:
///
/// 1. One Sparse Recovery: A One Sparse Recover Data Structure to recover from a stream of fead tokens
use std::fmt::Debug;

use rand::Rng;

/// A list of Bertrands primes to aid in computing a prime number between n^3 and 2n^3
const BERTRAND_PRIMES: &[u64] = &[
    2, 3, 5, 7, 13, 23, 43, 83, 163, 317, 631, 1259, 2503, 5003, 9973, 19937, 39869, 79699, 159389,
    318751, 637499, 1274989, 2549951, 5099893, 10199767, 20399531, 40799041, 81598067, 163196129,
    326392249, 652784471, 1305568919, 2611137817, 5222275627,
];

/// A structure for containing a finite field, and arithmetic within that field.
///
/// The value contained within the structure is the size of the field
#[derive(Clone, Copy)]
pub struct FiniteField(u64);

impl FiniteField {
    /// Generate a new field of size `size`.
    fn new(size: u64) -> Self {
        Self(size)
    }
    /// Compute base^expo within the field
    fn power(&self, base: i128, expo: u32) -> u64 {
        if expo == 0 {
            return 1;
        }
        self.multiply(base, self.power(base, expo - 1))
    }

    /// Computer v1 * v2 within the field
    fn multiply(&self, v1: i128, v2: u64) -> u64 {
        let v = v1 as i128 * v2 as i128;
        if v < 0 {
            let rev: u64 = -v as u64;
            self.0 - rev
        } else {
            (v as u64).rem_euclid(self.0)
        }
    }

    /// Compute v1 + v2 within the field
    fn addition(&self, v1: u64, v2: u64) -> u64 {
        (v1 + v2).rem_euclid(self.0)
    }

    /// Compute v1 - v2 within the field
    fn subtraction(&self, v1: u64, v2: u64) -> u64 {
        if v2 > v1 {
            let minus = v2 - v1;
            self.0 - minus
        } else {
            v1 - v2
        }
    }
}

/// One Sparse Recovery Data Structure. This includes both the Fingerprint values, and the initializing values, including a finite field to person arithmetic within
#[derive(Clone)]
pub struct OneSparseRecovery {
    /// Fingerprint
    l: i32,
    z: i32,
    p: u64,

    /// Init values
    r: u64,
    n: u32,
    prime_field: FiniteField,
}

#[cfg(test)]
impl Debug for OneSparseRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { l, z, p, r, n, .. } = self;
        write!(
            f,
            "-------\n
        Init Values: r: {:?}, n: {:?}\n
        Fingerprints: l: {:?}, z: {:?}, p: {:?} \n
        -------",
            r, n, l, z, p
        )
    }
}

/// Computes a prime value between n and 2n, using BERTRAND_PRIMES and Chebyshev's Theorem
fn find_prime_n_n2(n: u32) -> u64 {
    let mut left = 0;
    let mut right = BERTRAND_PRIMES.len() as u32;
    loop {
        let insertion_point = ((right - left) / 2) + left;
        let value = *BERTRAND_PRIMES.get(insertion_point as usize).unwrap();
        if n as u64 > value {
            left = insertion_point + 1
        } else if (n as u64) < value {
            right = insertion_point - 1;
        } else {
            return *BERTRAND_PRIMES.get(insertion_point as usize + 1).unwrap();
        }

        if right <= left {
            return value;
        }
    }
}

/// Output for a One S
#[derive(Debug, PartialEq)]
pub enum OneSparseRecoveryOutput {
    Zero,
    VeryLikely(i32, u32),
    NotOneSparse,
}

impl OneSparseRecovery {
    /// Initialize a new `OneSparseRecovery` DS, where the size of our universe is given as `n`.
    pub fn init(n: u32) -> Self {
        let mut rng = rand::thread_rng();
        let prime_field = find_prime_n_n2(n.pow(2));

        let r = rng.gen_range(0..prime_field);

        let (mut l, mut z, mut p) = (0, 0, 0);

        OneSparseRecovery {
            l,
            z,
            p,
            r,
            n,
            prime_field: FiniteField::new(prime_field),
        }
    }
    /// Process a token of some stream into the stream of the `OneSparseRecovery` DS.
    ///
    /// `token = (j, c)`
    ///
    /// Expecations:
    /// 1. `j \in [n]`
    /// 2. `c \in {-1, 1} - false -> -1; true -> 1`
    pub fn feed(&mut self, token: (u32, bool)) {
        let (coordinate, value) = token;
        let value_int = if value { 1 } else { -1 };
        self.l += value_int;
        self.z += value_int * coordinate as i32;

        let power = self.prime_field.power(self.r as i128, coordinate);

        self.p = if value {
            self.prime_field.addition(self.p, power)
        } else {
            self.prime_field.subtraction(self.p, power)
        };
    }

    /// Query a `OneSparseRecovery` DS. using the mathematical proof from [lecture notes](https://www.cs.dartmouth.edu/~ac/Teach/CS35-Spring20/Notes/lecnotes.pdf#page=41&zoom=100,96,854)
    /// we know that provided the values of our fingerprints we will reach either guaranteed not-one-sparse or very likely one-sparse
    ///
    /// This outputs a false positive with probability: O(1/n^2)
    pub fn query(self) -> OneSparseRecoveryOutput {
        let Self {
            l,
            z,
            p,
            r,
            prime_field,
            ..
        } = self;
        if p == 0 && z == 0 && l == z {
            OneSparseRecoveryOutput::Zero
        } else {
            let divided = (z as f32) / (l as f32);
            if divided.round() != divided {
                OneSparseRecoveryOutput::NotOneSparse
            } else if p
                != prime_field.multiply(
                    l as i128,
                    prime_field.power(r as i128, divided.round() as u32),
                )
            {
                OneSparseRecoveryOutput::NotOneSparse
            } else {
                OneSparseRecoveryOutput::VeryLikely(l, divided.round() as u32)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bertrand_primes() {
        let n = 20;

        let prime = find_prime_n_n2(n);

        assert_eq!(prime, 23)
    }

    #[test]
    fn true_positive() {
        let stream: Vec<(u32, bool)> = vec![
            (0, true),
            (9, true),
            (7, true),
            (6, true),
            (7, true),
            (9, true),
            (7, true),
            (9, false),
            (7, false),
            (9, false),
            (7, false),
            (0, false),
            (7, false),
        ];

        let mut recover = OneSparseRecovery::init(10);

        stream.into_iter().for_each(|token| recover.feed(token));

        let res = recover.query();

        assert_eq!(res, OneSparseRecoveryOutput::VeryLikely(1, 6))
    }

    #[test]
    fn true_zero() {
        let stream: Vec<(u32, bool)> = vec![
            (0, true),
            (9, true),
            (7, true),
            (6, true),
            (7, true),
            (9, true),
            (7, true),
            (9, false),
            (7, false),
            (9, false),
            (7, false),
            (0, false),
            (7, false),
            (6, false),
        ];

        let mut recover = OneSparseRecovery::init(10);

        stream.into_iter().for_each(|token| recover.feed(token));

        let res = recover.query();

        assert_eq!(res, OneSparseRecoveryOutput::Zero)
    }

    #[test]
    fn true_negative() {
        let stream: Vec<(u32, bool)> = vec![
            (0, true),
            (9, true),
            (7, true),
            (6, true),
            (7, true),
            (9, true),
            (7, true),
            (9, false),
            (7, false),
            (9, false),
            (7, false),
            (0, false),
        ];

        let mut recover = OneSparseRecovery::init(10);

        stream.into_iter().for_each(|token| recover.feed(token));

        let res = recover.query();

        assert_eq!(res, OneSparseRecoveryOutput::NotOneSparse)
    }
}
