use super::GraphStream;
use rand::Rng;
use std::hash::Hash;

impl<S, T> GraphStream<S>
where
    S: Iterator<Item = T>,
    T: Hash,
{
    fn morris(self) -> i32 {
        let mut x = 0;
        let mut rng = rand::thread_rng();

        self.0.for_each(|_| {
            let prob = (2 as i32).pow(x);

            if rng.gen_range(0..prob) == 0 {
                x += 1
            }
        });

        (2 as i32).pow(x) - 1
    }
}
