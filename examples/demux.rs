use std::{path::PathBuf, pin::pin, str::FromStr};

use bytes::Bytes;
use flowly::{Frame, Service, ServiceExt, flow, io::file::FileReader};
use futures::TryStreamExt;
use tokio::io::AsyncReadExt;

pub struct FileReader1;

impl<E: std::error::Error + Send + Sync + 'static> Service<Result<PathBuf, E>> for FileReader1 {
    type Out = std::io::Result<Bytes>;

    fn handle(
        self,
        input: impl futures::Stream<Item = Result<PathBuf, E>> + Send,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::try_stream! {
            let mut input = pin!(input);
            let mut buf = vec![0u8; 188 * 1024];

            while let Some(path) = input.try_next().await.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, Box::new(err)))?  {
                let mut file = tokio::fs::File::open(path).await?;
                loop {
                    yield match file.read(&mut buf[..]).await? {
                        0 => break,
                        n => buf[0..n].to_vec().into()
                    };
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // let reader = flow().flow(FileReader1).flow(Mpeg2TsDemux::new());

    // let stream = reader.handle(futures::stream::once(async move {
    //     PathBuf::from_str("/home/andrey/demo/h264/oxford_shoppers.ts")
    //     // PathBuf::from_str("/home/andrey/demo/h265/street.ts")
    // }));

    // let mut stream = pin!(stream);

    // while let Some(frame) = stream.try_next().await? {
    //     println!();

    //     for unit in frame.units() {
    //         println!(
    //             "{:0.2}\t{}\t {} {} {}",
    //             (frame.pts() as f64) / 1_000_000.0,
    //             frame.has_params(),
    //             frame.params().count(),
    //             frame.is_keyframe(),
    //             unit.len()
    //         );
    //     }
    // }

    Ok(())
}
