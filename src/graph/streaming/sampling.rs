use rand::Rng;

use crate::graph::streaming::sparse_recovery::OneSparseRecoveryOutput;

use super::*;

impl<S> GraphStream<S>
where
    S: Iterator<Item = (i32, i32)>,
{
    fn l_zero_sampling(self, n: u32) -> Option<(u32, i32)> {
        // Initialization
        let mut data_structure = vec![];
        for _ in 0..(n as f32).ln().round() as u32 {
            let recover = GraphStream::<S>::one_sparse_detection(n);

            data_structure.push(recover);
        }

        // Process
        let mut rng = rand::thread_rng();
        self.0.for_each(|(j, c)| {
            data_structure
                .iter_mut()
                .enumerate()
                .for_each(|(l, recovery)| {
                    if rng.gen_range(0..((2 as u32).pow(l as u32))) == 0 {
                        recovery.feed((j as u32, c))
                    }
                })
        });

        // Output
        for recovery in data_structure {
            match recovery.query() {
                OneSparseRecoveryOutput::VeryLikely(l, i) => return Some((i, l)),
                _ => continue,
            }
        }

        None
    }
}
