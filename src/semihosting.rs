#[macro_export]
#[cfg(feature = "semihosting")]
macro_rules! println {
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
#[cfg(not(feature = "semihosting"))]
macro_rules! println {
    ($($arg:tt)*) => {{}};
}
