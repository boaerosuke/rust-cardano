use wallet_crypto::{cbor, util::{hex}};
use command::{HasCommand};
use clap::{ArgMatches, Arg, SubCommand, App};
use storage;
use storage::{blob, tag, Storage};
use storage::types::{PackHash};
use storage::tag::{HEAD};
use std::time::{SystemTime, Duration};
use blockchain;
use config::{Config};

use protocol::command::*;
use exe_common::{config::{net}, network::{Network}};

pub fn new_network(cfg: &net::Config) -> Network {
    Network::new(cfg.protocol_magic, &cfg.domain.clone())
}

// TODO return BlockHeader not MainBlockHeader
fn network_get_head_header(storage: &Storage, net: &mut Network) -> blockchain::BlockHeader {
    let block_headers = GetBlockHeader::tip().execute(&mut net.0).expect("to get one header at least");
    if block_headers.len() != 1 {
        panic!("get head header return more than 1 header")
    }
    let mbh = block_headers[0].clone();
    tag::write(&storage, &HEAD.to_string(), mbh.get_previous_header().as_ref());
    mbh
}

fn network_get_blocks_headers(net: &mut Network, from: &blockchain::HeaderHash, to: &blockchain::HeaderHash) -> Vec<blockchain::BlockHeader> {
    let mbh = GetBlockHeader::range(&vec![from.clone()], to.clone()).execute(&mut net.0).expect("to get one header at least");
    mbh
}

fn duration_print(d: Duration) -> String {
    format!("{}.{:03} seconds", d.as_secs(), d.subsec_millis())
}

fn find_earliest_epoch(storage: &storage::Storage, minimum_epochid: blockchain::EpochId, start_epochid: blockchain::EpochId)
        -> Option<(blockchain::EpochId, PackHash)> {
    let mut epoch_id = start_epochid;
    loop {
        match tag::read_hash(storage, &tag::get_epoch_tag(epoch_id)) {
            None => {},
            Some(h) => {
                println!("latest known epoch found is {}", epoch_id);
                return Some((epoch_id, h.into_bytes()))
            },
        }

        if epoch_id > minimum_epochid {
            epoch_id -= 1
        } else {
            return None
        }
    }
}

