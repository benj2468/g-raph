//! L0 Sampling - Broken b/c of hash functions being from [n(prime)] -> [l(=2^k)]

use algebraics::traits::CeilLog2;

use crate::graph::streaming::sparse_recovery::one_sparse::{
    OneSparseRecovery, OneSparseRecoveryOutput,
};

use crate::utils::hash_function::{HashFunction, PowerFiniteFieldHasher};

#[derive(Clone, Debug)]
pub struct L0Sampler<H>
where
    H: HashFunction,
{
    inner: Vec<(OneSparseRecovery, H)>,
}

impl<H> L0Sampler<H>
where
    H: HashFunction,
{
    pub fn init(n: u64, _delta: f32) -> Self {
        let mut inner = vec![];
        let n_pow = n.next_power_of_two();

        for l in 0..n_pow.ceil_log2().unwrap() as u32 {
            let recover = OneSparseRecovery::init(n_pow);
            let hash_function = H::init(n_pow, 2_u64.pow(l));

            inner.push((recover, hash_function));
        }
        Self { inner }
    }

    pub fn feed(&mut self, token: (u64, bool)) {
        let (j, c) = token;

        self.inner.iter_mut().for_each(|(recovery, hasher)| {
            if hasher.is_zero(j) {
                recovery.feed((j, c))
            }
        })
    }

    pub fn query(self) -> Option<(u64, i64)> {
        for (recovery, _) in self.inner {
            let query = recovery.query();
            match query {
                OneSparseRecoveryOutput::VeryLikely(l, i) => return Some((i, l)),
                _ => continue,
            }
        }

        None
    }
}
