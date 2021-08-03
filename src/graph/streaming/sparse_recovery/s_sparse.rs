//! More Generalized s-Sparse Recovery

use super::one_sparse::{OneSparseRecovery, OneSparseRecoveryOutput};
use crate::utils::hash_function::HashFunction;
use num_primes::Generator;
use std::collections::HashMap;

/// S-Sparse Recovery Data Structure
///
/// Algorithm for recovery and detection is based off of [Algorithm 15](https://www.cs.dartmouth.edu/~ac/Teach/CS35-Spring20/Notes/lecnotes.pdf)
///
/// Storage: O(tlog(t) + tlog(n))
#[derive(Clone)]
pub struct SparseRecovery<F: HashFunction> {
    /// Sparsity Parameter
    ///
    /// Constant Space
    s: u64,
    /// The Sparse Recovery Data Structures
    ///
    /// Stores O(2st) = O(slog(s/del))
    structures: Vec<Vec<OneSparseRecovery>>,
    /// Hash Functions for hashing to the Sparse recovery systems
    /// Store O(t * HF bits)
    functions: Vec<F>,
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

        let structures = (0..t)
            .into_iter()
            .map(|_| {
                (0..(2 * s))
                    .into_iter()
                    .map(|_| OneSparseRecovery::init_with_order(n, order))
                    .collect()
            })
            .collect();

        let functions = (0..t).into_iter().map(|_| F::init(n, 2 * s)).collect();

        Self {
            s,
            structures,
            functions,
        }
    }

    /// Feed a token into the Structure
    pub fn feed(&mut self, token: (u64, bool)) {
        let Self {
            s,
            structures,
            functions,
        } = self;
        let (j, _) = token;
        structures
            .iter_mut()
            .zip(functions.iter())
            .enumerate()
            .for_each(|(i, (recoveries, hasher))| {
                let hashed_index = hasher.compute(j);
                if let Some(recovery) = recoveries.get_mut(hashed_index) {
                    recovery.feed(token);
                };
            });
    }

    /// Query the Structure for detection and recovery
    ///
    /// The HashMap contains a mapping from indices which are part of the recovery to the values they contained.
    ///
    /// If the stream was not s-sparse, or if one of the one-sparse recovery systems got an answer wrong, then we return `None`.
    pub fn query(self) -> Option<HashMap<u64, i64>> {
        let mut recovery = HashMap::new();

        for row in self.structures {
            for cell in row {
                match cell.query() {
                    OneSparseRecoveryOutput::VeryLikely(lambda, i) => {
                        if lambda != 0 {
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
                        }
                    }
                    _ => continue,
                }
            }
        }
        Some(recovery)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{graph::streaming::sparse_recovery::s_sparse, utils::hash_function::FieldHasher};

    use super::*;

    fn is_s_sparse() -> bool {
        let stream: Vec<(u64, bool)> = vec![
            (0, true),
            (9, true),
            (7, true),
            (6, true),
            (7, true),
            (9, true),
            (7, true),
            (9, false),
            (9, false),
        ];

        let mut recovery = SparseRecovery::<FieldHasher>::init(10, 3, 0.01);

        stream.into_iter().for_each(|token| recovery.feed(token));

        let mut expected: HashSet<(u64, i64)> = HashSet::new();
        expected.insert((0, 1));
        expected.insert((6, 1));
        expected.insert((7, 3));

        recovery
            .query()
            .map(|actual| {
                actual
                    .into_iter()
                    .collect::<HashSet<(u64, i64)>>()
                    .is_subset(&expected)
            })
            .unwrap_or_default()
    }

    fn not_s_sparse() -> Option<HashMap<u64, i64>> {
        let stream: Vec<(u64, bool)> = vec![
            (1, true),
            (2, true),
            (3, true),
            (4, true),
            (5, true),
            (6, true),
            (7, true),
            (8, true),
            (9, true),
            (9, true),
            (9, true),
            (9, true),
            (9, true),
            (9, true),
        ];

        let mut recovery = SparseRecovery::<FieldHasher>::init(10, 3, 0.01);

        stream.into_iter().for_each(|token| recovery.feed(token));

        recovery.query()
    }

    #[test]
    fn not_sparse_probability() {
        let n = 100;

        let mut incorrect = 0;

        for _ in 0..n {
            if not_s_sparse().is_some() {
                incorrect += 1;
            }
        }

        println!("Incorrect: {}/{}", incorrect, n);
    }

    #[test]
    fn sparse_probability() {
        let n = 100;

        let mut incorrect = 0;

        for _ in 0..n {
            if !is_s_sparse() {
                incorrect += 1;
            }
        }

        let probability = incorrect as f32 / n as f32;
        assert!(probability <= 0.01);
    }
}
