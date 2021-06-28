use rand::Rng;

pub trait Coutning {
    fn morris(self) -> i32;
}

impl<T> Coutning for T
where
    T: core::iter::Iterator<Item = (i32, i32)> + Sized,
{
    fn morris(self) -> i32 {
        let mut x = 0;
        let mut rng = rand::thread_rng();

        self.for_each(|_| {
            let prob = (2 as i32).pow(x);

            if rng.gen_range(0..prob) == 0 {
                x += 1
            }
        });

        (2 as i32).pow(x) - 1
    }
}
