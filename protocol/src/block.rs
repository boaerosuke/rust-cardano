use std::collections::{LinkedList};
use std::{fmt};
use wallet_crypto::cbor::{ExtendedResult};
use wallet_crypto::{cbor, util, tx};
use wallet_crypto::config::{ProtocolMagic};
use wallet_crypto::hdwallet;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Version {
   major:    u32, 
   minor:    u32, 
   revision: u32, 
}
impl Version {
    pub fn new(major: u32, minor: u32, revision: u32) -> Self {
        Version { major: major, minor: minor, revision: revision }
    }
}
impl Default for Version {
    fn default() -> Self { Version::new(0,1,0) }
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.revision)
    }
}
impl cbor::CborValue for Version {
    fn encode(&self) -> cbor::Value {
        cbor::Value::Array(
            vec![
                cbor::CborValue::encode(&self.major),
                cbor::CborValue::encode(&self.minor),
                cbor::CborValue::encode(&self.revision),
            ]
        )
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, major)    = cbor::array_decode_elem(array, 0).embed("major")?;
            let (array, minor)    = cbor::array_decode_elem(array, 0).embed("minor")?;
            let (array, revision) = cbor::array_decode_elem(array, 0).embed("revision")?;
            if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
            Ok(Version::new(major, minor, revision))
        }).embed("while decoding Version")
    }
}

const HASH_SIZE : usize = 32;

#[derive(Clone)]
pub struct HeaderHash([u8;HASH_SIZE]);
impl AsRef<[u8]> for HeaderHash { fn as_ref(&self) -> &[u8] { self.0.as_ref() } }
impl fmt::Debug for HeaderHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", util::hex::encode(self.as_ref()))
    }
}
impl fmt::Display for HeaderHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", util::hex::encode(self.as_ref()))
    }
}

impl HeaderHash {
    pub fn bytes<'a>(&'a self) -> &'a [u8;HASH_SIZE] { &self.0 }
    pub fn from_bytes(bytes :[u8;HASH_SIZE]) -> Self { HeaderHash(bytes) }
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != HASH_SIZE { return None; }
        let mut buf = [0;HASH_SIZE];

        buf[0..HASH_SIZE].clone_from_slice(bytes);
        Some(Self::from_bytes(buf))
    }
}
impl cbor::CborValue for HeaderHash {
    fn encode(&self) -> cbor::Value { cbor::Value::Bytes(cbor::Bytes::from_slice(self.as_ref())) }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.bytes().and_then(|bytes| {
            match Self::from_slice(bytes.as_ref()) {
                Some(digest) => Ok(digest),
                None         => {
                    cbor::Result::bytes(bytes, cbor::Error::InvalidSize(HASH_SIZE))
                }
            }
        }).embed("while decoding Hash")
    }
}

#[derive(Debug)]
pub struct MainBlockHeader {
    pub protocol_magic: ProtocolMagic,
    pub previous_header: HeaderHash,
    pub body_proof: BodyProof,
    pub consensus: main::Consensus,
    pub extra_data: HeaderExtraData
}
impl fmt::Display for MainBlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!( f
              , "Magic: 0x{:?} Previous Header: {}"
              , self.protocol_magic
              , self.previous_header
              )
    }
}
impl MainBlockHeader {
   pub fn new(pm: ProtocolMagic, pb: HeaderHash, bp: BodyProof, c: main::Consensus, ed: HeaderExtraData) -> Self {
        MainBlockHeader {
            protocol_magic: pm,
            previous_header: pb,
            body_proof: bp,
            consensus: c,
            extra_data: ed
        }
   }
}
impl cbor::CborValue for MainBlockHeader {
    fn encode(&self) -> cbor::Value {
        unimplemented!()
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, p_magic)    = cbor::array_decode_elem(array, 0).embed("protocol magic")?;
            let (array, prv_header) = cbor::array_decode_elem(array, 0).embed("Previous Header Hash")?;
            let (array, body_proof) = cbor::array_decode_elem(array, 0).embed("body proof")?;
            let (array, consensus)  = cbor::array_decode_elem(array, 0).embed("consensus")?;
            let (array, extra_data) = cbor::array_decode_elem(array, 0).embed("extra_data")?;
            if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
            Ok(MainBlockHeader::new(p_magic, prv_header, body_proof, consensus, extra_data))
        }).embed("While decoding a MainBlockHeader")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct BlockVersion(u16, u16, u8);
