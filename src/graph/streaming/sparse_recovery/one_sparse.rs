/// Sparse Recover
///
/// This File is a WIP
///
/// Current includes:
///
/// 1. One Sparse Recovery: A One Sparse Recover Data Structure to recover from a stream of fead tokens
use std::{fmt::Debug, time::Instant};

use primes::{PrimeSet, Sieve};
use rand::Rng;

use super::finite_field::*;

/// One Sparse Recovery Data Structure. This includes both the Fingerprint values, and the initializing values, including a finite field to person arithmetic within
#[derive(Clone)]
pub struct OneSparseRecovery {
    /// Fingerprint
    l: i32,
    z: i32,
    p: FieldElement,

    /// Init values
    r: FieldElement,
    n: u64,
    field: FiniteField,
}

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

/// Output for a One S
#[derive(Debug, PartialEq)]
pub enum OneSparseRecoveryOutput {
    Zero,
    /// First element is the value, second element is the index
    VeryLikely(i32, u64),
    NotOneSparse,
}

impl OneSparseRecovery {
    /// Initialize a new `OneSparseRecovery` DS, where the size of our universe is given as `n`.
    pub fn init(n: u64) -> Self {
        let mut rng = rand::thread_rng();
        let order = Sieve::new().find(n.pow(2) as u64).1;

        let r = rng.gen_range(0..order).into();

        let (l, z, p) = (0, 0, 0.into());

        OneSparseRecovery {
            l,
            z,
            p,
            r,
            n,
            field: FiniteField::new(order),
        }
    }
    /// Process a token of some stream into the stream of the `OneSparseRecovery` DS.
    ///
    /// `token = (j, c)`
    ///
    /// Expectations:
    /// 1. `j \in [n]`
    /// 2. `c \in {-1, 1} - false -> -1; true -> 1`
    pub fn feed(&mut self, token: (u64, bool)) {
        let (coordinate, value) = token;
        let value_int = if value { 1 } else { -1 };
        self.l += value_int;
        self.z += value_int * coordinate as i32;

        let power = self.field.pow(self.r, coordinate);

        self.p = if value {
            self.field.add(self.p, power)
        } else {
            self.field.add(self.p, self.field.neg(power))
        };
    }

    /// Query a `OneSparseRecovery` DS. using the mathematical proof from [lecture notes](https://www.cs.dartmouth.edu/~ac/Teach/CS35-Spring20/Notes/lecnotes.pdf#page=41&zoom=100,96,854)
    /// we know that provided the values of our fingerprints we will reach either guaranteed not-one-sparse or very likely one-sparse
    ///
    /// This outputs a false positive with probability: O(1/n^2)
    pub fn query(self) -> OneSparseRecoveryOutput {
        let Self {
            l, z, p, r, field, ..
        } = self;
        if p == 0 && z == 0 && l == z {
            OneSparseRecoveryOutput::Zero
        } else {
            let divided = (z as f32) / (l as f32);
            if divided.round() != divided {
                OneSparseRecoveryOutput::NotOneSparse
            } else if p != field.mul(field.mod_p_i32(l), field.pow(r, divided.round() as u64)) {
                OneSparseRecoveryOutput::NotOneSparse
            } else {
                OneSparseRecoveryOutput::VeryLikely(l, divided.round() as u64)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn true_positive() {
        let stream: Vec<(u64, bool)> = vec![
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
        let stream: Vec<(u64, bool)> = vec![
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
        let stream: Vec<(u64, bool)> = vec![
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
