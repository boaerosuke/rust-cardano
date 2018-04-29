//! Cardano's Lovelace value
//! 
//! This represents the type value and has some properties associated
//! such as a min bound of 0 and a max bound of `MAX_COIN`.
//! 

use cbor;
use cbor::ExtendedResult;
use std::{ops, fmt, result};

/// maximum value of a Lovelace.
pub const MAX_COIN: u64 = 45000000000000000;

/// error type relating to `Coin` operations
/// 
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Error {
    /// means that the given value was out of bound
    /// 
    /// Max bound being: `MAX_COIN`.
    OutOfBound(u64)
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::OutOfBound(v) => write!(f, "Coin of value {} is out of bound. Max coin value: {}.", v, MAX_COIN),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

// TODO: add custom implementation of `serde::de::Deserialize` so we can check the
// upper bound of the `Coin`.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Coin(u64);
impl Coin {
    /// create a coin of value `0`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use wallet_crypto::coin::{Coin};
    /// 
    /// println!("{}", Coin::zero());
    /// ```
    pub fn zero() -> Self { Coin(0) }

    /// create a coin of the given value
    /// 
    /// # Example
    /// 
    /// ```
    /// use wallet_crypto::coin::{Coin};
    /// 
    /// let coin = Coin::new(42);
    /// let invalid = Coin::new(45000000000000001);
    /// 
    /// assert!(coin.is_ok());
    /// assert!(invalid.is_err());
    /// ```
    pub fn new(v: u64) -> Result<Self> {
        if v <= MAX_COIN { Ok(Coin(v)) } else { Err(Error::OutOfBound(v)) }
    }
}
impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl cbor::CborValue for Coin {
    fn encode(&self) -> cbor::Value { cbor::Value::U64(self.0) }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.u64().and_then(|v| {
            match Coin::new(v) {
                Ok(coin) => Ok(coin),
                Err(Error::OutOfBound(_)) => cbor::Result::u64(v, cbor::Error::Between(0, MAX_COIN))
            }
        })
    }
}
impl ops::Add for Coin {
    type Output = Result<Coin>;
    fn add(self, other: Coin) -> Self::Output {
        Coin::new(self.0 + other.0)
    }
}
impl<'a> ops::Add<&'a Coin> for Coin {
    type Output = Result<Coin>;
    fn add(self, other: &'a Coin) -> Self::Output {
        Coin::new(self.0 + other.0)
    }
}
impl ops::Sub for Coin {
    type Output = Option<Coin>;
    fn sub(self, other: Coin) -> Self::Output {
        if other.0 > self.0 { None } else { Some(Coin(self.0 - other.0)) }
    }
}
impl<'a> ops::Sub<&'a Coin> for Coin {
    type Output = Option<Coin>;
    fn sub(self, other: &'a Coin) -> Self::Output {
        if other.0 > self.0 { None } else { Some(Coin(self.0 - other.0)) }
    }
}
// this instance is necessary to chain the substraction operations
//
// i.e. `coin1 - coin2 - coin3`
impl ops::Sub<Coin> for Option<Coin> {
    type Output = Option<Coin>;
    fn sub(self, other: Coin) -> Self::Output {
        if other.0 > self?.0 { None } else { Some(Coin(self?.0 - other.0)) }
    }
}

