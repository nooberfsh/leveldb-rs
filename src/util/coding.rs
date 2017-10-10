use std::mem;
use std::ptr;

#[cfg(target_endian = "little")]
const IS_LITTLE_ENDIAN: bool = true;

#[cfg(target_endian = "big")]
const IS_LITTLE_ENDIAN: bool = false;

macro_rules! encode_fixed {
    ($T: ty, $buf: expr, $value: expr) => {
        if IS_LITTLE_ENDIAN {
            let pbuf = mem::transmute($buf);
            ptr::copy_nonoverlapping(&$value, pbuf, mem::size_of::<$T>());
        } else {
            let mut p = $buf;
            for _ in 0..mem::size_of::<$T>() {
                *p = $value as u8;
                $value >>= 8;
                p = p.offset(1);
            }
        }
    };
}

macro_rules! decode_fixed {
    ($T: ty, $buf: expr) => {
        {
            let mut result: $T = mem::uninitialized();
            if IS_LITTLE_ENDIAN {
                let psrc = $buf.as_ptr() as *const $T;
                ptr::copy_nonoverlapping(psrc, &mut result, mem::size_of::<$T>());
            } else {
                for i in 0..mem::size_of::<$T>() {
                    result |= $buf[i] as $T << 8*i
                }
            }
            result
        }
    };
}

macro_rules! encode_var {
    ($T: ty, $buf: expr, $value: expr) => {
        {
            static B: $T= 128;
            let mut p = $buf;
            while $value >= B {
                *p = ($value | B) as u8;
                $value >>= 7;
                p = p.offset(1);
            }
            *p = $value as u8;
            p.offset_to($buf).unwrap() as usize + 1
        }
    };
}

macro_rules! get_varint {
    ($T: ty, $input: expr, $max_index: expr) => {
        {
            let mut result: $T = 0;
            for (i, byte) in $input.iter().enumerate() {
                if i >= $max_index {
                    return None;
                }
                if (byte & 128) !=0 {
                    result |= ((byte & 127) << 7*i) as $T;
                } else {
                    result |= (byte << 7*i) as $T;
                    return Some((&$input[i+1..], result))
                }
            }
            None
        }
    };
}

macro_rules! varint_length {
    ($value: expr) => {
        {
            let mut result = 1;
            while $value >= 128 {
                result += 1;
                $value >>= 7;
            }
            result
        }
    };
}

pub fn put_fixed32(dst: &mut Vec<u8>, value: u32) {
    unsafe {
        let mut buf: [u8; 4] = mem::uninitialized();
        encode_fixed32(buf.as_mut_ptr(), value);
        dst.extend_from_slice(&buf);
    }
}

pub fn put_fixed64(dst: &mut Vec<u8>, value: u64) {
    unsafe {
        let mut buf: [u8; 8] = mem::uninitialized();
        encode_fixed64(buf.as_mut_ptr(), value);
        dst.extend_from_slice(&buf);
    }
}

pub fn put_varint32(dst: &mut Vec<u8>, value: u32) {
    unsafe {
        let mut buf: [u8; 5] = mem::uninitialized();
        let length = encode_varint32(buf.as_mut_ptr(), value);
        dst.extend_from_slice(&buf[0..length]);
    }
}

pub fn put_varint64(dst: &mut Vec<u8>, value: u64) {
    unsafe {
        let mut buf: [u8; 10] = mem::uninitialized();
        let length = encode_varint64(buf.as_mut_ptr(), value);
        dst.extend_from_slice(&buf[0..length]);
    }
}

pub fn put_length_prefixed_slice(dst: &mut Vec<u8>, value: &[u8]) {
    put_varint32(dst, value.len() as u32);
    dst.extend_from_slice(&value[0..value.len()]);
}


pub fn get_varint32(input: &[u8]) -> Option<(&[u8], u32)> {
    get_varint!(u32, input, 5)
}

pub fn get_varint64(input: &[u8]) -> Option<(&[u8], u64)> {
    get_varint!(u64, input, 10)
}

pub fn get_length_prefixed_slice(input: &[u8]) -> Option<(&[u8], &[u8])> {
    get_varint32(input).and_then(|(remain, len)| {
        let len = len as usize;
        if remain.len() >= len {
            Some((&remain[len..], &remain[..len]))
        } else {
            None
        }
    })
}

pub fn varint32_length(mut value: u32) -> usize {
    varint_length!(value)
}

pub fn varint64_length(mut value: u64) -> usize {
    varint_length!(value)
}

pub unsafe fn encode_fixed32(buf: *mut u8, mut value: u32) {
    encode_fixed!(u32, buf, value);
}

pub unsafe fn encode_fixed64(buf: *mut u8, mut value: u64) {
    encode_fixed!(u64, buf, value);
}

pub unsafe fn encode_varint32(buf: *mut u8, mut value: u32) -> usize {
    encode_var!(u32, buf, value)
}

pub unsafe fn encode_varint64(buf: *mut u8, mut value: u64) -> usize {
    encode_var!(u64, buf, value)
}

#[inline]
pub unsafe fn decode_fixed32(input: &[u8]) -> u32 {
    decode_fixed!(u32, input)
}

#[inline]
pub unsafe fn decode_fixed64(input: &[u8]) -> u64 {
    decode_fixed!(u64, input)
}
