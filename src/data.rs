use byteorder::{ByteOrder, LittleEndian};
use chrono::{Utc, DateTime};
#[cfg(test)]
use std::{println as info, println as warn};
#[cfg(not(test))] 
use log::{info, warn};
use std::{
    env,
    fs::File,
    io::{Read, Seek, self},
    mem::{align_of, size_of},
    path::{Path, PathBuf}, time::Duration,
    convert::{Into, From}, any::Any,
};


#[repr(C)]
#[derive(Debug)]
pub struct Header {
    types: i16,
    report_date: i32,
    max_count: i16,
    uname: i32,
    report_size: i32,
    uname2: i32,
}

pub const HeaderSize: usize = 0x14;

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
pub struct StockItem {
    code: String,
    offset: u32,
}

pub const StockSize: usize = 7 + 4;

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
pub struct StockFinanceValue (f32);

const StockFinanceColValueSize: usize = 4;

impl Deserializer for StockFinanceValue {
    fn deserializer(buffer: &[u8]) -> Self {
        StockFinanceValue(LittleEndian::read_f32(&buffer))
    }
}

#[derive(Debug)]
pub struct TradeUnit {
    pub open: i32,
    pub close: i32,
    pub high: i32,
    pub low: i32,
    pub volume: i32,
    pub amount: f32,
}
pub const TradeUnitSize:usize = 24;

impl Deserializer for TradeUnit {
    fn deserializer(buffer: &[u8]) -> Self {
        TradeUnit { 
            open: LittleEndian::read_i32(&buffer[..4]), 
            high: LittleEndian::read_i32(&buffer[4..8]),
            low: LittleEndian::read_i32(&buffer[8..12]),
            close: LittleEndian::read_i32(&buffer[12..16]), 
            amount: LittleEndian::read_f32(&buffer[16..20]), 
            volume: LittleEndian::read_i32(&buffer[20..24]),
        }
    }
}

#[derive(Debug)]
pub struct DayTradeUnit {
    pub date: i32,
    pub trade_data: TradeUnit
}
pub const DayTradeUnitSize: usize = 32;

impl Deserializer for DayTradeUnit {
    fn deserializer(buffer: &[u8]) -> Self {
        DayTradeUnit { 
            date: LittleEndian::read_i32(&buffer[..4]), 
            trade_data:  TradeUnit::deserializer(&buffer[4..]) }
    }
}


#[derive(Debug)]
pub struct MinuteTradeUnit {
    date: i32,
    // 0点至目前的分钟数
    offset: i16,
    tradeData: TradeUnit
}
pub const MinuteTradeUnitSize: usize = 32;

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

use thiserror::Error;
#[derive(Debug, Error)]
pub enum DataSourceError {
    #[error("read file error")]
    Io {
        #[from]
        source: io::Error,
    }
}

pub type Result<T, E = DataSourceError> = std::result::Result<T, E>;

/**
 * 股票代码
 */
pub type StockCode = &'static str;
/**
 * 股票价格
 */
pub type StockPrice = f64;
/**
 * 成交量
 */
pub type Volume = i32;
/**
 * 交易时间
 */
pub type TradeTime = DateTime<Utc>;



pub trait TradeDataSource{
    fn prepare(&self) -> Result<()>;
    fn day(&self, code: StockCode, day: TradeTime) -> Result<DayTradeUnit>;
    fn minue(&self, code: StockCode, day: TradeTime) -> Result<MinuteTradeUnit>;
    fn day_duration(&self, code: StockCode, from: Option<TradeTime>, to: Option<TradeTime>) -> Result<DayTradeUnitIter>;
    fn minue_duration(&self, code: StockCode, day: TradeTime, from: TradeTime, to: TradeTime) -> MinuteTradeUnitIter;
}

pub struct StockTradeData {
    // code: StockCode,
}

pub struct DayTradeUnitIter {
    file: File,
}

impl DayTradeUnitIter {
    fn new(path: &Path) -> Result<Self> {
        
        let mut file = File::open(path)?;
        Ok(DayTradeUnitIter { 
            file: file
        })
    }
}

impl Iterator for DayTradeUnitIter {
    type Item = DayTradeUnit;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buff = [0u8; DayTradeUnitSize];
        match self.file.read(&mut buff[..]) {
            Ok(size) => match size {
                0 => return None,
                _ => ..,
            },
            Err(e) => return None,
        };
        Some(DayTradeUnit::deserializer(&buff))
    }
}

pub struct MinuteTradeUnitIter(MinuteTradeUnit);

impl Iterator for MinuteTradeUnitIter {
    type Item = MinuteTradeUnit;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl TradeDataSource for StockTradeData {
    fn prepare(&self) -> Result<()> {
        todo!()
    }

    fn day(&self, code: StockCode, day: TradeTime) -> Result<DayTradeUnit> {
        todo!()
    }

    fn minue(&self, code: StockCode, day: TradeTime) -> Result<MinuteTradeUnit> {
        todo!()
    }

    fn day_duration(&self, code: StockCode, from: Option<TradeTime>, to: Option<TradeTime>) -> Result<DayTradeUnitIter> {
        let path = get_day_path_by_code(code)?;
        let iter = DayTradeUnitIter::new(&path)?;
        Ok(iter)
    }

    fn minue_duration(&self, code: StockCode, day: TradeTime, from: TradeTime, to: TradeTime) -> MinuteTradeUnitIter {
        todo!()
    }
}

/**
 * todo: 工具函数
 * 1 通过code找到文件路径
 * 2 通过code判断是属于哪个市场
 */
fn get_day_path_by_code(code: StockCode) -> Result<PathBuf> {
    let parent = env::current_dir()?;
    let file_name = format!("sh{}.day", code);
    let data_path = Path::new(&parent).join("example").join(file_name);
    info!("day trade datafile path: {:?}", data_path);
    Ok(data_path)
}

#[cfg(test)]
mod tests {
    use super::{StockTradeData, TradeDataSource};


    #[test]
    fn iter_day_info() {
        let data = StockTradeData {};
        let iter = data.day_duration("603339", None, None).unwrap();

        for i in iter.take(10) {
            println!("date: {}, close: {}, amount: {}", i.date, i.trade_data.close, i.trade_data.amount)
        }
    }
}