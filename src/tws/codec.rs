use std::{
    cmp::{max, min},
    fmt::Write,
    io::ErrorKind,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec;
use tracing::{debug, error, info, instrument, trace, warn};

const MAX: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum TWSFrame {
    Incomming {
        data: Bytes,
        splits: Vec<std::ops::Range<usize>>,
    },
    Outgoing(Bytes),
}

pub type DecodedMessage = Vec<Bytes>;

#[derive(Debug)]
pub struct TWSCodec {}

impl TWSCodec {
    pub fn new() -> TWSCodec {
        TWSCodec {}
    }
}

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline(always)]
fn round_up<const N: usize>(n: usize) -> usize {
    (n + (N - 1)) / N * N
}

#[inline(always)]
unsafe fn nonz_index(data: __m128i) -> __m128i {
    let indx_const = 0xFEDCBA9876543210u64;
    let pshufbcnst = _mm_set_epi8(
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0E, 0x0C, 0x0A, 0x08, 0x06, 0x04, 0x02,
        0x00,
    );
    let mut null_mask = _mm_cmpeq_epi8(data, _mm_setzero_si128());
    null_mask = _mm_srli_epi64(null_mask, 4);
    null_mask = _mm_shuffle_epi8(null_mask, pshufbcnst);
    let mask64 = !(_mm_cvtsi128_si64x(null_mask) as u64);

    let indx64 = _pext_u64(indx_const, mask64) as i64;
    let indx = _mm_cvtsi64x_si128(indx64);
    let indx_024 = indx;
    let indx_135 = _mm_srli_epi64(indx, 4);
    let indx_res = _mm_unpacklo_epi8(indx_024, indx_135);

    _mm_and_si128(indx_res, _mm_set1_epi8(0x0F))
}

#[inline(always)]
unsafe fn extract<const N: i32>(input: __m128i, out_ptr: *const u32, i: usize) {
    let high = _mm_bsrli_si128(input, N);
    let shuf_high = _mm_cvtepu8_epi32(high);
    let added = _mm_add_epi32(shuf_high, _mm_set1_epi32(i as i32));
    _mm_storeu_si128(out_ptr.add(N as usize) as *mut __m128i, added);
}

#[inline(always)]
unsafe fn calc_splits_dst(src: &[u8], dst: &mut Vec<u32>) {
    let len = src.len();
    debug_assert!(len < (u32::MAX as usize));

    let estimated_req = len / 4;
    if dst.capacity() < estimated_req {
        dst.reserve(estimated_req - dst.capacity());
    }

    {
        let in_ptr = src.as_ptr();

        let iter_len = round_up::<32>(len);
        for i in (0..iter_len).step_by(32) {
            let vec = _mm_loadu_si128(in_ptr.add(i) as *const __m128i);
            let vec2 = _mm_loadu_si128(in_ptr.add(i + 16) as *const __m128i);

            let zero_eq = _mm_cmpeq_epi8(vec, _mm_setzero_si128());
            let zero_eq2 = _mm_cmpeq_epi8(vec2, _mm_setzero_si128());
            let zero_idx = nonz_index(zero_eq);
            let zero_idx2 = nonz_index(zero_eq2);

            let num_ones = (_mm_movemask_epi8(zero_eq) as u32).count_ones() as usize;
            let num_ones2 = (_mm_movemask_epi8(zero_eq2) as u32).count_ones() as usize;

            if num_ones > 0 {
                let num_nulls = min(num_ones, len - i);
                let rounded = round_up::<4>(num_nulls);

                let out_ptr = dst.as_mut_ptr().add(dst.len());

                extract::<0>(zero_idx, out_ptr, i);
                if rounded >= 4 {
                    extract::<4>(zero_idx, out_ptr, i);
                }
                if rounded >= 8 {
                    extract::<8>(zero_idx, out_ptr, i);
                }
                if rounded >= 12 {
                    extract::<12>(zero_idx, out_ptr, i);
                }

                dst.set_len(dst.len() + num_nulls);
            }

            if num_ones2 > 0 {
                let num_nulls = min(num_ones2, len - (i + 16));
                let rounded = round_up::<4>(num_nulls);

                let out_ptr = dst.as_mut_ptr().add(dst.len());

                extract::<0>(zero_idx2, out_ptr, i + 16);
                if rounded >= 4 {
                    extract::<4>(zero_idx2, out_ptr, i + 16);
                }
                if rounded >= 8 {
                    extract::<8>(zero_idx2, out_ptr, i + 16);
                }
                if rounded >= 12 {
                    extract::<12>(zero_idx2, out_ptr, i + 16);
                }

                dst.set_len(dst.len() + num_nulls);
            }
        }
    }
    if let Some(c) = src.last() {
        if *c != 0 {
            dst.push((len - 1) as u32);
        }
    }
}
fn calc_splits(src: &[u8]) -> Vec<std::ops::Range<usize>> {
    let mut dst = Vec::new();
    unsafe { calc_splits_dst(src, &mut dst) };

    dst.windows(2)
        .map(|o| std::ops::Range {
            start: o[0] as usize,
            end: (o[1] - 1) as usize,
        })
        .collect()
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
        trace!("Recieved data: {:?}", data);
        /* Ok(Some(TWSFrame::Incomming {
            data,
            splits: vec![],
        })) */
        //Ok(Some(data))
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
        trace!("Writing: {:?}", dst);

        /* let data = match &item {
            TWSFrame::Incomming { data, splits } => data,
            TWSFrame::Outgoing(data) => data,
        };
        println!("Writing: {:?}", data);
        dst.extend_from_slice(&data[..]);*/
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use bytes::{BufMut, BytesMut};
    use tokio_util::codec::Decoder;

    use super::TWSCodec;

    const STR: &str = include!("sample.txt");
    const DATA: &[u8] = STR.as_bytes();
    #[test]
    fn sse_wip() {
        let res = super::calc_splits(DATA);
        let mut a;
        let mut b;
        let my_iter: &mut dyn Iterator<Item = usize> = if DATA[DATA.len() - 1] == 0 {
            a = std::iter::empty();
            &mut a
        } else {
            b = std::iter::once(DATA.len());
            &mut b
        };

        let splits = DATA
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match *c {
                0 => Some(i),
                _ => None,
            })
            .chain(my_iter)
            .collect::<Vec<_>>()
            .windows(2)
            .map(|o| std::ops::Range {
                start: o[0],
                end: o[1] - 1,
            })
            .collect::<Vec<_>>();

        assert_eq!(res, splits);
    }

    #[test]
    fn memchr_test() {
        let simple = DATA
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match *c {
                0 => Some(i),
                _ => None,
            })
            .collect::<Vec<_>>();

        let res = memchr::memchr_iter(0, DATA).collect::<Vec<_>>();

        assert_eq!(res, simple);
    }

    /*     #[test]
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
        assert_eq!(c.decode(&mut buf), Ok(Some(Bytes::from(&DATA[..])));
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
    } */

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
