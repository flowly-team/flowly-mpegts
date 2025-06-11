use std::pin::pin;

use flowly::{Service, ServiceExt, io::file::FileReader};
use flowly_flv::FlvDemuxer;
use flowly_mpegts::Mpeg2TsMuxer;
use futures::TryStreamExt;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let mut out = tokio::io::stdout();

    let flow = flowly::flow() //-
        .flow(FileReader::default())
        .flow(FlvDemuxer::default())
        .flow(Mpeg2TsMuxer::default());

    let mut stream = pin!(flow.handle(futures::stream::iter([Ok::<_, std::io::Error>(
        "/home/andrey/demo/h264/test.flv"
    )])));

    while let Some(data) = stream.try_next().await.unwrap() {
        out.write_all(&data).await.unwrap();
    }
}
