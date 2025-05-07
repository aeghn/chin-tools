pub type AResult<T> = anyhow::Result<T>;
pub type EResult = anyhow::Result<()>;

pub use anyhow::Context;
pub use anyhow::anyhow as aanyhow;

#[macro_export]
macro_rules! eanyhow {
    ($ah:expr) => {
        Err(chin_tools::anyhow::aanyhow!($ah))
    };
}
