pub type AResult<T> = anyhow::Result<T>;
pub type EResult = anyhow::Result<()>;

pub use anyhow::anyhow as aanyhow;
pub use anyhow::Context;

#[macro_export]
macro_rules! eanyhow {
    ($ah:expr) => {
        Err(anyhow::anyhow!($ah))
    };
}
