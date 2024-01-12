use anyhow::Result;

use crate::{bitreader::BitReader, zlib_errors::ZlibError, inflate::inflate_block, adler32::adler32};


pub fn decode_zlib_file(p: &str) -> Result<Vec<u8>> {
    let mut br = BitReader::new(p);
    check_zlib_header(&mut br)?;
    let mut out_buffer: Vec<u8> = Vec::new();
    inflate_block(&mut out_buffer, &mut br)?;
    br.discard_bits();
    if br.available() != 4 {
        return Err(ZlibError::ZlibStreamNotConsumed { left: br.available() - 4 }.into());
    }
    // print!("result text: ");
    // for i in &out_buffer {
    //     print!("{}", *i as char);
    // }
    // print!(";");
    // println!();
    adler32(&out_buffer, br.read_u32_msb())?;
    Ok(out_buffer)
}

fn check_zlib_header(br: &mut BitReader) -> Result<()> {
    let cmf = br.read_byte();
    let flg = br.read_byte();
    let check = (((cmf as u16) << 8) | flg as u16) % 31;
    if check != 0 {
        return Err(ZlibError::ZlibCorrupted().into());
    }
    if flg & 0x20 != 0 {
        return Err(ZlibError::UnsupportedZlibAction { action: "preset dictionary" }.into());
    }
    let cm = cmf & 0x0F;
    let cinfo = cmf >> 4;
    if cm != 8 {
        return Err(ZlibError::InvalidZlibHeader { expected: 8, found: cm }.into());
    }
    if cinfo != 7 {
        return Err(ZlibError::InvalidZlibHeader { expected: 7, found: cinfo }.into());
    }
    Ok(())
}
