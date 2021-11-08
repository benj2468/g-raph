// pub mod ack;
pub mod ack;
mod ack_2;
pub mod bcg;

pub fn compute_s(n: u32) -> f64 {
    const C: f32 = 15.0;
    (C * n as f32) as f64 * (n as f64).log2()
}
