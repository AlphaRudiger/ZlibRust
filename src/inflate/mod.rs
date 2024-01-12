use anyhow::{Result, Ok};

use crate::{bitreader::BitReader, zlib_errors::{InflateError, ZlibError}, inflate::lookup::{DIST_TABLE, HCLEN_TABLE}};

use self::lookup::LEN_TABLE;

mod lookup;

pub fn inflate_block(out_buffer: &mut Vec<u8>, br: &mut BitReader) -> Result<bool> {
    let last = br.read_bit();
    let btype = br.read_bits_msb(2);
    match btype {
        0b00 => inflate_method00(out_buffer, br)?,
        0b01 => inflate_method01(out_buffer, br)?,
        0b10 => inflate_method10(out_buffer, br)?,
        _ => {
            return Err(InflateError::UnsupportedMethod { method: btype as u8 }.into());
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

pub enum TreeNode {
    Branch(Option<Box<TreeNode>>, Option<Box<TreeNode>>, bool),
    Leaf(u16)
}

impl Default for TreeNode {
    fn default() -> Self {
        TreeNode::Branch(None, None, false)
    }
}

impl TreeNode {

    fn test(dest: &mut Option<Box<TreeNode>>, value: u16, len: u8) -> (u32, bool) {
        if dest.is_none() {
            *dest = Some(Box::default());
        }
        if let Some(t) = dest {
            return t.insert(value, len - 1);
        }
        unreachable!();
    }

    pub fn start_insert(&mut self, value: u16, len: u8) -> u32 {
        let r = self.insert(value, len);
        debug_assert!(r.1);
        r.0
    }

    pub fn get_value(&self, br: &mut BitReader) -> u16 {
        match self {
            TreeNode::Branch(left, right, _) => {
                let branch = if br.read_bit() == 0 {
                    left
                } else {
                    right
                };
                if let Some(v) = branch {
                    return v.get_value(br);
                }
            }
            TreeNode::Leaf(value) => {
                return *value;
            }
            _ => {
                debug_assert!(false);
            }
        }
        0
    }
    
    fn insert(&mut self, value: u16, len: u8) -> (u32, bool) {
        println!("inserting");
        if let TreeNode::Branch(left, right, _) = self {
            if len == 1 {
                return if left.is_none() {
                    println!("insert left");
                    *left = Some(Box::new(TreeNode::Leaf(value)));
                    (0, true)
                } else if right.is_none() {
                    println!("insert right");
                    *right = Some(Box::new(TreeNode::Leaf(value)));
                    (1, true)
                } else {
                    (0, false)
                }
            } else {
                let mut r = Self::test(left, value, len);
                if !r.1 {
                    r = Self::test(right, value, len);
                    r.0 = (r.0 << 1) | 1;
                } else {
                    r.0 <<= 1;
                }
                return r;
            }
        } else {
            // debug_assert!(false, "invalid insert");
        }
        (0, false)
    }
}

fn inflate_method10(out_buffer: &mut Vec<u8>, br: &mut BitReader) -> Result<()> {
    let hlit = br.read_bits_msb(5) + 257;
    let hdist = br.read_bits_msb(5) + 1;
    let hclen = br.read_bits_msb(4) + 4;

    println!("hlit {hclen}");

    let mut hc_table: [(u16, u8); 19] = Default::default();

    for i in 0..hclen {
        let code = HCLEN_TABLE[i as usize];
        hc_table[code as usize] = (code as u16, br.read_bits_msb(3) as u8);
    }


    println!("{hc_table:?}");

    Ok(())
}

fn inflate_method01(out_buffer: &mut Vec<u8>, br: &mut BitReader) -> Result<()> {
    loop {
        let mut r = br.read_bits(7);
        let v = if r < 0b0011000 { // lit value 256 - 279
            if r == 0 {
                break;
            }
            r + 256
        } else {
            r = r << 1 | br.read_bit() as u16;
            if r < 0b11000000 { // 0 - 143
                r - 0b00110000
            } else if r < 0b11001000 { // 280 - 287
                r - 0b11000000 + 280
            } else { // 144 - 255
                (r << 1 | br.read_bit() as u16) - 0b110010000 + 144
            }
        };
        if v < 256 {
            out_buffer.push(v as u8);
            // println!("push {}", v as u8 as char);
            continue;
        }
        let (len, extra) = LEN_TABLE[v as usize - 257];
        let len = (len + br.read_bits_msb(extra)) as usize;
        let (dist, de) = DIST_TABLE[br.read_bits(5) as usize];
        let dist = (dist + br.read_bits_msb(de)) as usize;
        // out_buffer.resize(buflen + len, b'A');
        // println!("len: {len} {extra} dist {dist} {de}");
        let buflen = out_buffer.len();
        for i in (buflen - dist)..(buflen - dist + len) {
            // println!("push {}", out_buffer[i] as char);
            out_buffer.push(out_buffer[i]);
        }
    }
    Ok(())
}
