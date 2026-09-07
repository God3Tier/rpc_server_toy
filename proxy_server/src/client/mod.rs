use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

use crate::{Error, request};

/*
 * Please do not forget that this thing has a \n behind every end of response. Forgetting about that will 
 * be abit disasterous icl.....
 */
pub async fn request_from_server<'a>(req: request::Request<'a>) -> Result<Vec<u8>, Error> {
    let server_addr = std::env::var("SERVER_ADDRESS")?;
    let stream = TcpStream::connect(server_addr).await;
    if stream.is_err() {
        return Err(format!("Unable to connect to host server {}", stream.err().unwrap()).into());
    }
    let stream = stream.unwrap();

    let (mut reader, mut writer) = stream.into_split();
    if writer
        .write_all(format!("{}\n", req).as_bytes())
        .await
        .is_err()
    {
        return Err("Failed to write to server".into());
    }

    writer.flush().await?;

    let mut response = Vec::new();

    if reader.read_exact(&mut response).await.is_err() {
        return Err("Failed to read from server".into());
    }

    // Optomise this later. What the fu.....
    if response[0] == b'-'&& response[1] == b'1'  && response[2] == b'\n' {
    	return Err("Error response from server".into())
    }

    // What in the actual fuck.....
    Ok(response)
}
