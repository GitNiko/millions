use std::{collections::HashMap, num::ParseIntError};

use crate::data::{StockCode, TradeTime};

use chrono::{DateTime, Utc, Local};
use rust_decimal::{Decimal};
use thiserror::Error;
#[derive(Debug, Error, PartialEq)]
pub enum TradeError {
    /// 余额不足
    #[error("out of balance")]
    OutOfBalance,
    #[error("parse decimal fail")]
    DecimalError {
        #[from]
        source: rust_decimal::Error,
    },
    #[error("parse int fail")]
    ParseIntError {
        #[from]
        source: ParseIntError
    },
    #[error("decimal to number fail")]
    DecimalToNumberError,
    #[error("out of position")]
    OutOfPosition,
}

pub type Result<T, E = TradeError> = std::result::Result<T, E>;


pub trait Trade {
    fn buy(& mut self, code: StockCode, price: &str, vol: &str) -> Result<()>;
    fn sell(& mut self, code: StockCode, price: &str, vol: &str) -> Result<()>;
    /// 按余额百分比购买
    ///
    /// # Parameter
    ///
    /// * `percent` - 百分比 例如 20表示20%
    fn buy_part(&mut self, code: StockCode, price: &str, percent: u8);
    fn sell_part(&mut self, code: StockCode, price: &str, percent: u8);
}

type TODO = i32;
pub trait Strategy {
    fn decide(&self, now: TODO) -> ();
}

pub struct Exchange {
    
}

pub struct Account {
    /**
     * 余额
     */
    balance: Decimal,

    transaction_record: TransactionRecord,
    /**
     * 佣金费率
     */
    rate_brokerage_fee: Decimal,
    /**
     * 印花税
     */
    rate_stamp_duty: Decimal,
    /**
     * 交易费
     */
    rate_transfer_fee: Decimal,
}

pub type TransactionRecord = HashMap<StockCode, Vec<Order>>;
/// 成本价格
pub type StockCostPrice = Decimal;
/// 持仓
#[derive(Debug)]
pub struct Position(StockCode, OrderSettle);
#[derive(Debug)]
pub struct OrderSettle(
    /// 成交量(股)
    Decimal, 
    /// 均价
    StockCostPrice
);


impl Into<OrderSettle> for &Vec<Order>  {
    fn into(self) -> OrderSettle {
        self.iter().fold(OrderSettle(StockCostPrice::new(0, 2), StockCostPrice::new(0, 2)), |mut acc, x| {
            if (acc.0 + x.vol).is_zero() == false {
                acc.1 = (acc.1 * acc.0 + x.vol * x.price + x.brokerage_fee + x.stamp_duty + x.transfer_fee) / (acc.0 + x.vol);
                acc.0 = acc.0 + x.vol;
            }
            acc
        })
    }
}

/**
 * 订单
 */
#[derive(Debug)]
pub struct Order {
    time: TradeTime,
    vol: Decimal,
    price: Decimal,
    stamp_duty: Decimal,
    brokerage_fee: Decimal,
    transfer_fee: Decimal,
}

impl Account {
    pub fn new(balance: &str, rate_brokerage_fee: &str) -> Result<Self> {
        Ok(Account { 
            balance: Decimal::from_str_exact(balance)?, 
            transaction_record: HashMap::new(), 
            rate_brokerage_fee: Decimal::from_str_exact(rate_brokerage_fee)?, // 0.00025
            rate_stamp_duty: Decimal::from_str_exact("0.001")?,
            rate_transfer_fee: Decimal::from_str_exact("0.00001")?,
        })
    }
    
    /// 查看指定股票的持仓状态
    pub fn get_position(&self, code: StockCode) -> Result<Option<Position>> {
        if let Some(record) = self.transaction_record.get(code) {
            let settle: OrderSettle = record.into();
            Ok(Some(Position(code, settle)))
        } else {
            Ok(None)
        }
    }

    pub fn get_transaction(&self, code: StockCode) -> Option<&Vec<Order>> {
        self.transaction_record.get(code)
    }

    /// 查看余额
    pub fn get_balance(&self) -> Decimal {
        self.balance
    }
}

