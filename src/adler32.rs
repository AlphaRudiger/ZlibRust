use anyhow::Result;

use crate::zlib_errors::ZlibError;

fn update_adler32(adler: u32, buf: &[u8]) -> u32 {
    const BASE: u32 = 65521;
    let mut s1 = adler & 0xffff;
    let mut s2 = (adler >> 16) & 0xffff;

    for v in buf {
        s1 = (s1 + *v as u32) % BASE;
        s2 = (s2 + s1) % BASE;
    }
    (s2 << 16) + s1
}

pub fn adler32(buf: &[u8], check: u32) -> Result<()> {
    let r = update_adler32(1, buf);
    if r != check {
        return Err(ZlibError::ZlibChecksumMissmatch.into());
    }
    Ok(())
}
