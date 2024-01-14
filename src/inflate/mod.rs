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


    pub fn new(arr: &mut [(u16, u8)]) -> Self {
        let mut tree = Self::default();
        arr.sort_by(|a, b| { a.1.cmp(&b.1) });
        // let mut test: Vec<(u16, u8, u32)> = Vec::new();

        for (value, len) in arr {
            // println!("insert: {value}, {len}");
            tree.start_insert(*value, *len);
            // test.push((value, len, c));
        }

        // for (value, len, c) in test {
        //     if len > 0 {
        //         let mut tr = BitReader::from_data(vec![c as u8]);
        //         debug_assert!(tree_construction_tree.get_value(&mut tr) == value, "big uff");
        //     }
        // }
        tree
    }

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
        if len == 0 {
            return 0;
        }
        let r = self.insert(value, len);
        debug_assert!(r.1, "unable to insert (len: {len}, value: {value})");
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
        debug_assert!(false);
        0
    }
    
    fn insert(&mut self, value: u16, len: u8) -> (u32, bool) {
        // println!("inserting");
        if let TreeNode::Branch(left, right, _) = self {
            if len == 1 {
                return if left.is_none() {
                    // println!("insert left");
                    *left = Some(Box::new(TreeNode::Leaf(value)));
                    (0, true)
                } else if right.is_none() {
                    // println!("insert right");
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
        }
        (0, false)
    }
}

fn build_tree_from_tree(tree_construction_tree: &TreeNode, br: &mut BitReader, to: u16) -> TreeNode {
    let mut i = 0;
    let mut last_len = 0;
    let mut out: Vec<(u16, u8)> = Vec::new();

    while i < to as usize {
        let val = tree_construction_tree.get_value(br) as u8;
        match val {
            0..=15 => {
                out.push((i as u16, val));
                last_len = val;
                i += 1;
            }
            16 => {
                let copy = br.read_bits_msb(2) + 3;
                for j in 0..copy {
                    out.push((i as u16 + j, last_len));
                }
                i += copy as usize;
            }
            17 => {
                let skip = br.read_bits_msb(3) + 3;
                i += skip as usize;
            }
            18 => {
                let skip = br.read_bits_msb(7) + 11;
                i += skip as usize;
            }
            _ => {
                debug_assert!(false, "err");
            }
        }
    }

    TreeNode::new(&mut out)
}

fn inflate_method10(out_buffer: &mut Vec<u8>, br: &mut BitReader) -> Result<()> {
    let hlit = br.read_bits_msb(5) + 257;
    let hdist = br.read_bits_msb(5) + 1;
    let hclen = br.read_bits_msb(4) + 4;

    let mut hc_table: [(u16, u8); 19] = Default::default();

    for i in 0..hclen {
        let code = HCLEN_TABLE[i as usize];
        let val = br.read_bits_msb(3) as u8;
        hc_table[code as usize] = (code as u16, val);
        debug_assert!(val < 19);
    }

    let tree_construction_tree = TreeNode::new(&mut hc_table);

    let normal_tree = build_tree_from_tree(&tree_construction_tree, br, hlit);
    let dist_tree = build_tree_from_tree(&tree_construction_tree, br, hdist);

    loop {
        let v = normal_tree.get_value(br);
        
        if v < 257 {
            if v == 256 {
                break;
            }
            out_buffer.push(v as u8);
            continue;
        }
        let (len, extra) = LEN_TABLE[v as usize - 257];
        let len = (len + br.read_bits_msb(extra)) as usize;
        let (dist, de) = DIST_TABLE[dist_tree.get_value(br) as usize];
        let dist = (dist + br.read_bits_msb(de)) as usize;
        let buflen = out_buffer.len();
        for i in (buflen - dist)..(buflen - dist + len) {
            out_buffer.push(out_buffer[i]);
        }
    }

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
            continue;
        }
        let (len, extra) = LEN_TABLE[v as usize - 257];
        let len = (len + br.read_bits_msb(extra)) as usize;
        let (dist, de) = DIST_TABLE[br.read_bits(5) as usize];
        let dist = (dist + br.read_bits_msb(de)) as usize;
        let buflen = out_buffer.len();
        for i in (buflen - dist)..(buflen - dist + len) {
            out_buffer.push(out_buffer[i]);
        }
    }
    Ok(())
}
