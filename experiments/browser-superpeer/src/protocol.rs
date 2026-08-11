//! Simple LBRY-blob-over-Iroh request/response on a bidirectional QUIC stream.

use anyhow::{anyhow, bail, Context, Result};
use iroh::endpoint::{Connection, RecvStream, SendStream};
use tokio::io::AsyncReadExt;

pub const ALPN: &[u8] = b"lbry-blob-iroh/1";

const CMD_HAVE: u8 = 1;
const CMD_GET: u8 = 2;

const ST_OK: u32 = 0;
const ST_NOT_FOUND: u32 = 1;
const ST_BAD: u32 = 2;

async fn write_u32(send: &mut SendStream, v: u32) -> Result<()> {
    send.write_all(&v.to_be_bytes())
        .await
        .map_err(|e| anyhow!("write u32: {e}"))
}

async fn read_u32(recv: &mut RecvStream) -> Result<u32> {
    let mut buf = [0u8; 4];
    recv.read_exact(&mut buf)
        .await
        .map_err(|e| anyhow!("read u32: {e}"))?;
    Ok(u32::from_be_bytes(buf))
}

async fn write_u64(send: &mut SendStream, v: u64) -> Result<()> {
    send.write_all(&v.to_be_bytes())
        .await
        .map_err(|e| anyhow!("write u64: {e}"))
}

async fn read_u64(recv: &mut RecvStream) -> Result<u64> {
    let mut buf = [0u8; 8];
    recv.read_exact(&mut buf)
        .await
        .map_err(|e| anyhow!("read u64: {e}"))?;
    Ok(u64::from_be_bytes(buf))
}

async fn write_hash(send: &mut SendStream, hash_hex: &str) -> Result<()> {
    let h = hash_hex.to_lowercase();
    if h.len() > 255 {
        bail!("hash hex too long");
    }
    send.write_all(&[h.len() as u8])
        .await
        .map_err(|e| anyhow!("write hash len: {e}"))?;
    send.write_all(h.as_bytes())
        .await
        .map_err(|e| anyhow!("write hash: {e}"))
}

async fn read_hash(recv: &mut RecvStream) -> Result<String> {
    let mut len_buf = [0u8; 1];
    recv.read_exact(&mut len_buf)
        .await
        .map_err(|e| anyhow!("read hash len: {e}"))?;
    let len = len_buf[0] as usize;
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf)
        .await
        .map_err(|e| anyhow!("read hash: {e}"))?;
    String::from_utf8(buf).context("hash utf8")
}

/// Client: ask whether the peer has a blob.
pub async fn client_have(conn: &Connection, hash_hex: &str) -> Result<bool> {
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| anyhow!("open_bi: {e}"))?;
    send.write_all(&[CMD_HAVE])
        .await
        .map_err(|e| anyhow!("write cmd: {e}"))?;
    write_hash(&mut send, hash_hex).await?;
    send.finish().map_err(|e| anyhow!("finish: {e}"))?;
    let mut ans = [0u8; 1];
    AsyncReadExt::read_exact(&mut recv, &mut ans)
        .await
        .map_err(|e| anyhow!("read have: {e}"))?;
    Ok(ans[0] == 1)
}

/// Client: download a blob by LBRY hash hex.
pub async fn client_get_blob(conn: &Connection, hash_hex: &str) -> Result<Vec<u8>> {
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| anyhow!("open_bi: {e}"))?;
    send.write_all(&[CMD_GET])
        .await
        .map_err(|e| anyhow!("write cmd: {e}"))?;
    write_hash(&mut send, hash_hex).await?;
    send.finish().map_err(|e| anyhow!("finish: {e}"))?;

    let status = read_u32(&mut recv).await?;
    match status {
        ST_OK => {
            let len = read_u64(&mut recv).await? as usize;
            // Cap at slightly over 2MiB encrypted + padding.
            if len > 3 * 1024 * 1024 {
                bail!("blob too large: {len}");
            }
            let mut buf = vec![0u8; len];
            if len > 0 {
                recv.read_exact(&mut buf)
                    .await
                    .map_err(|e| anyhow!("read body: {e}"))?;
            }
            Ok(buf)
        }
        ST_NOT_FOUND => bail!("blob not found on superpeer: {hash_hex}"),
        other => bail!("superpeer error status {other} for {hash_hex}"),
    }
}

/// Server side: handle one bi stream request.
pub async fn serve_one(
    send: &mut SendStream,
    recv: &mut RecvStream,
    mut load: impl FnMut(&str) -> Result<Option<Vec<u8>>>,
) -> Result<()> {
    let mut cmd = [0u8; 1];
    recv.read_exact(&mut cmd)
        .await
        .map_err(|e| anyhow!("read cmd: {e}"))?;
    let hash = read_hash(recv).await?;

    match cmd[0] {
        CMD_HAVE => {
            let have = load(&hash)?.is_some();
            send.write_all(&[if have { 1 } else { 0 }])
                .await
                .map_err(|e| anyhow!("write have: {e}"))?;
        }
        CMD_GET => match load(&hash)? {
            Some(data) => {
                write_u32(send, ST_OK).await?;
                write_u64(send, data.len() as u64).await?;
                if !data.is_empty() {
                    send.write_all(&data)
                        .await
                        .map_err(|e| anyhow!("write body: {e}"))?;
                }
            }
            None => {
                write_u32(send, ST_NOT_FOUND).await?;
            }
        },
        other => {
            write_u32(send, ST_BAD).await?;
            bail!("unknown cmd {other}");
        }
    }
    send.finish().map_err(|e| anyhow!("finish: {e}"))?;
    Ok(())
}
