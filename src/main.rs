use byteorder::{ByteOrder, LittleEndian};
use std::{
    env,
    fs::File,
    io::{Read, Seek},
    mem::{align_of, size_of},
    path::Path,
};

#[repr(C)]
#[derive(Debug)]
struct Header {
    types: i16,
    report_date: i32,
    max_count: i16,
    uname: i32,
    report_size: i32,
    uname2: i32,
}

const HeaderSize: usize = 0x14;

trait Deserializer {
    fn deserializer(buffer: &[u8]) -> Self;
}

impl Deserializer for Header {
    fn deserializer(buffer: &[u8]) -> Self {
        Header {
            types: LittleEndian::read_i16(&buffer[0..2]),
            report_date: LittleEndian::read_i32(&buffer[2..6]),
            max_count: LittleEndian::read_i16(&buffer[6..8]),
            uname: LittleEndian::read_i32(&buffer[8..12]),
            report_size: LittleEndian::read_i32(&buffer[12..16]),
            uname2: LittleEndian::read_i32(&buffer[16..20]),
        }
    }
}

#[derive(Debug)]
struct StockItem {
    code: String,
    offset: u32,
}

const StockSize: usize = 7 + 4;

impl Deserializer for StockItem {
    fn deserializer(buffer: &[u8]) -> Self {
        println!("{:?}", &buffer[0..6]);
        let code = String::from_utf8(buffer[0..6].to_vec()).unwrap();
        StockItem {
            code,
            offset: LittleEndian::read_u32(&buffer[7..11]),
        }
    }
}

#[derive(Debug)]
struct StockFinanceValue (f32);

const StockFinanceColValueSize: usize = 4;

impl Deserializer for StockFinanceValue {
    fn deserializer(buffer: &[u8]) -> Self {
        StockFinanceValue(LittleEndian::read_f32(&buffer))
    }
}

fn main() {
    let parent = env::current_dir().unwrap();
    let data_path = Path::new(&parent).join("example").join("gpcw20190630.dat");
    let mut file = File::open(data_path).unwrap();
    let mut buff = [0u8; HeaderSize];
    file.read(&mut buff[..]).unwrap();
    let header = Header::deserializer(&buff);
    println!("{:?}", &header);

    //https://github.com/rainx/pytdx/pull/150/files
    file.seek(std::io::SeekFrom::Start(HeaderSize as u64));
    println!(
        "seek: {:?}, header size: {:?}, align size: {:?}",
        std::io::SeekFrom::Start(size_of::<Header>().try_into().unwrap()),
        size_of::<Header>(),
        align_of::<Header>()
    );
    let mut stockBuff = [0u8; StockSize];
    file.read(&mut stockBuff);
    let stockHeader = StockItem::deserializer(&stockBuff);
    // A6A0
    println!("{:?}", &stockHeader);
    let cols = header.report_size / 4;
    println!("data cols count: {:?}", cols);
    let col:u64 = 94;
    file.seek(std::io::SeekFrom::Start(u64::from(stockHeader.offset) + col * 4));
    let mut data_col = [0u8; StockFinanceColValueSize];
    // let mut info_data = [0f32; 580];
    file.read(&mut data_col);
    let final_val = StockFinanceValue::deserializer(&data_col);
    println!("data cols count: {:?}", final_val);

}

