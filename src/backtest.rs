use crate::{
    data::{DayTradeUnit, DayTradeUnitIter, StockCode, StockTradeData, TradeDataSource},
    strategy::{Account, Trade, TradeError},
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
    use std::{
        any::{request_ref, request_value, Any, Demand, Provider},
        collections::HashMap,
    };

    use ta::{
        indicators::{
            ExponentialMovingAverage, MovingAverageConvergenceDivergence,
            MovingAverageConvergenceDivergenceOutput, SimpleMovingAverage,
        },
        Next, Period,
    };

    use crate::{data::DayTradeUnit, strategy::Account};

    use super::{BackTest, Strategy};

    pub trait Serise: Provider {
        fn add(&mut self, value: f64);
    }

    impl dyn Serise {
        fn get<T: 'static>(&self) -> Option<&T> {
            request_ref(self)
        }
    }
    struct SeriseContainer<F, D> {
        data: Vec<D>,
        formula: F,
    }
    impl SeriseContainer<SimpleMovingAverage, f64> {
        fn new(formula: SimpleMovingAverage) -> Self {
            SeriseContainer {
                data: vec![],
                formula: formula,
            }
        }
    }
    impl Serise for SeriseContainer<SimpleMovingAverage, f64> {
        fn add(&mut self, value: f64) {
            let result = self.formula.next(value);
            self.data.push(result);
        }
    }
    impl Provider for SeriseContainer<SimpleMovingAverage, f64> {
        fn provide<'a>(&'a self, demand: &mut Demand<'a>) {
            demand.provide_ref(&self.data);
        }
    }
    impl SeriseContainer<ExponentialMovingAverage, f64> {
        fn new(formula: ExponentialMovingAverage) -> Self {
            SeriseContainer {
                data: vec![],
                formula: formula,
            }
        }
    }
    impl Serise for SeriseContainer<ExponentialMovingAverage, f64> {
        fn add(&mut self, value: f64) {
            todo!()
        }
    }
    impl Provider for SeriseContainer<ExponentialMovingAverage, f64> {
        fn provide<'a>(&'a self, demand: &mut Demand<'a>) {
            demand.provide_ref(&self.data);
        }
    }
    type MACDFomula = MovingAverageConvergenceDivergence;
    type MACD = MovingAverageConvergenceDivergenceOutput;
    impl SeriseContainer<MACDFomula, MACD> {
        fn new(formula: MACDFomula) -> Self {
            SeriseContainer {
                data: vec![],
                formula: formula,
            }
        }
    }
    impl Serise for SeriseContainer<MACDFomula, MACD> {
        fn add(&mut self, value: f64) {
            let result = self.formula.next(value);
            self.data.push(result);
        }
    }
    impl Provider for SeriseContainer<MACDFomula, MACD> {
        fn provide<'a>(&'a self, demand: &mut Demand<'a>) {
            demand.provide_ref(&self.data);
        }
    }
    pub struct MACross {
        serises: HashMap<&'static str, Box<dyn Serise>>,
    }

    impl MACross {
        fn new() -> Self {
            let sma5 = Box::new(SeriseContainer::<SimpleMovingAverage, f64>::new(
                SimpleMovingAverage::new(5).unwrap(),
            ));
            let sma5: Box<dyn Serise> = sma5;
            let macd = Box::new(SeriseContainer::<MACDFomula, MACD>::new(
                MACDFomula::new(3, 6, 4).unwrap(),
            ));
            let macd: Box<dyn Serise> = macd;
            let mut serises = HashMap::new();
            serises.insert("sma5", sma5);
            serises.insert("macd", macd);
            MACross { serises }
        }
    }

    impl Strategy for MACross {
        fn next(&mut self, account: &mut Account, stock_trade_info: &Vec<DayTradeUnit>) {
            if let Some(today) = stock_trade_info.last() {
                for (k, v) in self.serises.iter_mut() {
                    v.add(f64::from(today.trade_data.close));
                }
                let macd: &Vec<MACD> = self.serises.get("macd").unwrap().get().unwrap();
                println!("macd: {:?}", macd)
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
