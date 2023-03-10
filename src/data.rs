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
    // 0????????????????????????
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
    },
    #[error("read env error")]
    Env {
        #[from]
        source: env::VarError
    },
    #[error("stock not exist in market")]
    StockCodeNotExistInMarket
}

pub type Result<T, E = DataSourceError> = std::result::Result<T, E>;

/**
 * ????????????
 */
pub type StockCode = &'static str;
pub trait WhereIsFrom {
    /// ??????????????????????????????
    fn where_is_from(&self) -> Option<Market>;
}
impl WhereIsFrom for &str {
    fn where_is_from(&self) -> Option<Market> {
        if self.starts_with("600") 
            || self.starts_with("601") 
            || self.starts_with("603")
            || self.starts_with("605") 
            || self.starts_with("688")
            || self.starts_with("900"){
            return Some(Market::SH)
        }
        if self.starts_with("000")
            || self.starts_with("001")
            || self.starts_with("002")
            || self.starts_with("003")
            || self.starts_with("300")
            || self.starts_with("301")
            || self.starts_with("200")
            || self.starts_with("201") {
            return Some(Market::SZ)
        }
        None
    }
}
/// ??????
pub enum Market {
    SZ,
    SH
}
/**
 * todo: ????????????
 * 1 ??????code??????????????????
 * 2 ??????code???????????????????????????
 */
fn get_day_path_by_code(code: StockCode) -> Result<PathBuf> {
    let root = env::var("MILLIONS_TDX")?;
    let market = code.where_is_from().ok_or(DataSourceError::StockCodeNotExistInMarket)?;
    match market {
        Market::SZ => {
            let file_name = format!("sz{}.day", code);
            let data_path = Path::new(&root).join("sz").join("lday").join(file_name);
            info!("day trade datafile path: {:?}", data_path);
            return Ok(data_path)
        },
        Market::SH =>  {
            let file_name = format!("sh{}.day", code);
            let data_path = Path::new(&root).join("sh").join("lday").join(file_name);
            info!("day trade datafile path: {:?}", data_path);
            return Ok(data_path)
        },
    }
}
/**
 * ????????????
 */
pub type StockPrice = f64;
/**
 * ?????????
 */
pub type Volume = i32;
/**
 * ????????????
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