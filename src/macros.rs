//! Useful Macros

#[macro_export]
macro_rules! start_dur {
    () => {{
        std::time::Instant::now()
    }};
}

#[macro_export]
macro_rules! printdur {
    ($label:literal, $start_time:ident) => {
        let duration = std::time::Instant::now().duration_since($start_time);
        println!("{}: {:?}", $label, duration);
        let $start_time = start_dur!();
    };
}