impl BlockVersion {
    pub fn new(major: u16, minor: u16, revision: u8) -> Self {
        BlockVersion(major, minor, revision)
    }
}
impl fmt::Debug for BlockVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}
impl fmt::Display for BlockVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Default for BlockVersion {
    fn default() -> Self { BlockVersion::new(0,1,0) }
}
impl cbor::CborValue for BlockVersion {
    fn encode(&self) -> cbor::Value {
        cbor::Value::Array(
            vec![
                cbor::CborValue::encode(&self.0),
                cbor::CborValue::encode(&self.1),
                cbor::CborValue::encode(&self.2),
            ]
        )
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, major)    = cbor::array_decode_elem(array, 0).embed("major")?;
            let (array, minor)    = cbor::array_decode_elem(array, 0).embed("minor")?;
            let (array, revision) = cbor::array_decode_elem(array, 0).embed("revision")?;
            if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
            Ok(BlockVersion::new(major, minor, revision))
        }).embed("While decoding a BlockVersion")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SoftwareVersion {
    application_name: String,
    application_version: u32
}
impl SoftwareVersion {
    pub fn new(name: String, version: u32) -> Self {
        SoftwareVersion {
            application_name: name,
            application_version: version
        }
    }
}
impl Default for SoftwareVersion {
    fn default() -> Self {
        SoftwareVersion::new(
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap()
        )
    }
}
impl cbor::CborValue for SoftwareVersion {
    fn encode(&self) -> cbor::Value {
        cbor::Value::Array(
            vec![
                cbor::CborValue::encode(&self.application_name),
                cbor::CborValue::encode(&self.application_version),
            ]
        )
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, name)    = cbor::array_decode_elem(array, 0).embed("name")?;
            let (array, version) = cbor::array_decode_elem(array, 0).embed("version")?;
            if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
            Ok(SoftwareVersion::new(name, version))
        }).embed("While decoding a SoftwareVersion")
    }
}

#[derive(Debug)]
pub struct BlockHeaderAttributes(cbor::Value);
impl cbor::CborValue for BlockHeaderAttributes {
    fn encode(&self) -> cbor::Value {
        self.0.clone()
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        Ok(BlockHeaderAttributes(value))
    }
}

#[derive(Debug)]
pub struct HeaderExtraData {
    pub block_version: BlockVersion,
    pub software_version: SoftwareVersion,
    pub attributes: BlockHeaderAttributes,
    pub extra_data_proof: tx::Hash // hash of the Extra body data
}
impl HeaderExtraData {
    pub fn new(block_version: BlockVersion, software_version: SoftwareVersion, attributes: BlockHeaderAttributes, extra_data_proof: tx::Hash) -> Self {
        HeaderExtraData {
            block_version: block_version,
            software_version: software_version,
            attributes: attributes,
            extra_data_proof: extra_data_proof
        }
    }
}
impl cbor::CborValue for HeaderExtraData {
    fn encode(&self) -> cbor::Value {
        unimplemented!()
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, block_version)    = cbor::array_decode_elem(array, 0).embed("block version")?;
            let (array, software_version) = cbor::array_decode_elem(array, 0).embed("software version")?;
            let (array, attributes)       = cbor::array_decode_elem(array, 0).embed("attributes")?;
            let (array, extra_data_proof) = cbor::array_decode_elem(array, 0).embed("extra data proof")?;
            if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
            Ok(HeaderExtraData::new(block_version, software_version, attributes, extra_data_proof))
        }).embed("While decoding a MainBlockHeader")
    }
}

#[derive(Debug)]
pub enum BlockHeader {
    // Todo: GenesisBlockHeader
    MainBlockHeader(MainBlockHeader)
}
impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &BlockHeader::MainBlockHeader(ref mbh) => {
                write!(f, "{}", mbh)
            }
        }
    }
}

impl cbor::CborValue for BlockHeader {
    fn encode(&self) -> cbor::Value {
        match self {
            &BlockHeader::MainBlockHeader(ref mbh) => {
                cbor::Value::Array(
                   vec![cbor::Value::U64(1), cbor::CborValue::encode(mbh)]
                )
            }
        }
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, code)  = cbor::array_decode_elem(array, 0).embed("enumeration code")?;
            if code == 1u64 {
                let (array, mbh) = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(BlockHeader::MainBlockHeader(mbh))
            } else {
                cbor::Result::array(array, cbor::Error::InvalidSumtype(code))
            }
        })
    }
}

pub mod main {
    use super::*;
    use wallet_crypto::{tx, cbor};
    use std::{fmt};
    use std::collections::linked_list::{Iter};

