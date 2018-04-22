use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;

/// Limit the amount of data to read. In case a user accidentally wants to read
/// a very large file as a ROM.
const MAX_ROM_SIZE: u64 = 5 * 1024 * 1024;

pub enum MirroringType {
    Nothing,
}

/// Structure representing a rom/cartridge of the NES system.
pub struct Rom {
    prg: Vec<u8>,
    chr: Vec<u8>,
    mirroring_type: MirroringType,
}

pub struct InesHeader {
    prg_banks: usize,
    chr_banks: usize,
    control_1: u8,
    control_2: u8,
}

/// Create a Rom from bytes in the Ines format
fn FromInes(b: Vec<u8>) -> Result<Rom, ParseError> {
    Ok(Rom {prg: Vec::new(), chr: Vec::new(), mirroring_type: MirroringType::Nothing})
}

pub fn load<P>(fp: P) -> Vec<u8>
where
    P: AsRef<Path>,
{
    let f = File::open(fp).unwrap();
    let mut bytes: Vec<u8> = Vec::new();

    let file_size = f.take(MAX_ROM_SIZE).read_to_end(&mut bytes).unwrap();

    bytes
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(message: String, kind: ParseErrorKind) -> ParseError {
        ParseError { message, kind }
    }
}

#[derive(Debug)]
pub enum ParseErrorKind {
    HeaderError,
}

pub fn parse_ines(b: &Vec<u8>) -> Result<(), ParseError> {
    parse_ines_header(b)?;

    Ok(())
}

pub fn parse_ines_header(b: &Vec<u8>) -> Result<(), ParseError> {
    if str::from_utf8(&b[0..3]).unwrap() != "NES" {
        return Err(ParseError::new(
            String::from("could not find NES"),
            ParseErrorKind::HeaderError,
        ));
    }

    if b[3] != 0x1A {
        return Err(ParseError::new(
            String::from(format!("4th byte is not 0x1A but {}", b[3])),
            ParseErrorKind::HeaderError,
        ));
    }

    Ok(())
}
