use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct DistinctElements<T> {
    foo: T,
}

fn zeros(p: u64) -> u64 {
    let (mut max, mut curr) = (0, 0);
    while curr <= p / 2 {
        if p % (2 as u64).pow(curr as u32) == 0 {
            max = curr
        }
        curr += 1;
    }

    max
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl<T, S> GraphStream<S>
where
    S: Iterator<Item = T>,
    T: Hash,
{
    fn tidemark(self) -> f32 {
        let mut z = 0;

        self.0.for_each(|token| {
            let hash = calculate_hash(&token);
            let zeros = zeros(hash);
            if zeros < z {
                z = zeros
            }
        });

        (2 as i32).pow(z as u32) as f32 * (2 as f32).sqrt()
    }
}