    #[derive(Debug)]
    pub struct TxPayload {
        txaux: LinkedList<tx::TxAux>
    }
    impl fmt::Display for TxPayload {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.txaux.is_empty() {
                return write!(f, "<no transactions>");
            }
            for txaux in self.txaux.iter() {
                writeln!(f, "{}", txaux)?;
            }
            write!(f, "")
        }
    }
    impl TxPayload {
        pub fn new(txaux: LinkedList<tx::TxAux>) -> Self {
            TxPayload { txaux: txaux }
        }
        pub fn empty() -> Self {
            TxPayload::new(LinkedList::new())
        }
        pub fn iter(&self) -> Iter<tx::TxAux> { self.txaux.iter() }
    }
    impl cbor::CborValue for TxPayload {
        fn encode(&self) -> cbor::Value {
            unimplemented!()
        }
        fn decode(value: cbor::Value) -> cbor::Result<Self> {
            value.iarray().and_then(|array| {
                let mut l = LinkedList::new();
                for i in array {
                    l.push_back(cbor::CborValue::decode(i)?);
                }
                Ok(TxPayload::new(l))
            }).embed("While decoding TxPayload")
        }
    }

    #[derive(Debug)]
    pub struct Body {
        pub tx: TxPayload,
        pub scc: cbor::Value,
        pub delegation: cbor::Value,
        pub update: cbor::Value
    }
    impl Body {
        pub fn new(tx: TxPayload, scc: cbor::Value, dlg: cbor::Value, upd: cbor::Value) -> Self {
            Body { tx: tx, scc: scc, delegation: dlg, update: upd }
        }
    }
    impl fmt::Display for Body {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.tx)
        }
    }
    impl cbor::CborValue for Body {
        fn encode(&self) -> cbor::Value {
            unimplemented!()
        }
        fn decode(value: cbor::Value) -> cbor::Result<Self> {
            value.array().and_then(|array| {
                let (array, tx)  = cbor::array_decode_elem(array, 0).embed("tx")?;
                let (array, scc) = cbor::array_decode_elem(array, 0).embed("scc")?;
                let (array, dlg) = cbor::array_decode_elem(array, 0).embed("dlg")?;
                let (array, upd) = cbor::array_decode_elem(array, 0).embed("update")?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(Body::new(tx, scc, dlg, upd))
            }).embed("While decoding Body")
        }
    }

    #[derive(Debug)]
    pub struct Block {
        pub header: MainBlockHeader,
        pub body: Body,
        pub extra: cbor::Value
    }
    impl Block {
        pub fn new(h: MainBlockHeader, b: Body, e: cbor::Value) -> Self {
            Block { header: h, body: b, extra: e }
        }
    }
    impl fmt::Display for Block {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            writeln!(f, "{}", self.header)?;
            write!(f, "{}", self.body)
        }
    }
    impl cbor::CborValue for Block {
        fn encode(&self) -> cbor::Value {
            unimplemented!()
        }
        fn decode(value: cbor::Value) -> cbor::Result<Self> {
            value.array().and_then(|array| {
                let (array, header) = cbor::array_decode_elem(array, 0).embed("header")?;
                let (array, body)   = cbor::array_decode_elem(array, 0).embed("body")?;
                let (array, extra)  = cbor::array_decode_elem(array, 0).embed("extra")?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(Block::new(header, body, extra))
            }).embed("While decoding block::Block")
        }
    }

    #[derive(Debug)]
    pub struct SlotId {
        pub epoch: u32,
        pub slotid: u32,
    }

    impl cbor::CborValue for SlotId {
        fn encode(&self) -> cbor::Value {
            unimplemented!()
        }
        fn decode(value: cbor::Value) -> cbor::Result<Self> {
            value.array().and_then(|array| {
                let (array, epoch) = cbor::array_decode_elem(array, 0).embed("epoch")?;
                let (array, slotid) = cbor::array_decode_elem(array, 0).embed("slotid")?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(SlotId { epoch: epoch, slotid: slotid })
            }).embed("While decoding Slotid")
        }
    }

    type ChainDifficulty = u64;

    type SignData = ();

    #[derive(Debug)]
    pub enum BlockSignature {
        Signature(hdwallet::Signature<SignData>),
        ProxyLight(Vec<cbor::Value>),
        ProxyHeavy(Vec<cbor::Value>),
    }
    impl cbor::CborValue for BlockSignature {
        fn encode(&self) -> cbor::Value {
            unimplemented!()
        }
        fn decode(value: cbor::Value) -> cbor::Result<Self> {
            value.array().and_then(|array| {
                let (array, code)  = cbor::array_decode_elem(array, 0).embed("enumeration code")?;
                match code {
                    0u64 => {
                        let (array, sig) = cbor::array_decode_elem(array,0).embed("")?;
                        if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                        Ok(BlockSignature::Signature(sig))
                    },
                    1u64 => { Ok(BlockSignature::ProxyLight(array)) },
                    2u64 => { Ok(BlockSignature::ProxyHeavy(array)) },
                    _    => { cbor::Result::array(array, cbor::Error::UnparsedValues) },
                }
            }).embed("While decoding main::BlockSignature")
        }
    }

    #[derive(Debug)]
    pub struct Consensus {
        pub slot_id: SlotId,
        pub leader_key: hdwallet::XPub,
        pub chain_difficulty: ChainDifficulty,
        pub block_signature: BlockSignature,
    }
    impl cbor::CborValue for Consensus {
        fn encode(&self) -> cbor::Value {
            unimplemented!()
        }
        fn decode(value: cbor::Value) -> cbor::Result<Self> {
            value.array().and_then(|array| {
                let (array, slotid)  = cbor::array_decode_elem(array, 0).embed("slotid code")?;
                let (array, leaderkey)  = cbor::array_decode_elem(array, 0).embed("leader key")?;
                let (array, chain_difficulty) : (Vec<cbor::Value>, Vec<u64>) = cbor::array_decode_elem(array, 0).embed("chain difficulty")?;
                let (array, block_signature) = cbor::array_decode_elem(array, 0).embed("block signature")?;

                Ok(Consensus {
                    slot_id: slotid,
                    leader_key: leaderkey,
                    chain_difficulty: chain_difficulty[0],
                    block_signature: block_signature,
                })
            }).embed("While decoding main::Consensus")
        }
    }
}

