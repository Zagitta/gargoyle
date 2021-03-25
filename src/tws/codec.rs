use std::{
    cmp::{max, min},
    fmt::Write,
    io::ErrorKind,
};

use ascii::AsAsciiStr;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec;

const MAX: usize = 8 * 1024 * 1024;
pub type DecodedMessage = Vec<Bytes>;
pub struct TWSCodec {}

impl TWSCodec {
    pub fn new() -> TWSCodec {
        TWSCodec {}
    }
}

impl codec::Decoder for TWSCodec {
    type Item = DecodedMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length),
            ));
        }

        if src.len() < 4 + length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        //println!("Decoding: {:?}", src);
        src.advance(4); //drop packet size bytes
        let mut data = src.split_to(length).freeze();

        let splits = data
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match *c {
                0 => Some(i),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut v = vec![];
        let mut i = 0usize;
        for split in splits {
            let at = min(max(split - i + 1, 1), data.len() - 1);

            let mut res = data.split_to(at);

            i += res.len();
            match res.last() {
                Some(c) if *c == 0 => res.truncate(res.len() - 1),
                _ => {}
            };
            v.push(res);
        }

        Ok(Some(v))
    }
}

impl codec::Encoder<DecodedMessage> for TWSCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: DecodedMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let sizes = item
            .iter()
            .map(|s| (s.len() as u32) + 1u32)
            .collect::<Vec<_>>();

        let size = sizes.iter().sum::<u32>() - 1u32;
        dst.extend_from_slice(&size.to_be_bytes());
        for s in item.iter().take(item.len() - 1) {
            dst.put(&s[..]);
            dst.put(&b"\0"[..]);
        }
        dst.put(&item.last().unwrap()[..]);
        println!("Writing: {:?}", dst);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use bytes::{BufMut, BytesMut};
    use tokio_util::codec::Decoder;

    use super::TWSCodec;
    #[test]
    fn empty_buffer_decodes_none() {
        let mut c = TWSCodec::new();

        assert_eq!(c.decode(&mut BytesMut::from("")).unwrap(), None);
    }
    #[test]
    fn small_buffer_decodes_none() {
        let mut c = TWSCodec::new();
        let mut buf = BytesMut::new();
        buf.put_i16(123);

        assert_eq!(c.decode(&mut buf).unwrap(), None);
    }

    #[test]
    fn only_msg_len_decodes_none() {
        let mut c = TWSCodec::new();
        let mut buf = BytesMut::new();
        buf.put_i32(24);
        assert_eq!(c.decode(&mut buf).unwrap(), None);
    }
    #[test]
    fn null_decodes_some() {
        let mut c = TWSCodec::new();
        let mut buf = BytesMut::new();
        const DATA: &[u8] = b"\0";
        buf.put_i32(DATA.len() as i32);
        buf.put(&DATA[..]);
        assert_eq!(c.decode(&mut buf).unwrap(), Some(vec!["".into()]));
    }
    #[test]
    fn multiple_nulls_decodes_some() {
        let mut c = TWSCodec::new();
        let mut buf = BytesMut::new();
        const DATA: &[u8] = b"\0\0\0\0";
        buf.put_i32(DATA.len() as i32);
        buf.put(&DATA[..]);
        assert_eq!(
            c.decode(&mut buf).unwrap(),
            Some(vec!["".into(), "".into(), "".into(), "".into(),])
        );
    }
    #[test]
    fn zero_decodes_as_string() {
        let mut c = TWSCodec::new();
        let mut buf = BytesMut::new();
        const DATA: &[u8] = b"0\0";
        buf.put_i32(DATA.len() as i32);
        buf.put(&DATA[..]);
        assert_eq!(c.decode(&mut buf).unwrap(), Some(vec!["0".into()]));
    }
    #[test]
    fn zero_size_msg_decodes() {
        let mut c = TWSCodec::new();
        let mut buf = BytesMut::new();
        const DATA: &[u8] = b"";
        buf.put_i32(DATA.len() as i32);
        buf.put(&DATA[..]);
        assert_eq!(c.decode(&mut buf).unwrap(), Some(vec![]));
    }

    #[test]
    fn initial_server_msg_decodes() {
        let mut c = TWSCodec::new();
        let mut buf = BytesMut::new();
        const DATA: &[u8] = b"\0\0\0\x1a151\020210309 22:54:30 CET\0";
        buf.put(&DATA[..]);
        assert_eq!(
            c.decode(&mut buf).unwrap(),
            Some(vec!["151".into(), "20210309 22:54:30 CET".into()])
        );
    }

    #[test]
    fn empty_bytes_behaves_as_expected() {
        let mut buf = BytesMut::from(&b""[..]);
        let empty = buf.split_to(0);
        assert!(buf.is_empty());
        assert!(empty.is_empty());
    }
    #[test]
    fn empty_bytes_behaves_as_expected2() {
        let mut buf = BytesMut::from(&b"\0"[..]);
        let empty = buf.split_to(0);
        assert_eq!(buf.len(), 1);
        assert!(empty.is_empty());
    }
    #[test]
    fn empty_bytes_behaves_as_expected3() {
        let mut buf = BytesMut::from(&b"\0\0\0\0"[..]);
        assert!(buf.split_to(0).is_empty());
        assert_eq!(buf.len(), 4);
        assert!(buf.split_to(0).is_empty());
        assert_eq!(buf.len(), 4);
        assert!(buf.split_to(0).is_empty());
        assert_eq!(buf.len(), 4);
        assert!(buf.split_to(0).is_empty());
        assert_eq!(buf.len(), 4);
    }
}
