use num_bigint::BigUint;
use num_primes::Verification;
use num_traits::FromPrimitive;

#[test]
fn main() {
    // Set BigUint To 7
    let x: BigUint = BigUint::from_u64(7u64).unwrap();

    // Verify Its A Smooth Number
    let result: bool = Verification::is_very_smooth_number(&x, 31.0, 5);

    println!("Is A {} Smooth Number: {}", x, result);
}
