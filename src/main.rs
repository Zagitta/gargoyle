use chrono::{DateTime, Utc};
use fixed::traits::LossyInto;
use std::collections::HashSet;
use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::sync::RwLock;
use tracing::metadata::LevelFilter;
use twsapi::core::contract::Contract;

use gargoyle::tws;
use tracing::{debug, error, info, instrument, warn, Level};
use tracing_subscriber::{self, EnvFilter};

//use futures::stream::StreamExt;
use tws::{
    client::ClientImpl,
    messages::{
        BarSize, HistoricalDataType, TWSIncommingMessage, TWSIncommingMessageImpl, TickType,
    },
};

use rillrate::{Counter, Gauge, Histogram, Logger, Pulse, RillRate};

struct RillrateData {
    counter: Counter,
    bid: Pulse,
    bid_c: Gauge,
    ask: Pulse,
    ask_c: Gauge,
    bid_size: Pulse,
    ask_size: Pulse,
    spread: Pulse,
}

impl RillrateData {
    fn new(name: &str) -> RillrateData {
        RillrateData {
            counter: Counter::create(format!("{}.ticks", name).as_str()).unwrap(),
            bid: Pulse::create(format!("{}.bid", name).as_str(), Some(5000)).unwrap(),
            bid_c: Gauge::create(format!("{}.bid_c", name), 0.0, 1000.0).unwrap(),
            ask: Pulse::create(format!("{}.ask", name).as_str(), Some(5000)).unwrap(),
            ask_c: Gauge::create(format!("{}.ask_c", name), 0.0, 1000.0).unwrap(),
            bid_size: Pulse::create(format!("{}.bid_size", name).as_str(), Some(5000)).unwrap(),
            ask_size: Pulse::create(format!("{}.ask_size", name).as_str(), Some(5000)).unwrap(),
            spread: Pulse::create(format!("{}.spread", name).as_str(), Some(5000)).unwrap(),
        }
    }
}

fn handle_msg(ent: &RillrateData, msg: &TWSIncommingMessageImpl) {
    ent.counter.inc(1.0);
    match *msg {
        TWSIncommingMessageImpl::TickPrice {
            price, tick_type, ..
        } => {
            match tick_type {
                TickType::Bid => {
                    //bid = price;
                    ent.bid.set(price.lossy_into());
                    ent.bid_c.set(price.lossy_into());
                }
                TickType::Ask => {
                    //ask = price.lossy_into();
                    ent.ask.set(price.lossy_into());
                    ent.ask_c.set(price.lossy_into());
                }
                _ => {}
            }
            //ent.spread.set(ask - bid);
        }
        TWSIncommingMessageImpl::TickSize {
            tick_type, size, ..
        } => match tick_type {
            TickType::BidSize => ent.bid_size.set(size.into()),
            TickType::AskSize => ent.ask_size.set(size.into()),
            _ => {}
        },
        _ => {}
    }
}