#[derive(Debug)]
pub enum Block {
    MainBlock(main::Block)
}
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Block::MainBlock(ref blk) => write!(f, "{}", blk)
        }
    }
}

impl cbor::CborValue for Block {
    fn encode(&self) -> cbor::Value {
        unimplemented!()
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, code)  = cbor::array_decode_elem(array, 0).embed("enumeration code")?;
            // if code == 0u64 { TODO: support genesis::Block
            if code == 1u64 {
                let (array, blk) = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(Block::MainBlock(blk))
            } else {
                cbor::Result::array(array, cbor::Error::InvalidSumtype(code))
            }
        }).embed("While decoding block::Block")
    }
}

#[derive(Debug)]
pub enum SscProof {
    Commitments(tx::Hash, tx::Hash),
    Openings(tx::Hash, tx::Hash),
    Shares(tx::Hash, tx::Hash),
    Certificate(tx::Hash)
}
impl cbor::CborValue for SscProof {
    fn encode(&self) -> cbor::Value {
        unimplemented!()
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, code)  = cbor::array_decode_elem(array, 0).embed("enumeration code")?;
            if code == 0u64 {
                let (array, commhash) = cbor::array_decode_elem(array, 0)?;
                let (array, vss)      = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(SscProof::Commitments(commhash, vss))
            } else if code == 1u64 {
                let (array, commhash) = cbor::array_decode_elem(array, 0)?;
                let (array, vss)      = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(SscProof::Openings(commhash, vss))
            } else if code == 2u64 {
                let (array, commhash) = cbor::array_decode_elem(array, 0)?;
                let (array, vss)      = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(SscProof::Shares(commhash, vss))
            } else if code == 3u64 {
                let (array, cert)      = cbor::array_decode_elem(array, 0)?;
                if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
                Ok(SscProof::Certificate(cert))
            } else {
                cbor::Result::array(array, cbor::Error::InvalidSumtype(code))
            }
        }).embed("While decoding block::Block")
    }
}

#[derive(Debug)]
pub struct BodyProof {
    pub tx: tx::TxProof,
    pub mpc: SscProof,
    pub proxy_sk: tx::Hash, // delegation hash
    pub update: tx::Hash, // UpdateProof (hash of UpdatePayload)
}
impl BodyProof {
    pub fn new(tx: tx::TxProof, mpc: SscProof, proxy_sk: tx::Hash, update: tx::Hash) -> Self {
        BodyProof {
            tx: tx,
            mpc: mpc,
            proxy_sk: proxy_sk,
            update: update
        }
    }
}
impl cbor::CborValue for BodyProof {
    fn encode(&self) -> cbor::Value {
        unimplemented!()
    }
    fn decode(value: cbor::Value) -> cbor::Result<Self> {
        value.array().and_then(|array| {
            let (array, tx)  = cbor::array_decode_elem(array, 0).embed("tx")?;
            let (array, mpc)  = cbor::array_decode_elem(array, 0).embed("mpc")?;
            let (array, proxy_sk)  = cbor::array_decode_elem(array, 0).embed("proxy_sk")?;
            let (array, update)  = cbor::array_decode_elem(array, 0).embed("update")?;
            if ! array.is_empty() { return cbor::Result::array(array, cbor::Error::UnparsedValues); }
            Ok(BodyProof::new(tx, mpc, proxy_sk, update))
        }).embed("While decoding BodyProof")
    }
}
