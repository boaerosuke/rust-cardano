/// BIP44 addressing
///
/// provides all the logic to create safe sequential addresses
/// using BIP44 specification.
///
/// # Example
///
/// ```
/// use wallet_crypto::bip44::{Account, Change, Addressing};
///
/// let addr = Account::new(0x80000000).unwrap()
///     .external().unwrap()
///     .index(0).unwrap();
///
/// assert!(addr.index =- 0);
/// ```

use hdpayload::{Path};
use std::{fmt, result};

/// the BIP44 derivation path has a specific length
pub const BIP44_PATH_LENGTH : usize = 5;
/// the BIP44 derivation path has a specific purpose
pub const BIP44_PURPOSE   : u32 = 0x8000002C;
/// the BIP44 coin type is set, by default, to cardano ada.
pub const BIP44_COIN_TYPE : u32 = 0x80000717;

/// the soft derivation is upper bounded
pub const BIP44_SOFT_UPPER_BOUND : u32 = 0x80000000;

/// Error relating to `bip44`'s `Addressing` operations
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Error {
    /// this means the given `Path` has an incompatible length
    /// for bip44 derivation. See `BIP44_PATH_LENGTH` and `Addressing::from_path`.
    InvalidLength(usize),

    /// this means the given `Path` has an incompatible purpose
    /// for bip44 derivation. See `BIP44_PURPOSE` and `Addressing::from_path`.
    InvalidPurpose(u32),

    /// this means the given `Path` has an incompatible coin type
    /// for bip44 derivation. See `BIP44_COIN_TYPE` and `Addressing::from_path`.
    InvalidType(u32),

    /// this means the given `Path` has an incompatible account
    /// for bip44 derivation. That it is out of bound. Indeed
    /// the account derivation is expected to be a hard derivation.
    AccountOutOfBound(u32),

    /// this means the given `Path` has an incompatible change
    /// for bip44 derivation. That it is out of bound. Indeed
    /// the change derivation is expected to be a soft derivation.
    ChangeOutOfBound(u32),

    /// this means the given `Path` has an incompatible index
    /// for bip44 derivation. That it is out of bound. Indeed
    /// the index derivation is expected to be a soft derivation.
    IndexOutOfBound(u32)
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidLength(given)     => write!(f, "Invalid length, expecting {} but received {}", BIP44_PATH_LENGTH, given),
            &Error::InvalidPurpose(given)   => write!(f, "Invalid purpose, expecting 0x{:x} but received 0x{:x}", BIP44_PURPOSE, given),
            &Error::InvalidType(given)       => write!(f, "Invalid type, expecting 0x{:x} but received 0x{:x}", BIP44_COIN_TYPE, given),
            &Error::AccountOutOfBound(given) => write!(f, "Account out of bound, should have a hard derivation but received 0x{:x}", given),
            &Error::ChangeOutOfBound(given) => write!(f, "Change out of bound, should have a soft derivation but received 0x{:x}", given),
            &Error::IndexOutOfBound(given) => write!(f, "Index out of bound, should have a soft derivation but received 0x{:x}", given),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Account(u32);
impl Account {
    pub fn new(account: u32) -> Result<Self> {
        if account  <  0x80000000 { return Err(Error::AccountOutOfBound(account)); }
        Ok(Account(account))
    }

    pub fn internal(&self) -> Result<Change> {
        Change::new(*self, 1)
    }
    pub fn external(&self) -> Result<Change> {
        Change::new(*self, 0)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Change {
    account: Account,
    change:  u32
}
impl Change {
    pub fn new(account: Account, change: u32) -> Result<Self> {
        if change  >= 0x80000000 { return Err(Error::ChangeOutOfBound(change)); }
        Ok(Change{ account: account, change: change })
    }

    pub fn index(&self, index: u32) -> Result<Addressing> {
        Addressing::new_from_change(self, index)
    }
}

/// Bip44 address derivation
///
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Addressing {
    pub account: u32,
    pub change: u32,
    pub index: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AddrType {
    Internal,
    External,
}

impl Addressing {
    /// create a new `Addressing` for the given account and `AddrType`.
    /// The default index is set to `0` (the starting index).
    ///
    /// # example
    ///
    /// ```
    /// use wallet_crypto::bip44::{Addressing, AddrType};
    ///
    /// let addr = Addressing::new(0x80000000, AddrType::External).unwrap();
    ///
    /// assert!(Addressing::new(0, AddrType::External).is_err());
    /// ```
    pub fn new(account: u32, typ: AddrType) -> Result<Self> {
        let change = match typ {
                        AddrType::Internal => 1,
                        AddrType::External => 0,
                    };
        if account  <  0x80000000 { return Err(Error::AccountOutOfBound(account)); }
        Ok(Addressing { account: account, change: change, index: 0 })
    }

    fn new_from_change(change: &Change, index: u32) -> Result<Self> {
        if index  >= 0x80000000 { return Err(Error::IndexOutOfBound(index)); }
        Ok(Addressing{account: change.account.0, change: change.change, index: index})
    }

    /// return a path ready for derivation
    pub fn to_path(&self) -> Path {
        Path::new(vec![BIP44_PURPOSE, BIP44_COIN_TYPE, self.account, self.change, self.index])
    }

    pub fn address_type(&self) -> AddrType {
        if self.change == 0 {
            AddrType::External
        } else {
            AddrType::Internal
        }
    }

    pub fn from_path(path: Path) -> Result<Self> {
        let len = path.as_ref().len();
        if path.as_ref().len() != BIP44_PATH_LENGTH { return Err(Error::InvalidLength(len)); }

        let p = path.as_ref()[0];
        if p != BIP44_PURPOSE   { return Err(Error::InvalidPurpose(p)); }
        let t = path.as_ref()[1];
        if t != BIP44_COIN_TYPE { return Err(Error::InvalidType(t)); }
        let a = path.as_ref()[2];
        if a  <  0x80000000      { return Err(Error::AccountOutOfBound(a)); }
        let c = path.as_ref()[3];
        if c  >= 0x80000000      { return Err(Error::ChangeOutOfBound(c)); }
        let i = path.as_ref()[4];
        if i  >= 0x80000000      { return Err(Error::IndexOutOfBound(i)); }

        Ok(Addressing {
            account: path.as_ref()[2],
            change:  path.as_ref()[3],
            index:   path.as_ref()[4],
        })
    }

    /// try to generate a new `Addressing` starting from the given
    /// `Addressing`'s index incremented by the given parameter;
    ///
    /// # Example
    ///
    /// ```
    /// use wallet_crypto::bip44::{Addressing, AddrType};
    ///
    /// let addr = Addressing::new(0x80000000, AddrType::External).unwrap();
    ///
    /// let next = addr.incr(32).unwrap().incr(10).unwrap();
    ///
    /// assert!(next.index == 42);
    /// assert!(next.incr(0x80000000).is_err());
    /// ```
    pub fn incr(&self, incr: u32) -> Result<Self> {
        if incr >= 0x80000000 { return Err(Error::IndexOutOfBound(incr)); }
        let mut addr = self.clone();
        addr.index += incr;
        Ok(addr)
    }

    /// generate a sequence of Addressing from the given
    /// addressing as starting point up to the `chunk_size`.
    ///
    /// the function will return as soon as `chunk_size` is reached
    /// or at the first `Error::IndexOutOfBound`.
    ///
    pub fn next_chunks(&self, chunk_size: usize) -> Result<Vec<Self>> {
        let mut v = Vec::with_capacity(chunk_size);
        for i in 0..chunk_size {
            match self.incr(i as u32) {
                Err(Error::IndexOutOfBound(_)) => break,
                Err(err) => return Err(err),
                Ok(r) => v.push(r)
            }
        }
        Ok(v)
    }
}
