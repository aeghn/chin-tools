#[macro_export]
macro_rules! log_and_bail {
    ($fmt:expr $(, $arg:expr)*) => {{
        let msg = format!(concat!(" [{}:{}]", $fmt), file!(), line!(), $($arg)*);
        tracing::error!("{}", msg);
        anyhow::bail!(msg)
    }};
}
