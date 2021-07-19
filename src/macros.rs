#[macro_export]
macro_rules! printdur {
    ($label:literal, $start:ident) => {
        let duration = Instant::now().duration_since($start);
        println!("{}: {:?}", $label, duration);
    };
}
