use std::{fmt};
use wallet_crypto::cbor::{ExtendedResult};
use wallet_crypto::{cbor};

use types::HeaderHash;
use genesis;
use normal;

#[derive(Debug)]
pub enum BlockHeader {
    GenesisBlockHeader(genesis::BlockHeader),
    MainBlockHeader(normal::BlockHeader),
}

impl BlockHeader {
    pub fn get_previous_header(&self) -> HeaderHash {
        match self {
            BlockHeader::GenesisBlockHeader(ref blo) => blo.previous_header.clone(),
            BlockHeader::MainBlockHeader(ref blo) => blo.previous_header.clone(),
        }
    }
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &BlockHeader::GenesisBlockHeader(ref mbh) => {
                write!(f, "{}", mbh)
            },
            &BlockHeader::MainBlockHeader(ref mbh) => {
                write!(f, "{}", mbh)
            },
        }
    }
}

#[derive(Debug)]
pub enum Block {
    GenesisBlock(genesis::Block),
    MainBlock(normal::Block),
}
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Block::GenesisBlock(ref blk) => write!(f, "{}", blk),
            &Block::MainBlock(ref blk) => write!(f, "{}", blk)
        }
    }
}

// **************************************************************************
// CBOR implementations
// **************************************************************************

impl cbor::CborValue for Block {
    fn encode(&self) -> cbor::Value {
        unimplemented!()
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, code)  = cbor::array_decode_elem(array, 0).embed("enumeration code")?;
            if code == 0u64 {
                let (array, blk) = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(Block::GenesisBlock(blk))
            } else if code == 1u64 {
                let (array, blk) = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(Block::MainBlock(blk))
            } else {
                cbor::Result::array(array, cbor::Error::InvalidSumtype(code))
            }
        }).embed("While decoding block::Block")
    }
}

impl cbor::CborValue for BlockHeader {
    fn encode(&self) -> cbor::Value {
        match self {
            &BlockHeader::GenesisBlockHeader(ref mbh) => {
                cbor::Value::Array(
                   vec![cbor::Value::U64(0), cbor::CborValue::encode(mbh)]
                )
            },
            &BlockHeader::MainBlockHeader(ref mbh) => {
                cbor::Value::Array(
                   vec![cbor::Value::U64(1), cbor::CborValue::encode(mbh)]
                )
            },
        }
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, code)  = cbor::array_decode_elem(array, 0).embed("enumeration code")?;
            if code == 0u64 {
                let (array, mbh) = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(BlockHeader::GenesisBlockHeader(mbh))
            } else if code == 1u64 {
                let (array, mbh) = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(BlockHeader::MainBlockHeader(mbh))
            } else {
                cbor::Result::array(array, cbor::Error::InvalidSumtype(code))
            }
        })
    }
}

