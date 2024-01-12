use rustpng::{bitreader::{self, BitReader}, zlib};

#[allow(unused_imports)]

fn main() {

    let r = zlib::decode_zlib_file("/home/rudiger/Desktop/zlibtest1").expect("error");
    print!("result text: ");
    for i in r {
        print!("{}", i as char);
    }

}