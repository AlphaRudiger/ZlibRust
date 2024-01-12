use std::arch::asm;


pub struct BitReader {
    data: Vec<u8>,
    index: usize,
    bit_counter: i32,
    total: usize,
}

impl BitReader {
    pub fn new(path: &str) -> Self {
        Self::from_data(std::fs::read(path).expect("file not found"))
    }
    pub fn from_data(data: Vec<u8>) -> Self {
        BitReader { data, index: 0, bit_counter: 0, total: 0 }
    }
    pub fn read_u32_msb(&mut self) -> u32 {
        (self.read_byte() as u32) << 24 | (self.read_byte() as u32) << 16 | (self.read_byte() as u32) << 8 | (self.read_byte() as u32)
    }
    pub fn available(&mut self) -> usize {
        self.data.len() - self.index
    }
    pub fn read_bits(&mut self, amount: u8) -> u16 {
        let mut r = 0;
        for i in (0..amount).rev() {
            r |= (self.read_bit() as u16) << i;
        }
        r
    }
    pub fn read_byte(&mut self) -> u8 {
        let r = unsafe {
            // println!("{}", self.index);
            *self.data.get_unchecked(self.index)
        };
        self.index += 1;
        r
    }
    pub fn read_bit(&mut self) -> u8 {
        let mut r = unsafe {
            // println!("{}", self.index);
            *self.data.get_unchecked(self.index)
        };
        r = (r >> self.bit_counter) & 1;
        self.bit_counter += 1;
        if self.bit_counter == 8 {
            self.bit_counter = 0;
            self.index += 1;
        }
        r 
    }
    pub fn read_u16_lsb(&mut self) -> u16 {
        self.read_byte() as u16 | ((self.read_byte() as u16) << 8)
    }
    pub fn read_bit_asm(&mut self) -> u8 {
        let mut r = unsafe {
            // println!("{}", self.index);
            *self.data.get_unchecked(self.index)
        };
        r = (r >> self.bit_counter) & 1;
        self.bit_counter += 1;
        unsafe {
            asm!(
                "cmp {x:e}, 8",
                "mov {z:e}, 0",
                "lea {n:r}, [{i:r} + 1]",
                "cmovz {x:e}, {z:e}",
                "cmovz {i:r}, {n:r}",


                x = inout(reg) self.bit_counter,
                i = inout(reg) self.index,
                z = out(reg) _,
                n = out(reg) _
            );
        }
        r 
    }

    pub fn read_bit_better(&mut self) -> u8 {
        let mut r = unsafe {
            *self.data.get_unchecked(self.index)
        };
        r = (r >> self.bit_counter) & 1;
        self.bit_counter += 1;
        self.index += (self.bit_counter >> 3) as usize;
        self.bit_counter &= 0x7;
        r
    }
    pub fn read_bit_cursed(&mut self) -> u8 {
        let mut r = unsafe {
            *self.data.get_unchecked(self.total / 8)
        };
        r = (r >> (self.total % 8)) & 1;
        self.total += 1;
        r
    }
    pub fn discard_bits(&mut self) {
        self.index += (self.bit_counter > 0) as usize;
        self.bit_counter = 0;
    }
    pub fn copy_to(&mut self, dest: &mut [u8]) {
        dest.copy_from_slice(&self.data[self.index..self.index + dest.len()]);
        self.index += dest.len();
    }
}

#[test]
fn test_reader() {
    let data = vec![1,6];
    let mut br1 = BitReader::from_data(data.clone());
    let mut br2 = BitReader::from_data(data.clone());
    let mut br3 = BitReader::from_data(data.clone());
    let mut br4 = BitReader::from_data(data.clone());

    for _ in 0..data.len() * 8 {
        let r1 = br1.read_bit();
        let r2 = br2.read_bit_better();
        let r3 = br3.read_bit_cursed();
        let r4 = br4.read_bit_cursed();
        println!("[{}]{}, [{}]{}", r1, br1.index, r2, br2.index);
        assert!(r1 == r2 && r2 == r3 && r3 == r4);
    }
}