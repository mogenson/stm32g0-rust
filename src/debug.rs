#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug {
    ($($arg:tt)*) => {
        {
            use cortex_m_semihosting::hio;
            use core::fmt::Write;
            let mut stdout = hio::hstdout().unwrap();
            writeln!(stdout, $($arg)*).ok();
        }
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($($arg:tt)*) => {{}};
}
