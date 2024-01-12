use anyhow::{Result, Ok};

use crate::{bitreader::BitReader, zlib_errors::{ZlibError, InflateError}};

pub fn inflate_block(out_buffer: &mut Vec<u8>, br: &mut BitReader) -> Result<bool> {
    let last = br.read_bit();
    let btype = br.read_bit() | (br.read_bit() << 1);
    match btype {
        0x00 => inflate_method00(out_buffer, br)?,
        0x01 => inflate_method01(out_buffer, br)?,
        _ => {
            return Err(InflateError::UnsupportedMethod { method: btype }.into());
        }
    }
    Ok(last != 0)
}

fn inflate_method00(out_buffer: &mut Vec<u8>, br: &mut BitReader) -> Result<()> {
    br.discard_bits();
    let len = br.read_u16_lsb();
    let nlen = br.read_u16_lsb();
    if len != !nlen {
        return Err(InflateError::FailedNLenCheck.into());
    }
    let len = len as usize;
    let buf_len = out_buffer.len();
    out_buffer.resize(buf_len + len, 0);
    br.copy_to(&mut out_buffer[buf_len..buf_len + len]);
    Ok(())
}

fn inflate_method01(out_buffer: &mut Vec<u8>, br: &mut BitReader) -> Result<()> {
    loop {
        let mut r = br.read_bits(7);
        let v = if r < 0b0011000 { // lit value 256 - 279
            r + 256
        } else {
            r = r << 1 | br.read_bit() as u16;
            if r < 0b11000000 { // 0 - 143
                r - 0b00110000
            } else if r < 0b11001000 { // 280
                r - 0b11000000 + 280
            } else {
                (r << 1 | br.read_bit() as u16) - 0b110010000 + 144
            }
        };
        if v == 256 {
            break;
        }
        if v < 256 {
            out_buffer.push(v as u8);
        }
    }
    Ok(())
}
