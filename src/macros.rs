#[macro_export]
macro_rules! start_dur {
    () => {
        #[cfg(test)]
        let start = Instant::now();
    };
}

#[macro_export]
macro_rules! printdur {
    ($label:literal, $start:ident) => {
        #[cfg(test)]
        {
            let duration = Instant::now().duration_since($start);
            println!("{}: {:?}", $label, duration);
        }
    };
}
