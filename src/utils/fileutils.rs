#[cfg(feature = "ftokio")]
use tokio::{
    fs::File,
    io::{self, BufWriter},
};
#[cfg(feature = "ftokio")]
use tokio_util::io::ReaderStream;

#[cfg(feature = "ftokio")]
use crate::wrapper::anyhow::AResult;

#[cfg(feature = "ftokio")]
use futures::{Stream, TryStreamExt};

#[cfg(feature = "ftokio")]
use bytes::Bytes;

use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "ftokio")]
pub async fn stream_to_file_async<S>(stream: S, save_file: &PathBuf) -> AResult<()>
where
    S: Stream<Item = AResult<Bytes>>,
{
    async {
        let body_with_io_error =
            stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
        let body_reader = tokio_util::io::StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        let mut file = BufWriter::new(File::create(save_file).await?);

        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await?;

    Ok(())
}
#[cfg(feature = "ftokio")]
pub async fn file_to_stream_async(filepath: &impl AsRef<Path>) -> AResult<ReaderStream<File>> {
    use crate::log_and_err;

    let file = match tokio::fs::File::open(&filepath).await {
        Ok(file) => file,
        Err(err) => return log_and_err!("Read file error: {}", err),
    };

    Ok(ReaderStream::new(file))
}
