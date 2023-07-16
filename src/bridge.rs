use tokio::io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind, Result};
use tokio::net::TcpStream;
use tokio::select;

async fn try_receive<'a>(stream: &mut TcpStream, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
    let count = stream.read(buffer).await?;
    if count == 0 {
        Err(Error::new(ErrorKind::NotConnected, "empty read"))?;
    }
    Ok(&mut buffer[..count])
}

pub async fn bridge(mut stream_a: TcpStream, mut stream_b: TcpStream) -> Result<()> {
    let mut buffer_a = [0_u8; 65536];
    let mut buffer_b = [0_u8; 65536];

    loop {
        select! {
            maybe_buffer = try_receive(&mut stream_a, &mut buffer_a) => {
                stream_b.write_all(maybe_buffer?).await?;
            }
            maybe_buffer = try_receive(&mut stream_b, &mut buffer_b) => {
                stream_a.write_all(maybe_buffer?).await?;
            }
        }
    }
}
