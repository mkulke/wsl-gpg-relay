use atoi::atoi;
use std::error::Error;
use std::io::Read;
use std::{env, path::PathBuf};
use std::{fs, process};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let localappdata = env::var("LOCALAPPDATA")?;
    let debug = match env::args().nth(1) {
        Some(arg) if arg == "--debug" => true,
        _ => false,
    };
    let path: PathBuf = [&localappdata, "gnupg", "S.gpg-agent"].iter().collect();
    if debug {
        eprintln!("reading socket info from {:?}", path);
    }
    let mut file = fs::File::open(&path)?;
    let mut bytes = Vec::new();
    let _ = file.read_to_end(&mut bytes)?;
    let split_point = bytes.partition_point(|&b| b == b'\n');
    let port_bytes = &bytes[..split_point];
    let port: u16 = atoi(port_bytes).ok_or("no port found")?;
    let nonce = &bytes[split_point..];
    if nonce.len() != 16 {
        return Err(format!("invalid nonce length: {}", nonce.len()).into());
    }

    if debug {
        eprintln!("connecting to 127.0.0.1:{}", port);
    }
    let mut conn = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    if debug {
        eprintln!("write nonce");
    }
    conn.write_all(nonce).await?;
    if debug {
        eprintln!("connected");
    }

    let (mut reader, mut writer) = conn.into_split();
    tokio::spawn(async move {
        if let Err(e) = io::copy(&mut io::stdin(), &mut writer).await {
            eprintln!("failed to copy from stdin: {}", e);
            process::exit(1);
        }
    });

    tokio::io::copy(&mut reader, &mut io::stdout())
        .await
        .inspect_err(|e| eprintln!("failed to copy from conn: {}", e))?;
    if debug {
        eprintln!("connection closed");
    }
    Ok(())
}
