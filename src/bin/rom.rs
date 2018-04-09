extern crate nesru;
use nesru::rom;

use std::env::args;


fn main() {
    let mut args = std::env::args();
    let fp = args.nth(1).expect("need a file to load");
    println!("Trying to load rom {}", fp);

    let raw = nesru::rom::load(fp);

    let res = rom::parse_ines(&raw);
    println!("{:?}", res);
}
