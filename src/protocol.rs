use anyhow::{Result, bail};
use bytes::{BufMut, Bytes, BytesMut};
use iroh::endpoint::{RecvStream, SendStream};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub const ALPN: &[u8] = b"livescreen/1";
pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_MSG: usize = 64 * 1024;
pub const MAX_FRAME: usize = 8 * 1024 * 1024;

pub const CLOSE_NORMAL: u32 = 0;
pub const CLOSE_FULL: u32 = 1;
pub const CLOSE_PROTOCOL: u32 = 2;
pub const CLOSE_KICKED: u32 = 3;
pub const CLOSE_STOPPED: u32 = 4;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ViewerMsg {
    Hello {
        protocol: u32,
        app_version: String,
        #[serde(default)]
        name: String,
    },
    RequestKeyframe,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostMsg {
    Welcome {
        width: u32,
        height: u32,
        fps: u32,
        audio: bool,
        #[serde(default)]
        title: String,
    },
    Rejected {
        reason: String,
    },
    StreamEnded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedFrame {
    pub seq: u64,
    pub keyframe: bool,
    pub pts_us: u64,
    pub data: Bytes,
}

pub fn encode_msg<T: Serialize>(m: &T) -> Result<Vec<u8>> {
    let body = serde_json::to_vec(m)?;
    if body.len() > MAX_MSG {
        bail!("mensagem grande demais");
    }
    let mut v = (body.len() as u32).to_le_bytes().to_vec();
    v.extend(body);
    Ok(v)
}

pub async fn write_msg<T: Serialize>(send: &mut SendStream, m: &T) -> Result<()> {
    send.write_all(&encode_msg(m)?).await?;
    Ok(())
}

pub async fn read_msg<T: DeserializeOwned>(recv: &mut RecvStream) -> Result<T> {
    let mut l = [0u8; 4];
    recv.read_exact(&mut l).await?;
    let len = u32::from_le_bytes(l) as usize;
    if len > MAX_MSG {
        bail!("erro de protocolo: mensagem > 64 KiB");
    }
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await?;
    Ok(serde_json::from_slice(&buf)?)
}

pub fn encode_video_frame(f: &EncodedFrame) -> Bytes {
    let len = 17 + f.data.len();
    let mut b = BytesMut::with_capacity(4 + len);
    b.put_u32_le(len as u32);
    b.put_u8(f.keyframe as u8);
    b.put_u64_le(f.seq);
    b.put_u64_le(f.pts_us);
    b.extend_from_slice(&f.data);
    b.freeze()
}

/// Valida o cabeçalho e devolve o tamanho do corpo (len).
fn check_len(len: usize) -> Result<usize> {
    if len < 17 || len > MAX_FRAME {
        bail!("erro de protocolo: tamanho de quadro inválido ({len})");
    }
    Ok(len)
}

fn frame_from_body(body: Bytes) -> EncodedFrame {
    EncodedFrame {
        keyframe: body[0] & 1 == 1,
        seq: u64::from_le_bytes(body[1..9].try_into().unwrap()),
        pts_us: u64::from_le_bytes(body[9..17].try_into().unwrap()),
        data: body.slice(17..),
    }
}

pub fn parse_video_body(body: &[u8]) -> EncodedFrame {
    frame_from_body(Bytes::copy_from_slice(body))
}

pub async fn read_video_frame(recv: &mut RecvStream) -> Result<Option<EncodedFrame>> {
    let mut l = [0u8; 4];
    match recv.read_exact(&mut l).await {
        Ok(()) => {}
        Err(iroh::endpoint::ReadExactError::FinishedEarly(0)) => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let len = check_len(u32::from_le_bytes(l) as usize)?;
    // BytesMut::zeroed + freeze evita a cópia extra que `vec!` + `copy_from_slice` faria por quadro.
    let mut buf = BytesMut::zeroed(len);
    recv.read_exact(&mut buf).await?;
    Ok(Some(frame_from_body(buf.freeze())))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let f = EncodedFrame {
            seq: 7,
            keyframe: true,
            pts_us: 123456,
            data: Bytes::from_static(&[1, 2, 3]),
        };
        let b = encode_video_frame(&f);
        let len = u32::from_le_bytes(b[..4].try_into().unwrap()) as usize;
        assert_eq!(len, 20);
        assert_eq!(parse_video_body(&b[4..]), f);
    }
    #[test]
    fn limits() {
        assert!(check_len(MAX_FRAME + 1).is_err());
        assert!(check_len(MAX_FRAME).is_ok());
        let big = ViewerMsg::Hello {
            protocol: 1,
            app_version: "x".repeat(MAX_MSG),
            name: String::new(),
        };
        assert!(encode_msg(&big).is_err());
        assert!(encode_msg(&ViewerMsg::RequestKeyframe).is_ok());
    }
    #[test]
    fn hello_without_name() {
        let m = serde_json::from_str::<ViewerMsg>(
            r#"{"type":"hello","protocol":1,"app_version":"0.1.0"}"#,
        )
        .unwrap();
        let ViewerMsg::Hello { name, .. } = m else {
            panic!()
        };
        assert_eq!(name, "");
    }
    #[test]
    fn welcome_without_title() {
        let m = serde_json::from_str::<HostMsg>(
            r#"{"type":"welcome","width":1,"height":2,"fps":30,"audio":false}"#,
        )
        .unwrap();
        let HostMsg::Welcome { title, .. } = m else {
            panic!()
        };
        assert_eq!(title, "");
    }
}
