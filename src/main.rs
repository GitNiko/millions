use byteorder::{ByteOrder, LittleEndian};
use std::{env, fs::File, io::Read, mem::size_of, path::Path};

#[repr(C)]
#[derive(Debug)]
struct Header {
    types: i16,
    report_date: i32,
    max_count: i16,
    uname: i32,
    uname2: i32,
    report_size: i32,
}

impl Header {
    fn make_from_bytes(buffer: &[u8]) -> Header {
        Header {
            types: LittleEndian::read_i16(&buffer[0..2]),
            report_date: LittleEndian::read_i32(&buffer[2..6]),
            max_count: LittleEndian::read_i16(&buffer[6..8]),
            uname: LittleEndian::read_i32(&buffer[8..12]),
            uname2: LittleEndian::read_i32(&buffer[12..16]),
            report_size: LittleEndian::read_i32(&buffer[16..20]),
        }
    }
}

fn main() {
    let parent = env::current_dir().unwrap();
    let data_path = Path::new(&parent).join("example").join("gpcw20190630.dat");
    let mut file = File::open(data_path).unwrap();
    let mut buff = [0u8; size_of::<Header>()];
    file.read(&mut buff[..]).unwrap();
    let header = Header::make_from_bytes(&buff);
    println!("{:?}", &header);
}
