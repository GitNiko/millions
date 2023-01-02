

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
    use ta::{indicators::ExponentialMovingAverage, Next};

    use crate::{strategy::Account, data::DayTradeUnit};

    use super::{BackTest, Strategy};

    pub struct MACross {
        ema5: ExponentialMovingAverage,
        ema10: ExponentialMovingAverage,
        EMA5: Vec<f64>,
        EMA10: Vec<f64>
    }

    impl MACross {
        fn new() -> Self {
            let ema5 = ExponentialMovingAverage::new(5).unwrap();
            let ema10 = ExponentialMovingAverage::new(10).unwrap();
            MACross { ema5, ema10, EMA5: Vec::new(), EMA10: Vec::new() }
        }
    }

    impl Strategy for MACross {
        fn next(&mut self, account: &mut Account, stock_trade_info: &Vec<DayTradeUnit>) {
            if let Some(today) = stock_trade_info.last() {
                let ema5 = self.ema5.next(f64::from(today.trade_data.close));
                let ema10 = self.ema10.next(f64::from(today.trade_data.close));
                self.EMA5.push(ema5);
                self.EMA10.push(ema10);
                println!("{:?}", ema5);
            }
        }
    }

    #[test]
    fn iter_day_info() {
        let mut ma = MACross::new();
        let strategy = &mut ma as &mut dyn Strategy;
        let mut backTest = BackTest::new("603339", strategy).unwrap();
        backTest.run("20160520", "20180601");
    }
}