impl Trade for Account {
    fn buy(&mut self, code: StockCode, price: &str, vol: &str) -> Result<()> {
        let price = Decimal::from_str_exact(price)?;
        let vol = Decimal::from_str_exact(vol)?;
        let charge = price * vol;
        let stamp_duty = Decimal::from_str_exact("0")?;
        let mut brokerage_fee = (charge * self.rate_brokerage_fee).ceil_point(2);
        let min_brokerage_fee = Decimal::from_str_exact("0.5")?;
        if brokerage_fee < min_brokerage_fee {
            brokerage_fee = min_brokerage_fee;
        }
        let transfer_fee = (charge * self.rate_transfer_fee).ceil_point(2);
        let balance = self.balance - brokerage_fee - transfer_fee - charge;
        if balance.is_sign_negative() {
            return Err(TradeError::OutOfBalance);
        }
        self.balance = balance;
        let time = DateTime::<Utc>::from(Local::now());
        if let Some(orders) = self.transaction_record.get_mut(code) {
            orders.push(Order { time, vol, price, stamp_duty, brokerage_fee, transfer_fee, });
        } else {
            let mut orders = vec![];
            orders.push(Order { time, vol, price, stamp_duty, brokerage_fee, transfer_fee, });
            self.transaction_record.insert(code, orders);
        }
        Ok(())
    }

    fn sell(&mut self, code: StockCode, price: &str, vol: &str) -> Result<()> {
        let vol = Decimal::from_str_exact(vol)?;

        if let Some(Position(.., OrderSettle(current, ..))) = self.get_position(code)? {
            if vol > current {
                return Err(TradeError::OutOfPosition);
            }
        } else {
            return Err(TradeError::OutOfPosition);
        }
        
        let price = Decimal::from_str_exact(price)?;
        let charge = price * vol;
        let stamp_duty = self.rate_stamp_duty * charge;
        let mut brokerage_fee = (charge * self.rate_brokerage_fee).ceil_point(2);
        let min_brokerage_fee = Decimal::from_str_exact("0.5")?;
        if brokerage_fee < min_brokerage_fee {
            brokerage_fee = min_brokerage_fee;
        }
        let transfer_fee = (charge * self.rate_transfer_fee).ceil_point(2);
        let balance = self.balance + charge - brokerage_fee - transfer_fee  - stamp_duty;
        if balance.is_sign_negative() {
            return Err(TradeError::OutOfBalance);
        }
        self.balance = balance;
        let time = DateTime::<Utc>::from(Local::now());
        if let Some(orders) = self.transaction_record.get_mut(code) {
            orders.push(Order { time, vol: -vol, price, stamp_duty, brokerage_fee, transfer_fee, });
        } // unreach else
        Ok(())
    }

    fn buy_part(&mut self, code: StockCode, price: &str, percent: u8) {
        todo!()
    }

    fn sell_part(&mut self, code: StockCode, price: &str, percent: u8) {
        todo!()
    }
    
}

trait Ceil {
    fn ceil_point(&self, scale: u8) -> Self;
}

impl Ceil for Decimal {
    fn ceil_point(&self, scale: u8) -> Self {
        let multiplier = Decimal::from(10i64.pow(scale as u32));
	    (self * multiplier).ceil() / multiplier
    }
}


#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::strategy::{Position, OrderSettle};

    use super::{Account, Trade, TradeError};


    #[test]
    fn out_of_balance() {
        let mut account = Account::new("100_000.0", "0.00025").unwrap();
        let out_of_balance = account.buy("603339", "20.0", "10_000").unwrap_err();
        assert_eq!(out_of_balance, TradeError::OutOfBalance);
    }

    #[test]
    fn buy_stock() {
        let mut account = Account::new("100_000.0", "0.00025").unwrap();

        account.buy("603339", "20.0", "1_000").unwrap();
        let balance = account.get_balance();
        assert_eq!(balance, Decimal::from_str_exact("79994.8").unwrap());
        
        account.buy("603339", "20.0", "2_000").unwrap();
        let balance = account.get_balance();
        assert_eq!(balance, Decimal::from_str_exact("39984.4").unwrap());

        let position = account.get_position("603339").unwrap().unwrap();
        assert_eq!(position.0, "603339");
        let settle = position.1;
        assert_eq!(settle.0, Decimal::from_str_exact("3_000").unwrap());
        assert_eq!(settle.1, Decimal::from_str_exact("20.0052").unwrap());

        let position = account.get_position("000000").unwrap();
        assert!(position.is_none());

        let out_of_position = account.sell("000000", "30", "2000").unwrap_err();
        assert_eq!(out_of_position, TradeError::OutOfPosition);

        let out_of_position = account.sell("603339", "30", "5000").unwrap_err();
        assert_eq!(out_of_position, TradeError::OutOfPosition);

        account.sell("603339", "30.0", "1000").unwrap();
        let balance = account.get_balance();
        assert_eq!(balance, Decimal::from_str_exact("69946.6").unwrap());
        let Position(.., OrderSettle(current, cost)) = account.get_position("603339").unwrap().unwrap();
        assert_eq!(current, Decimal::from_str_exact("2000").unwrap());
        assert_eq!(cost, Decimal::from_str_exact("15.0267").unwrap());

        let records = account.get_transaction("603339").unwrap();
        println!("{:?}", records)
        
    }
}
