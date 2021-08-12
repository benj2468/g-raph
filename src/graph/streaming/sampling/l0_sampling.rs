//! L0 Sampling - Broken b/c of hash functions being from [n(prime)] -> [l(=2^k)]

use crate::graph::streaming::sparse_recovery::one_sparse::{
    OneSparseRecovery, OneSparseRecoveryOutput,
};

use crate::utils::hash_function::{FieldHasher, HashFunction};

/// L-0 Sampling Data Str
pub trait L0Sampling {
    /// Sample a coordinate at random from a high-demensional vector (vector of size n)
    ///
    /// In l-0-sampling we sample each coordinate with probability 1/||f||_0,
    /// meaning uniformly from the set of all distinct coordinates
    fn l_zero_sampling(self, n: u64, delta: f32) -> Option<(u64, i64)>;
}

impl<T> L0Sampling for T
where
    T: core::iter::Iterator<Item = (u64, bool)> + Sized,
{
    fn l_zero_sampling(self, n: u64, delta: f32) -> Option<(u64, i64)> {
        // Initialization
        let mut data_structure = vec![];
        let order = (n as f32).log2() * (1_f32 / delta).log2();
        for l in 0..order.round() as u64 {
            let recover = OneSparseRecovery::init(n);
            let hash_function = FieldHasher::init(n, l);

            data_structure.push((recover, hash_function));
        }

        // Process
        self.for_each(|(j, c)| {
            data_structure.iter_mut().for_each(|(recovery, hasher)| {
                if hasher.is_zero(j) {
                    recovery.feed((j, c))
                }
            })
        });

        // Output
        for (recovery, _) in data_structure {
            let query = recovery.query();
            match query {
                OneSparseRecoveryOutput::VeryLikely(l, i) => return Some((i, l)),
                _ => continue,
            }
        }

        None
    }
}
