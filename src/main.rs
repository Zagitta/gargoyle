#[macro_use]
extern crate bitflags;

use ascii::AsAsciiStr;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::{collections::HashMap, error::Error, string, sync::Arc};
use std::{thread, time::Duration};
use tokio::{net::TcpStream, sync::Notify};
use twsapi::core::contract::Contract;
mod tws;

use futures::prelude::*;

//use futures::stream::StreamExt;
use serde::Deserialize;
use tokio_util::codec::{Decoder, Framed};
use tws::{
    codec::{DecodedMessage, TWSCodec},
    messages::{TWSIncommingMessage, TickType},
    serde::Deserializer,
};

use rillrate::{Counter, Gauge, Histogram, Logger, Pulse, RillRate};

#[derive(Default, Debug)]
struct MarketDataReq {
    contract: Contract,
}

fn make_message(contract: &Contract) -> DecodedMessage {
    vec![
        contract.con_id.to_string().into(),
        contract.symbol.clone().into(),
        contract.sec_type.clone().into(),
        contract.last_trade_date_or_contract_month.clone().into(),
        contract.strike.to_string().into(),
        contract.right.clone().into(),
        contract.multiplier.clone().into(),
        contract.exchange.clone().into(),
        contract.primary_exchange.clone().into(),
        contract.currency.clone().into(),
        contract.local_symbol.clone().into(),
        contract.trading_class.clone().into(),
        "0".into(),   //ComboLegs
        Bytes::new(), //delta_neutral_contract
        "0".into(),   //generic_tick_list
        "0".into(),   //snapshot
        Bytes::new(), //regulatory_snapshot
        Bytes::new(), //mkt_data_options_str
    ]
}

#[derive(Debug)]
struct Tick {
    timestamp: DateTime<Utc>,
    data: TickData,
}
#[derive(Debug)]
enum TickData {
    Price { bid: bool, value: f64 },
    Size { bid: bool, value: f64 },
    Exchange { bid: bool, value: String },
}

struct RillrateData {
    counter: Counter,
    bid: Pulse,
    ask: Pulse,
    bid_size: Pulse,
    ask_size: Pulse,
    spread: Pulse,
}

impl RillrateData {
    fn new(name: i32) -> RillrateData {
        RillrateData {
            counter: Counter::create(format!("{}_ticks", name).as_str()).unwrap(),
            bid: Pulse::create(format!("{}_bid", name).as_str()).unwrap(),
            ask: Pulse::create(format!("{}_ask", name).as_str()).unwrap(),
            bid_size: Pulse::create(format!("{}_bid_size", name).as_str()).unwrap(),
            ask_size: Pulse::create(format!("{}_ask_size", name).as_str()).unwrap(),
            spread: Pulse::create(format!("{}_spread", name).as_str()).unwrap(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _rillrate = RillRate::from_env("gargoly")?;

    println!("Connecting to TWS");
    // Connect to a peer
    let tcp = TcpStream::connect("127.0.0.1:7496").await?;
    let decoder = TWSCodec::new();
    let f = Framed::new(tcp, decoder);

    println!("Sending API msg");

    let v_100_prefix = "API\0";
    let v_100_version = "v151..151";

    let mut v = Vec::new();
    v.extend_from_slice(v_100_prefix.as_bytes());
    v.extend_from_slice(&(v_100_version.len() as i32).to_be_bytes());
    v.extend_from_slice(v_100_version.as_ascii_str().unwrap().as_bytes());

    println!("{:?}", v);

    f.get_ref().try_write(&v).unwrap();

    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();

    let wait_init = notify2.notified();
    let (mut sink, mut stream) = f.split();

    let jh = tokio::spawn(async move {
        println!("Starting message loop");

        let mut map = HashMap::new();

        while let Some(res) = stream.next().await {
            match res {
                Ok(msg) => {
                    if msg.len() >= 1 && msg[0] == "151" {
                        let _ = notify.notify_waiters();
                    } else {
                        let mut de = Deserializer::from_msg(&msg);
                        match TWSIncommingMessage::deserialize(&mut de) {
                            Ok(msg) => {
                                println!("Decoded: {:?}", msg);

                                if let Some(req_id) = msg.get_req_id() {
                                    println!("req_id = {}", req_id);
                                    let ent = map
                                        .entry(*req_id)
                                        .or_insert_with(|| RillrateData::new(*req_id));

                                    ent.counter.inc(1.0);

                                    match msg {
                                        TWSIncommingMessage::TickPrice {
                                            price, tick_type, ..
                                        } => {
                                            match tick_type {
                                                TickType::Bid => {
                                                    //bid = price;
                                                    ent.bid.set(price);
                                                }
                                                TickType::Ask => {
                                                    //ask = price;
                                                    ent.ask.set(price);
                                                }
                                                _ => {}
                                            }
                                            //ent.spread.set(ask - bid);
                                        }
                                        TWSIncommingMessage::TickSize {
                                            tick_type, size, ..
                                        } => match tick_type {
                                            TickType::BidSize => ent.bid_size.set(size.into()),
                                            TickType::AskSize => ent.ask_size.set(size.into()),
                                            _ => {}
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            _ => println!("Couldn't decode: {:?}", msg),
                        }
                    }
                }
                Err(e) => println!("Got err: {:?}", e),
            }
        }
        println!("Exiting message loop");
    });

    let _ = wait_init.await;
    let mut msg = vec![
        "71".into(),
        "2".into(),
        "0".into(),
        Bytes::new(),
        Bytes::new(),
    ];
    println!("{:?}", msg);
    let _ = sink.send(msg).await;

    thread::sleep(Duration::from_millis(500));
    {
        let mut contract = Contract::default();
        contract.symbol = "LC".to_owned();
        contract.sec_type = "STK".to_owned();
        contract.exchange = "SMART".to_owned();
        contract.currency = "USD".to_owned();

        let ticker_id = "7331";
        msg = vec!["1".into(), "11".into(), ticker_id.into()];
        msg.append(&mut make_message(&contract));

        println!("{:?}", msg);

        let _ = sink.send(msg).await;
    }

    {
        let mut contract = Contract::default();
        contract.symbol = "LC".to_owned();
        contract.exchange = "SMART".to_owned();
        contract.currency = "USD".to_owned();
        contract.sec_type = "OPT".to_owned();
        contract.multiplier = "100".to_owned();
        contract.strike = 20.0;
        contract.last_trade_date_or_contract_month = "20210319".to_owned();
        contract.right = "C".to_owned();

        let ticker_id = "1337";
        msg = vec!["1".into(), "11".into(), ticker_id.into()];
        msg.append(&mut make_message(&contract));

        println!("{:?}", msg);

        let _ = sink.send(msg).await;
    }

    Ok(jh.await.unwrap())
}
