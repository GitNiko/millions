

use crate::{
    data::{DayTradeUnit, DayTradeUnitIter, StockCode, StockTradeData, TradeDataSource},
    strategy::{Account, TradeError, Trade},
};

use thiserror::Error;
#[derive(Debug, Error, PartialEq)]
pub enum BackTestError {
    #[error("trade error")]
    TradeError {
        #[from]
        source: TradeError,
    },
}

pub type Result<T, E = BackTestError> = std::result::Result<T, E>;

pub struct BackTest<'a> {
    /** 回测账户 */
    account: Account,
    /** 策略 */
    // strategy: &'a dyn Strategy,
    code: StockCode,
    data: Vec<DayTradeUnit>,
    strategy: &'a mut dyn Strategy,
}

pub trait Strategy {
    fn next(&mut self, account: &mut Account, stock_trade_info: &Vec<DayTradeUnit>);
}


impl<'a> BackTest<'a> {
    fn new(code: StockCode, strategy: &'a mut dyn Strategy) -> Result<Self> {
        // todo: change to builder pattern
        let mut account = Account::new("100000", "0.00025")?;
        Ok(BackTest {
            code,
            data: vec![],
            account,
            strategy,
        })
    }

    fn run(&mut self, from: &str, to: &str) -> () {
        let start: i32 = from.parse().unwrap();
        let end: i32 = to.parse().unwrap();
        let data = StockTradeData {};
        let iter = data.day_duration(self.code, None, None).unwrap();
        for i in iter {
            if i.date < start {
                continue;
            }
            if i.date > end {
                continue;
            }
            self.data.push(i);
            self.strategy.next(&mut self.account, &self.data);
        }
    }
}


#[cfg(test)]
mod tests {
    use ta::{indicators::{ExponentialMovingAverage, SimpleMovingAverage}, Next};

    use crate::{strategy::Account, data::DayTradeUnit};

    use super::{BackTest, Strategy};

    pub trait Serise {
        fn get();
        fn add();
        fn calc();
    }
    pub struct MACross {
        ema5: ExponentialMovingAverage,
        ema10: ExponentialMovingAverage,
        ema5_data: Vec<f64>,
        ema10_data: Vec<f64>,
        sma5: SimpleMovingAverage,
        sma10: SimpleMovingAverage,
        sma5_data: Vec<f64>,
        sma10_data: Vec<f64>
    }

    impl MACross {
        fn new() -> Self {
            let ema5 = ExponentialMovingAverage::new(5).unwrap();
            let ema10 = ExponentialMovingAverage::new(10).unwrap();
            let sma5 = SimpleMovingAverage::new(5).unwrap();
            let sma10 = SimpleMovingAverage::new(10).unwrap();
            MACross { ema5, ema10, ema5_data: Vec::new(), ema10_data: Vec::new(), sma5, sma10, sma5_data: Vec::new(), sma10_data: Vec::new() }
        }
    }

    impl Strategy for MACross {
        fn next(&mut self, account: &mut Account, stock_trade_info: &Vec<DayTradeUnit>) {
            if let Some(today) = stock_trade_info.last() {
                let ema5 = self.ema5.next(f64::from(today.trade_data.close));
                let ema10 = self.ema10.next(f64::from(today.trade_data.close));
                self.ema5_data.push(ema5);
                self.ema10_data.push(ema10);
                let sma5 = self.sma5.next(f64::from(today.trade_data.close));
                let sma10 = self.sma10.next(f64::from(today.trade_data.close));
                if sma5 > sma10 && sma10 > 0.0 && sma5 > 0.0 {
                    // println!("{}  {}", sma5, sma10)
                    println!("{:?}", today);
                }
            }
        }
    }

    #[test]
    fn iter_day_info() {
        let mut ma = MACross::new();
        let strategy = &mut ma as &mut dyn Strategy;
        let mut backTest = BackTest::new("603339", strategy).unwrap();
        backTest.run("20220901", "20230103");
    }
}
