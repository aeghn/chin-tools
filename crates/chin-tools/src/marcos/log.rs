#[macro_export]
macro_rules! log_and_err {
    ($fmt:expr $(, $arg:expr)*) => {{
        let msg = format!(concat!(" [{}:{}]", $fmt), file!(), line!(), $($arg)*);
        log::error!("{}", msg);
        Err(anyhow::anyhow!(msg))
    }};
}
