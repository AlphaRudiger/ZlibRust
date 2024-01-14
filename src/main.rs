use rustpng::{zlib, inflate::TreeNode, bitreader::BitReader};

#[allow(unused_imports)]

// https://blog.za3k.com/understanding-gzip-2/
fn main() {
    // let mut tree = TreeNode::default();
    // let r1 = tree.start_insert(10, 1);
    // println!("_______");
    // let r2 = tree.start_insert(10, 2);
    // println!("_______");
    // let r3 = tree.start_insert(42, 3);
    // let r4 = tree.start_insert(10, 3);


    // println!("r; {r1:03b}");
    // println!("r; {r2:03b}");
    // println!("r; {r3:03b}");

    // let mut br = BitReader::from_data(vec![0b011]);
    // let r = tree.get_value(&mut br);
    // println!("ikd {r}");
    // // assert!(r == 42);


    // return;

    let r = zlib::decode_zlib_file("/home/rudiger/Desktop/zlibtest1").expect("error");
    print!("result text: ");
    for i in r {
        print!("{}", i as char);
    }

}