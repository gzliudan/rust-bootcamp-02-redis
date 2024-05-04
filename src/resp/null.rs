use bytes::BytesMut;

use super::{extract_fixed_data, RespDecoder, RespEncoder, RespError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

// - null: "_\r\n"
impl RespEncoder for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespDecoder for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", "Null")?;
        Ok(RespNull)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;
    use anyhow::Result;

    #[test]
    fn test_encode_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }

    #[test]
    fn test_null_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"_\r\n");

        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);

        Ok(())
    }
}
