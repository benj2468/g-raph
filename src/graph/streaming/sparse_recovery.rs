use super::*;

type SizeLimit = u32;

#[derive(Clone)]
pub struct OneSparseRecovery {
    l: i32,
    z: i32,
    p: i32,
    r: u64,
    n: SizeLimit,
}

const bertrand_primes: &[u64] = &[
    2, 3, 5, 7, 13, 23, 43, 83, 163, 317, 631, 1259, 2503, 5003, 9973, 19937, 39869, 79699, 159389,
    318751, 637499, 1274989, 2549951, 5099893, 10199767, 20399531, 40799041, 81598067, 163196129,
    326392249, 652784471, 1305568919, 2611137817, 5222275627,
];

fn find_prime_n_n2(n: u32) -> u64 {
    let mut left = 0;
    let mut right = bertrand_primes.len() as u32;
    loop {
        let insertion_point = ((right - left) / 2) + left;
        let value = *bertrand_primes.get(insertion_point as usize).unwrap();
        if n as u64 > value {
            left = insertion_point + 1
        } else if (n as u64) < value {
            right = insertion_point - 1;
        } else {
            return *bertrand_primes.get(insertion_point as usize + 1).unwrap();
        }

        if right <= left {
            return value;
        }
    }
}

pub enum OneSparseRecoveryOutput {
    Zero,
    VeryLikely(i32, u32),
    NotOneSparse,
}

impl OneSparseRecovery {
    pub fn feed(&mut self, token: (u32, i32)) {
        let (coordinate, value) = token;
        self.l += value;
        self.z += value * coordinate as i32;

        let power = self.r.pow(coordinate) as i32;
        self.p += value * power;
    }

    pub fn query(self) -> OneSparseRecoveryOutput {
        let Self { l, z, p, r, n } = self;
        if l == z && z == p {
            OneSparseRecoveryOutput::Zero
        } else {
            let divided = (z as f64) / (l as f64);
            if divided.round() != divided {
                OneSparseRecoveryOutput::NotOneSparse
            } else if p != (l * r.pow(divided.round() as u32) as i32) {
                // This relies on the premise that z / l will always be within [0,n]
                // I'm not sure if that's a guarantee, but I think it is considering our use of a Field?
                OneSparseRecoveryOutput::NotOneSparse
            } else {
                OneSparseRecoveryOutput::VeryLikely(l, divided.round() as u32)
            }
        }
    }
}

impl<S> GraphStream<S>
where
    S: Iterator<Item = (i32, i32)>,
{
    pub fn one_sparse_detection(n: u32) -> OneSparseRecovery {
        let r = find_prime_n_n2(n.pow(2));
        let (mut l, mut z, mut p) = (0, 0, 0);

        OneSparseRecovery { l, z, p, r, n }
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
}