// download a complete epoch and create a new pack with all the blocks
//
// x_start_hash should reference an epoch genesis block, and latest_hash
// should gives the latest known hash of the chain.
fn download_epoch(storage: &storage::Storage, mut net: &mut Network,
                  epoch_id: blockchain::EpochId,
                  x_start_hash: &blockchain::HeaderHash,
                  latest_hash: &blockchain::HeaderHash) -> blockchain::HeaderHash {
    let mut start_hash = x_start_hash.clone();
    let mut found_epoch_boundary = None;
    let mut writer = storage::pack::PackWriter::init(&storage.config);
    let mut previous_headerhash = start_hash.clone();
    let epoch_time_start = SystemTime::now();
    let mut expected_slotid = 0;

    loop {
        println!("  ### slotid={} from={}", expected_slotid, start_hash);
        let metrics = net.read_start();
        let block_headers = network_get_blocks_headers(&mut net, &start_hash, latest_hash);
        let hdr_metrics = net.read_elapsed(&metrics);
        println!("  got {} headers  ( {} )", block_headers.len(), hdr_metrics);

        let mut start = 0;
        let mut end = block_headers.len() - 1;

        // if the earliest block headers we receive has an epoch
        // less than the expected epoch, we just fast skip
        // this set of headers and restart the loop with the
        // latest known hash
        if block_headers[start].get_slotid().epoch < epoch_id {
            start_hash = block_headers[start].compute_hash();
            println!("headers are of previous epochs, fast skip to {}", start_hash);
            continue;
        }

        while end >= start && block_headers[start].get_slotid().epoch > epoch_id {
            start += 1
        }
        while end > start && block_headers[end].get_slotid().epoch < epoch_id {
            end -= 1
        }

        if start > 0 {
            println!("  found next epoch");
            found_epoch_boundary = Some(block_headers[start-1].compute_hash());
        }
        let latest_block = &block_headers[start];
        let first_block = &block_headers[end];

        if first_block.get_previous_header() != previous_headerhash {
            panic!("previous header doesn't match: hash {} slotid {} got {} expected {}",
                   first_block.compute_hash(),
                   first_block.get_slotid(),
                   first_block.get_previous_header(),
                   previous_headerhash)
        }

        let metrics = net.read_start();
        let blocks_raw = GetBlock::from(&first_block.compute_hash(), &latest_block.compute_hash())
                                .execute(&mut net.0)
                                .expect("to get one block at least");
        let blocks_metrics = net.read_elapsed(&metrics);
        println!("  got {} blocks  ( {} )", blocks_raw.len(), blocks_metrics);

        for block_raw in blocks_raw.iter() {
            let block = block_raw.decode().unwrap();
            let hdr = block.get_header();
            let slot = hdr.get_slotid();
            let blockhash = hdr.compute_hash();
            let block_previous_header = hdr.get_previous_header();

            if slot.epoch != epoch_id {
                panic!("trying to append a block of different epoch id {}", slot.epoch)
            }

            if previous_headerhash != block_previous_header {
                panic!("previous header doesn't match: hash {} slotid {} got {} expected {}", blockhash, slot, block_previous_header, previous_headerhash)
            }

            /*
            match last_packed {
                None    => {},
                Some(ref p) => { if p == &blockhash { continue; } else {} },
            }
            */
            if slot.slotid == expected_slotid {
                expected_slotid += 1
            } else {
                println!("  WARNING: not contiguous. slot id {} found, expected {}", slot.slotid, expected_slotid);
                expected_slotid = slot.slotid + 1
            }

            writer.append(&storage::types::header_to_blockhash(&blockhash), block_raw.as_ref());
            previous_headerhash = blockhash.clone();
        }
        // println!("packing {}", slot);
        start_hash = previous_headerhash.clone();
        /*
        match last_packed {
            None    => {},
            Some(ref p) => { start_hash = p.clone() },
        }
        */

        match found_epoch_boundary {
            None    => {},
            Some(b) => {
                println!("=> packing finished {} slotids", expected_slotid);
                // write packfile
                let (packhash, index) = writer.finalize();
                let (_, tmpfile) = storage::pack::create_index(storage, &index);
                tmpfile.render_permanent(&storage.config.get_index_filepath(&packhash)).unwrap();
                let epoch_time_elapsed = epoch_time_start.elapsed().unwrap();
                println!("=> pack {} written for epoch {} in {}", hex::encode(&packhash[..]), epoch_id, duration_print(epoch_time_elapsed));
                tag::write(storage, &tag::get_epoch_tag(epoch_id), &packhash[..]);
                return b
            },
        }
    }
}

fn net_sync_fast(storage: Storage) {
    let netcfg_file = storage.config.get_config_file();
    let net_cfg = net::Config::from_file(&netcfg_file).expect("no network config present");
    let mut net = new_network(&net_cfg);

    //let mut our_tip = tag::read_hash(&storage, &"TIP".to_string()).unwrap_or(genesis.clone());

    // recover and print the TIP of the network
    let mbh = network_get_head_header(&storage, &mut net);
    let network_tip = mbh.compute_hash();
    let network_slotid = mbh.get_slotid();

    println!("Configured genesis : {}", net_cfg.genesis);
    println!("Network TIP is     : {}", network_tip);
    println!("Network TIP slotid : {}", network_slotid);

    // start from our tip towards network tip
    /*
    if &network_tip == &our_tip {
        println!("Qapla ! already synchronised");
        return ();
    }
    */

    // find the earliest epoch we know about starting from network_slotid
    let (latest_known_epoch_id, start_hash) = match find_earliest_epoch(&storage, net_cfg.epoch_start, network_slotid.epoch) {
        None => { (net_cfg.epoch_start, net_cfg.genesis) },
        Some((found_epoch_id, packhash)) => { (found_epoch_id + 1, get_last_blockid(&storage.config, &packhash).unwrap()) }
    };
    println!("latest known epoch {} hash={}", latest_known_epoch_id, start_hash);

    let mut download_epoch_id = latest_known_epoch_id;
    let mut download_start_hash = start_hash;
    while download_epoch_id < network_slotid.epoch {
        println!("downloading epoch {} {}", download_epoch_id, download_start_hash);
        download_start_hash = download_epoch(&storage, &mut net, download_epoch_id, &download_start_hash, &network_tip);
        download_epoch_id += 1;
    }

}

impl HasCommand for Network {
    type Output = ();
    type Config = ();

    const COMMAND : &'static str = "network";

