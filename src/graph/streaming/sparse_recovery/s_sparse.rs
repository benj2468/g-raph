//! Generalized `s`-Sparse Recovery

use super::one_sparse::{OneSparseRecovery, OneSparseRecoveryOutput};
use crate::{graph::streaming::Query, printdur, start_dur, utils::hash_function::HashFunction};
use num_primes::Generator;

use std::{collections::HashMap, fmt::Debug};

/// `S`-Sparse Recovery Data Structure
///
/// Algorithm for recovery and detection is based off of [Algorithm 15](https://www.cs.dartmouth.edu/~ac/Teach/CS35-Spring20/Notes/lecnotes.pdf)
///
/// Storage: O(tlog(t) + tlog(n))
#[derive(Clone)]
pub struct SparseRecovery<F: HashFunction> {
    /// The domain of the sparse recover structure
    n: u64,
    /// Sparsity Parameter
    ///
    /// Constant Space
    s: u64,
    /// The Sparse Recovery Data Structures
    ///
    /// Stores O(2st) = O(slog(s/del))
    structures: Vec<HashMap<u64, OneSparseRecovery>>,
    /// Hash Functions for hashing to the Sparse recovery systems
    /// Store O(t * HF bits)
    functions: Vec<F>,
    /// One sparse recovery order calculation,
    // this helps speed up finding a prime number for the OneSparseRecover finite field
    order: u64,
}

impl<F: HashFunction> Debug for SparseRecovery<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "---------\n[{}] -> [{}]-Sparse Recovery Structure\n---------",
            self.n, self.s
        )
    }
}

impl<F> SparseRecovery<F>
where
    F: HashFunction,
{
    /// Initialize a new S-Sparse Detection and Recovery Data Structure
    ///
    /// - *n* : Universe Size
    /// - *s* : Sparsity we wish to detect
    /// - *del* : Error probability controller
    pub fn init(n: u64, s: u64, del: f32) -> Self {
        let mut s = s;
        if n < s {
            s = n
        }

        let t = (s as f32 / del).log2().ceil() as u64;

        let order = {
            let prime_bits = (3_f64 * (n as f64).log2()).ceil() as u64 + 1;
            let prime = Generator::new_prime(prime_bits);
            prime
                .to_u32_digits()
                .into_iter()
                .enumerate()
                .fold(0, |val, (i, next)| {
                    let digit_value = 32u64.pow(i as u32) * next as u64;
                    val + digit_value
                })
        };

        let n_pow = n.next_power_of_two();
        let s_pow = (2 * s).next_power_of_two();

        // println!("S-Sparse Setup: t: [{}], [{}] -> [{}]", t, n_pow, s_pow);
        let start = start_dur!();

        let structures = (0..t).into_iter().map(|_| HashMap::new()).collect();

        // printdur!("Structured", start);
        let start = start_dur!();

        let hash_base = F::init(n_pow, s_pow);
        // printdur!("Hash Base", start);
        let functions = (0..t)
            .into_iter()
            .map(|_| hash_base.random_copy())
            .collect();

        // printdur!("Functions", start);

        Self {
            n,
            s,
            structures,
            functions,
            order,
        }
    }

    /// Feed a token into the Structure
    pub fn feed(&mut self, token: (u64, bool)) {
        let Self {
            structures,
            functions,
            n,
            order,
            ..
        } = self;
        let (j, _) = token;

        structures
            .iter_mut()
            .zip(functions.iter())
            .enumerate()
            .for_each(|(_, (recoveries, hasher))| {
                let hashed_index = hasher.compute(j);
                recoveries
                    .entry(hashed_index)
                    .or_insert_with(|| OneSparseRecovery::init_with_order(*n, *order))
                    .feed(token)
            });
    }

    /// Query the Structure for detection and recovery
    ///
    /// The HashMap contains a mapping from indices which are part of the recovery to the values they contained.
    ///
    /// If the stream was not s-sparse, or if one of the one-sparse recovery systems got an answer wrong, then we return `None`.
    pub fn query(self) -> Option<HashMap<u64, i64>> {
        let mut recovery = HashMap::new();

        let mut can_return = false;

        for (_, row) in self.structures.into_iter().enumerate() {
            for (_, (_, cell)) in row.into_iter().enumerate() {
                let res = cell.query();
                match res {
                    OneSparseRecoveryOutput::VeryLikely(lambda, i) => {
                        if recovery
                            .get(&i)
                            .map(|val| val != &lambda)
                            .unwrap_or_default()
                        {
                            return None;
                        }
                        recovery.insert(i, lambda);
                        if recovery.keys().len() > self.s as usize {
                            return None;
                        }
                        can_return = true
                    }
                    _ => continue,
                }
            }
        }
        if can_return {
            Some(recovery)
        } else {
            None
        }
    }
}

impl<F> Query<Option<HashMap<u64, i64>>> for SparseRecovery<F>
where
    F: HashFunction,
{
    fn query(self) -> Option<HashMap<u64, i64>> {
        self.query()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::utils::hash_function::PowerFiniteFieldHasher;

    use super::*;

    fn large_sparse() -> Option<HashMap<u64, i64>> {
        let mut recovery = SparseRecovery::<PowerFiniteFieldHasher>::init(5000, 100, 0.01);

        (0..90)
            .into_iter()
            .for_each(|token| recovery.feed((token, true)));

        recovery.query()
    }

    fn large_not_sparse() -> Option<HashMap<u64, i64>> {
        let mut recovery = SparseRecovery::<PowerFiniteFieldHasher>::init(5000, 100, 0.01);

        (0..400)
            .into_iter()
            .for_each(|token| recovery.feed((token, true)));

        recovery.query()
    }

    #[test]
    fn not_sparse_probability() {
        let n = 100;

        let mut incorrect = 0;

        for _ in 0..n {
            let res = large_not_sparse();
            if res.is_some() {
                incorrect += 1;
            }
        }

        let probability = incorrect as f32 / n as f32;
        assert!(probability <= 0.01);
    }

    #[test]
    fn sparse_probability() {
        let n = 100;

        let mut incorrect = 0;

        for _ in 0..n {
            let res = large_sparse();
            if res.is_none() {
                incorrect += 1;
            }
        }

        let probability = incorrect as f32 / n as f32;
        assert!(probability <= 0.01);
    }

    #[test]
    fn test() {
        let mut recovery = SparseRecovery::<PowerFiniteFieldHasher>::init(5000, 100, 0.01);

        (0..400)
            .into_iter()
            .for_each(|token| recovery.feed((token, true)));

        println!("{:?}", recovery.query())
    }
}
