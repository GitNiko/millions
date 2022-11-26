#![feature(int_roundings)]

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

#[derive(Debug)]
struct TradeUnit {
    open: i32,
    close: i32,
    high: i32,
    low: i32,
    volume: i32,
    amount: f32,
}
const TradeUnitSize:usize = 24;

impl Deserializer for TradeUnit {
    fn deserializer(buffer: &[u8]) -> Self {
        TradeUnit { 
            open: LittleEndian::read_i32(&buffer[0..3]), 
            high: LittleEndian::read_i32(&buffer[4..7]),
            low: LittleEndian::read_i32(&buffer[8..11]),
            close: LittleEndian::read_i32(&buffer[12..15]), 
            amount: LittleEndian::read_f32(&buffer[16..19]), 
            volume: LittleEndian::read_i32(&buffer[20..23]),
        }
    }
}

#[derive(Debug)]
struct DayTradeUnit {
    date: i32,
    tradeData: TradeUnit
}
const DayTradeUnitSize: usize = 32;

impl Deserializer for DayTradeUnit {
    fn deserializer(buffer: &[u8]) -> Self {
        DayTradeUnit { 
            date: LittleEndian::read_i32(&buffer[0..3]), 
            tradeData:  TradeUnit::deserializer(&buffer[4..]) }
    }
}

#[derive(Debug)]
struct MinuteTradeUnit {
    date: i32,
    // 0点至目前的分钟数
    offset: i16,
    tradeData: TradeUnit
}
const MinuteTradeUnitSize: usize = 32;

impl Deserializer for MinuteTradeUnit {
    fn deserializer(buffer: &[u8]) -> Self {
        let date_raw = LittleEndian::read_i16(&buffer[0..1]);
        let year = date_raw.div_floor(2048) + 2004;
        let month = (date_raw % 2048).div_floor(100);
        let day = (date_raw % 2048) % 100;
        MinuteTradeUnit { 
            date: 0, // todo
            offset: LittleEndian::read_i16(&buffer[2..3]),
            tradeData: TradeUnit::deserializer(&buffer[4..]) 
        }
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