    fn clap_options<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.about("blockchain network operation")
            .subcommand(SubCommand::with_name("new")
                .about("create a new network, networks can be shared between wallets and work independently.")
                .arg(Arg::with_name("template")
                        .long("template").help("the template for the new network").required(false)
                        .possible_values(&["mainnet", "testnet"]).default_value("mainnet"))
                .arg(Arg::with_name("name").help("the network name").index(1).required(true))
            )
            .subcommand(SubCommand::with_name("get-block-header")
                .arg(Arg::with_name("name").help("the network name").index(1).required(true))
                .about("get a given block header")
            )
            .subcommand(SubCommand::with_name("get-block")
                .about("get a given block")
                .arg(Arg::with_name("name").help("the network name").index(1).required(true))
                .arg(Arg::with_name("blockid").help("hexadecimal encoded block id").index(2).required(true))
            )
            .subcommand(SubCommand::with_name("sync")
                .about("get the next block repeatedly")
                .arg(Arg::with_name("name").help("the network name").index(1).required(true))
            )
    }

    fn run(_: Self::Config, args: &ArgMatches) -> Self::Output {
        match args.subcommand() {
            ("new", Some(opts)) => {
                let net_cfg = match value_t!(opts.value_of("template"), String).unwrap().as_str() {
                    "mainnet" => { net::Config::mainnet() },
                    "testnet" => { net::Config::testnet() },
                    _         => {
                        // we do not support custom template yet.
                        // in the mean while the error is handled by clap
                        // (possible_values)
                        panic!("invalid template option")
                    }
                };
                let name = value_t!(opts.value_of("name"), String).unwrap();

                let mut config = Config::default();
                config.network = name;

                let storage_config = config.get_storage_config();
                let _ = Storage::init(&storage_config).unwrap();

                let network_file = storage_config.get_config_file();
                net_cfg.to_file(&network_file)
            },
            ("get-block-header", Some(opts)) => {
                let name = value_t!(opts.value_of("name"), String).unwrap();
                let mut config = Config::default();
                config.network = name;
                let netcfg_file = config.get_storage_config().get_config_file();
                let net_cfg = net::Config::from_file(&netcfg_file).expect("no network config present");
                let mut net = new_network(&net_cfg);
                let storage = config.get_storage().unwrap();
                let mbh = network_get_head_header(&storage, &mut net);
                println!("prv block header: {}", mbh.get_previous_header());
            },
            ("get-block", Some(opts)) => {
                let name = value_t!(opts.value_of("name"), String).unwrap();
                let mut config = Config::default();
                config.network = name;
                let hh_hex = value_t!(opts.value_of("blockid"), String).unwrap();
                let hh_bytes = hex::decode(&hh_hex).unwrap();
                let hh = blockchain::HeaderHash::from_slice(&hh_bytes).expect("blockid invalid");
                let netcfg_file = config.get_storage_config().get_config_file();
                let net_cfg = net::Config::from_file(&netcfg_file).expect("no network config present");
                let mut net = new_network(&net_cfg);
                let mut b = GetBlock::only(&hh).execute(&mut net.0)
                    .expect("to get one block at least");

                let storage = config.get_storage().unwrap();
                blob::write(&storage, hh.bytes(), b[0].as_ref()).unwrap();
            },
            ("sync", Some(opts)) => {
                let name = value_t!(opts.value_of("name"), String).unwrap();
                let mut config = Config::default();
                config.network = name;
                net_sync_fast(config.get_storage().unwrap())
            },
            _ => {
                println!("{}", args.usage());
                ::std::process::exit(1);
            },
        }
    }
}


fn get_last_blockid(storage_config: &storage::config::StorageConfig, packref: &PackHash) -> Option<blockchain::HeaderHash> {
    let mut reader = storage::pack::PackReader::init(&storage_config, packref);
    let mut last_blk_raw = None;

    while let Some(blk_raw) = reader.get_next() {
        last_blk_raw = Some(blk_raw);
    }
    if let Some(blk_raw) = last_blk_raw {
        let blk : blockchain::Block = cbor::decode_from_cbor(&blk_raw[..]).unwrap();
        let hdr = blk.get_header();
        println!("last_blockid: {} {}", hdr.compute_hash(), hdr.get_slotid());
        Some(hdr.compute_hash())
    } else {
        None
    }
}