fn spawn_market_req_data(
    ic: Arc<ClientImpl>,
    im: Arc<RwLock<HashMap<i32, RillrateData>>>,
    contract: Contract,
) {
    let name = format!(
        "{}.{}.{}{}",
        contract.symbol,
        contract.last_trade_date_or_contract_month,
        contract.strike,
        contract.right
    );
    tokio::spawn(async move {
        let mut reader = ic.req_market_data(contract).await.unwrap();

        while let Some(msg) = reader.recv().await {
            let msg = msg.get_msg();
            match &msg {
                e @ &TWSIncommingMessageImpl::Error { .. } => println!("{:?}", e),
                _ => {}
            }

            if let Some(req_id) = msg.get_req_id() {
                let mut map = im.write().await;
                let ent = map
                    .entry(req_id)
                    .or_insert_with(|| RillrateData::new(name.as_str()));
                handle_msg(ent, msg);
            }
        }
    });
}

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .with_env_filter(
                EnvFilter::try_from_default_env()?
                    .add_directive("gargoyle::tws::codec[run{self}]=info".parse()?),
            )
            //.pretty()
            .finish(),
    )?;
    let _rillrate = RillRate::from_env("gargoly")?;

    info!("Connecting to TWS");

    let client = Arc::new(ClientImpl::new("127.0.0.1:4001").await?);

    info!("Connected...");
    let jh = {
        let ic = client.clone();
        tokio::spawn(async move {
            let _wtf = ic.run().await;
        })
    };

    let map = Arc::new(RwLock::new(HashMap::new()));

    /* {
        let ic = client.clone();
        let mut contract = Contract::default();
        contract.symbol = "TSLA".to_owned();
        contract.exchange = "SMART".to_owned();
        contract.currency = "USD".to_owned();
        contract.sec_type = "OPT".to_owned();
        contract.multiplier = "100".to_owned();
        contract.strike = 750.0;
        contract.last_trade_date_or_contract_month = "20210423".to_owned();
        contract.right = "C".to_owned();
        let mut reader = ic
            .req_histogram_data(&contract, true, "3 days")
            .await
            .unwrap();

        if let Some(msg) = reader.recv().await {
            let msg = msg.get_msg();
            println!("{:?}", msg);
        } else {
            println!("failed to request historical data :(");
        }
    } */
    {
        let ic = client.clone();
        let mut contract = Contract::default();
        contract.symbol = "TSLA".to_owned();
        contract.exchange = "SMART".to_owned();
        contract.currency = "USD".to_owned();
        contract.sec_type = "OPT".to_owned();
        contract.multiplier = "100".to_owned();
        contract.strike = 610.0;
        contract.last_trade_date_or_contract_month = "20210618".to_owned();
        contract.right = "C".to_owned();
        let mut reader = ic
            .req_historical_data(
                &contract,
                "",
                "10 D",
                BarSize::Seconds::<30>,
                HistoricalDataType::Trades,
                true,
                true,
            )
            .await
            .unwrap();

        while let Some(msg) = reader.recv().await {
            let msg = msg.get_msg();
            //println!("{:?}", msg);
        } /* else {
              println!("failed to request historical data :(");
          } */
        println!("Historical data done");
    }
    return Ok(());

    let (strikes, expiries) = {
        let ic = client.clone();
        let mut reader = ic
            .req_sec_def_opt_params("TSLA".to_owned(), "".to_owned(), "STK".to_owned(), 76792991)
            .await
            .unwrap();
        let mut rstrikes = HashSet::new();
        let mut rexp = HashSet::new();
        while let Some(msg) = reader.recv().await {
            let msg = msg.get_msg();
            //println!("{:?}", msg);
            match msg {
                TWSIncommingMessageImpl::SecurityDefinitionOptionParameter {
                    req_id,
                    exchange,
                    underlying_con_id,
                    trading_class,
                    multiplier,
                    expirations,
                    strikes,
                } => {
                    rstrikes.extend(strikes.iter().cloned().map(|s| s.to_owned()));
                    rexp.extend(expirations.iter().cloned().map(|s| s.to_owned()));
                }
                TWSIncommingMessageImpl::SecurityDefinitionOptionParameterEnd { .. } => {
                    break;
                }
                _ => {}
            }
        }
        let mut rstrikes = rstrikes.into_iter().collect::<Vec<_>>();
        rstrikes.sort();
        let mut rexp = rexp.into_iter().collect::<Vec<_>>();
        rexp.sort();
        println!(
            "req_sec_def_opt_params completed with strikes = {:?}, expiries = {:?}",
            rstrikes, rexp
        );
        (rstrikes, rexp)
    };

    let price = 725.0;
    let spread = price * 0.2;
    let upper = price + spread;
    let lower = price - spread;
    let mapped = strikes
        .iter()
        .map(|s| s.as_str().parse::<f64>().unwrap())
        .filter(|s| *s < upper && *s > lower);

    for s in mapped {
        for exp in expiries.iter().take(3) {
            let mut contract = Contract::default();
            contract.symbol = "TSLA".to_owned();
            contract.exchange = "SMART".to_owned();
            contract.currency = "USD".to_owned();
            contract.sec_type = "OPT".to_owned();
            contract.multiplier = "100".to_owned();
            contract.strike = s.into();
            contract.last_trade_date_or_contract_month = exp.to_owned();
            contract.right = "C".to_owned();
            spawn_market_req_data(client.clone(), map.clone(), contract.clone());
            contract.right = "P".to_owned();
            spawn_market_req_data(client.clone(), map.clone(), contract);

            tokio::time::sleep(std::time::Duration::from_millis(200)).await
        }
    }

    /*  {
        let mut contract = Contract::default();
        contract.symbol = "TSLA".to_owned();
        contract.sec_type = "STK".to_owned();
        contract.exchange = "SMART".to_owned();
        contract.currency = "USD".to_owned();
        spawn_market_req_data(client.clone(), map.clone(), contract);
    } */
    /*
    {
        let mut contract = Contract::default();
        contract.symbol = "TSLA".to_owned();
        contract.exchange = "SMART".to_owned();
        contract.currency = "USD".to_owned();
        contract.sec_type = "OPT".to_owned();
        contract.multiplier = "100".to_owned();
        contract.strike = 810.0;
        contract.last_trade_date_or_contract_month = "20210423".to_owned();
        contract.right = "C".to_owned();
        spawn_market_req_data(client.clone(), map.clone(), contract);
    } */

    Ok(jh.await.unwrap())
}
