use std::mem;
use std::u16;

use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};

pub struct Frame {
    pub flags: FrameFlags,
    pub masked: bool,
    pub masking_key: Option<[u8; 4]>,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new<P>(opcode: OpCode, payload: P) -> Self
    where
        P: Into<Vec<u8>>,
    {
        Frame {
            flags: FrameFlags(FINAL_MASK | (opcode as u8 & OPCODE_MASK)),
            masked: false,
            masking_key: None,
            payload: payload.into(),
        }
    }

    pub fn with_final(mut self, fin: bool) -> Self {
        self.flags.set_final(fin);
        self
    }

    pub fn with_mask(mut self, key: [u8; 4]) -> Frame {
        self = self.unmask();
        self.masking_key = Some(key);
        self.toggle_mask()
    }

    pub fn frame_size(&self) -> usize {
        let mut size = 0;

        size += 2; // base header

        // length
        if self.payload.len() > u16::MAX as usize {
            size += 8;
        } else if self.payload.len() > 125 {
            size += 2;
        }

        // payload
        size += self.payload.len();

        size
    }

    pub fn mask(self) -> Self {
        if self.masked {
            self
        } else {
            self.toggle_mask()
        }
    }

    pub fn unmask(self) -> Self {
        if self.masked {
            self.toggle_mask()
        } else {
            self
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u8(self.flags.0);

        let mask_flag = if self.masked { 0b1000_0000 } else { 0 };
        if self.payload.len() > u16::MAX as usize {
            dst.put_u8(127 | mask_flag);
            dst.put_u64_be(self.payload.len() as u64);
        } else if self.payload.len() > 125 {
            dst.put_u8(126 | mask_flag);
            dst.put_u16_be(self.payload.len() as u16);
        } else {
            dst.put_u8(self.payload.len() as u8 | mask_flag);
        }

        if let Some(key) = self.masking_key {
            dst.put(&key[..]);
        }

        dst.put(&self.payload);
    }

    pub fn is_complete(data: &BytesMut) -> Option<(FrameFlags, usize)> {
        if data.len() < 2 {
            return None;
        }

        let masked = (data[1] & 0b1000_0000) != 0;
        let mlen = if masked { 4 } else { 0 };
        let (hlen, plen) = match data[1] & 0b0111_1111 {
            len @ 0..=125 => (2, len as usize),
            126 => {
                if data.len() < 4 {
                    return None;
                }
                (4, BigEndian::read_u16(&data[2..4]) as usize)
            }
            127 => {
                if data.len() < 10 {
                    return None;
                }
                (10, BigEndian::read_u64(&data[2..10]) as usize)
            }
            _ => unreachable!(),
        };

        let tlen = hlen + mlen + plen;
        if data.len() < tlen {
            return None;
        }

        Some((FrameFlags(data[0]), tlen))
    }

    pub fn decode(data: &BytesMut) -> Frame {
        let flags = FrameFlags(data[0]);
        let byte_len = data[1] & 0x7F;
        let masked = data[1] & 0x80 != 0;

        let (hlen, len) = match byte_len {
            0..=125 => (2, byte_len as usize),
            126 => (4, BigEndian::read_u16(&data[2..4]) as usize),
            127 => (10, BigEndian::read_u64(&data[2..10]) as usize),
            _ => unreachable!(),
        };

        let mut i = hlen;
        let masking_key = if masked {
            let key = [data[i], data[i + 1], data[i + 2], data[i + 3]];
            i += 4;
            Some(key)
        } else {
            None
        };

        let payload = data[i..i + len].iter().cloned().collect();

        Frame {
            flags,
            masked,
            masking_key,
            payload,
        }
    }

    fn toggle_mask(mut self) -> Frame {
        let key = self.masking_key
            .expect("Cannot mask/unmask a frame without masking_key");

        for (i, b) in self.payload.iter_mut().enumerate() {
            *b = *b ^ key[i % 4];
        }

        self.masked = !self.masked;
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FrameFlags(pub u8);

const FINAL_MASK: u8 = 0b1000_0000;
const RSV1_MASK: u8 = 0b0100_0000;
const RSV2_MASK: u8 = 0b0010_0000;
const RSV3_MASK: u8 = 0b0001_0000;
const OPCODE_MASK: u8 = 0b0000_1111;

impl FrameFlags {
    pub fn is_final(self) -> bool {
        self.0 & FINAL_MASK != 0
    }

    pub fn opcode(self) -> OpCode {
        unsafe { mem::transmute(self.0 & OPCODE_MASK) }
    }

    pub fn rsv1(self) -> bool {
        self.0 & RSV1_MASK != 0
    }

    pub fn rsv2(self) -> bool {
        self.0 & RSV2_MASK != 0
    }

    pub fn rsv3(self) -> bool {
        self.0 & RSV3_MASK != 0
    }

    pub fn set_final(&mut self, value: bool) {
        self.0 &= !FINAL_MASK;
        if value {
            self.0 |= FINAL_MASK;
        }
    }

    pub fn set_opcode(&mut self, value: OpCode) {
        self.0 &= !OPCODE_MASK;
        self.0 |= value as u8 & OPCODE_MASK;
    }

    pub fn set_rsv1(&mut self, value: bool) {
        self.0 &= !RSV1_MASK;
        if value {
            self.0 |= RSV1_MASK;
        }
    }

    pub fn set_rsv2(&mut self, value: bool) {
        self.0 &= !RSV2_MASK;
        if value {
            self.0 |= RSV2_MASK;
        }
    }

    pub fn set_rsv3(&mut self, value: bool) {
        self.0 &= !RSV3_MASK;
        if value {
            self.0 |= RSV3_MASK;
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpCode {
    Continuation = 0,
    Text = 1,
    Binary = 2,
    Close = 8,
    Ping = 9,
    Pong = 10,

    // Reserved non-control frames
    Rsv3 = 3,
    Rsv4 = 4,
    Rsv5 = 5,
    Rsv6 = 6,
    Rsv7 = 7,

    // Reserved control frames
    Rsv11 = 11,
    Rsv12 = 12,
    Rsv13 = 13,
    Rsv14 = 14,
    Rsv15 = 15,
}
