use bytes::{Buf, BytesMut};
use std::ops::Deref;

use super::{parse_length, RespDecoder, RespEncoder, RespError, CRLF_LEN};

const NULL_BULK_STRING: &[u8] = b"$-1\r\n";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
// - null bulk string: "$-1\r\n"
impl RespEncoder for BulkString {
    fn encode(self) -> Vec<u8> {
        if self.is_empty() {
            return NULL_BULK_STRING.to_vec();
        }

        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self.0);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespDecoder for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.starts_with(NULL_BULK_STRING) {
            buf.advance(NULL_BULK_STRING.len());
            return Ok(BulkString::new(vec![]));
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        if len > buf.len() {
            return Err(RespError::NotComplete);
        }
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl From<String> for BulkString {
    fn from(s: String) -> Self {
        BulkString(s.into_bytes())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;
    use anyhow::Result;

    #[test]
    fn test_encode_null_bulk_string() {
        let frame: RespFrame = b"".into();
        assert_eq!(frame.encode(), NULL_BULK_STRING.to_vec());
    }

    #[test]
    fn test_decode_null_bulk_string() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(NULL_BULK_STRING);

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString(vec![]));

        Ok(())
    }

    #[test]
    fn test_encode_bulk_string() {
        let frame: RespFrame = BulkString::new("hello".to_string()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n".to_vec());
    }

    #[test]
    fn test_decode_bulk_string() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        Ok(())
    }
}
