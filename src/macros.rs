//! Useful Macros

#[macro_export]
macro_rules! start_dur {
    () => {
        // #[cfg(test)]
        let start = Instant::now();
    };
}

#[macro_export]
macro_rules! printdur {
    ($label:literal, $start_time:ident) => {
        // #[cfg(test)]
        let duration = Instant::now().duration_since($start_time);
        // #[cfg(test)]
        println!("{}: {:?}", $label, duration);
    };
}
