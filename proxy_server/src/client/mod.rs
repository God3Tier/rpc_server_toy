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
        .write_all(format!("{}", req).as_bytes())
        .await
        .is_err()
    {
        return Err("Failed to write to server".into());
    }

    writer.flush().await?;

    let mut response = Vec::new();
    let mut buffer: [u8; 1024] = [0; 1024];

    loop {
        let read_attempt = reader.read(&mut buffer).await;

        match read_attempt {
            Ok(n) => {
                response.extend_from_slice(&buffer[0..n]);
                if n > 0 && buffer[n - 1] != b'\n' {
                	buffer.fill(0); 
                } else {
                	break; 
                }
            }
            Err(e) => {
                return Err(format!("Failed to read from server {e}").into());
            }
        }
    }

    // Optomise this later. What the fu.....
    if response.len() == 3 && response[0] == b'-' && response[1] == b'1' && response[2] == b'\n' {
        return Err("Error response from server".into());
    }

    // What in the actual fuck.....
    Ok(response)
}
