use bytes::{BufMut, BytesMut};
use futures::StreamExt;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt,
};
use std::fmt::Debug;
use std::{
    collections::HashMap,
    error::Error,
    sync::{self, atomic::AtomicI32},
    u32,
};
use sync::atomic;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        RwLock,
    },
};
use tokio_util::codec::Framed;
use twsapi::core::contract::Contract;

use crate::tws::messages::BarSize::ValidBarSize;

use super::{
    codec::{DecodedMessage, TWSCodec},
    messages::{HistoricalDataType, TWSIncommingMessage, TWSOutgoingMessage},
    serde::ser,
};
use bytes::Bytes;
use serde::Serialize;

use tracing::{debug, error, info, instrument, trace, warn};
#[derive(Debug)]
pub struct ClientImpl {
    //framed: Framed<TcpStream, TWSCodec>,
    stream: RwLock<SplitStream<Framed<TcpStream, TWSCodec>>>,
    sink: RwLock<SplitSink<Framed<TcpStream, TWSCodec>, DecodedMessage>>,
    //receiver: UnboundedReceiver<DecodedMessage>,
    //sender: UnboundedSender<DecodedMessage>,
    req_id: AtomicI32,
    map: RwLock<HashMap<i32, UnboundedSender<TWSIncommingMessage>>>,
}
use std::fmt;
/* impl Debug for ClientImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientImpl")
            .field(name, value).finish()
    }
} */

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

impl ClientImpl {
    #[instrument]
    pub async fn new<A: ToSocketAddrs + Debug>(addr: A) -> Result<ClientImpl, Box<dyn Error>> {
        debug!("Creating new client");
        let tcp = TcpStream::connect(addr).await?;
        let mut framed = Framed::new(tcp, TWSCodec::new());
        let mut bytes = BytesMut::with_capacity(20);
        bytes.put(&b"API\0"[..]);
        const VERSION: &[u8] = b"v151..151";
        bytes.put_u32(VERSION.len() as u32);
        bytes.put(&VERSION[..]);
        debug!(?bytes, "Writing init bytes");
        framed.get_mut().write_all(&bytes[..]).await?;

        let (sink, stream) = framed.split();
        let c = ClientImpl {
            sink: RwLock::new(sink),
            stream: RwLock::new(stream),
            req_id: AtomicI32::default(),
            map: RwLock::default(),
        };

        let ok = {
            let mut stream = c.stream.write().await;

            loop {
                if let Some(res) = stream.next().await {
                    match res {
                        Ok(msg) => {
                            if let Some(b) = msg.first() {
                                if *b == "151" {
                                    let _ = c
                                        .sink
                                        .write()
                                        .await
                                        .send(vec![
                                            "71".into(),
                                            "2".into(),
                                            "0".into(),
                                            Bytes::new(),
                                            Bytes::new(),
                                        ])
                                        .await;

                                    break true;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Got error: {:?}", e);
                            break false;
                        }
                    }
                }
            }
        };
        return if ok { Ok(c) } else { Err("foobar".into()) };
    }

    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Started client loop");
        while let Some(res) = self.stream.write().await.next().await {
            match res {
                Ok(msg) => {
                    let body = TWSIncommingMessage::from_decoded_message(msg)?;
                    let msg = body.get_msg();
                    if let Some(req_id) = msg.get_req_id() {
                        if let Some(s) = self.map.read().await.get(&req_id) {
                            s.send(body);
                        } else {
                            warn!(?msg, "Got msg without handler");
                        }
                    } else {
                        trace!(?msg, "Got req without id");
                    }
                }
                Err(e) => error!(?e, "Got error during streaming"),
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn req_market_data(
        &self,
        contract: Contract,
    ) -> Result<UnboundedReceiver<TWSIncommingMessage>, Box<dyn Error + '_>> {
        let (s, r) = tokio::sync::mpsc::unbounded_channel();

        let id = self.req_id.fetch_add(1, atomic::Ordering::Relaxed);
        self.map.write().await.insert(id, s);

        let msg = TWSOutgoingMessage::RequestMarketData {
            req_id: id,
            version: 11,
            contract,
        };

        let mut buf = Vec::with_capacity(1024);
        let _ = ser::to_writer(&msg, &mut buf)?;

        self.sink.write().await.send(vec![Bytes::from(buf)]).await?;

        Ok(r)
    }

    #[instrument(skip(self))]
    pub async fn req_contract_data(
        &self,
        contract: Contract,
    ) -> Result<UnboundedReceiver<TWSIncommingMessage>, Box<dyn Error + '_>> {
        let (s, r) = tokio::sync::mpsc::unbounded_channel();

        let id = self.req_id.fetch_add(1, atomic::Ordering::Relaxed);
        self.map.write().await.insert(id, s);

        debug!("Sending req: {}", id);

        let msg = TWSOutgoingMessage::RequestContractData {
            req_id: id,
            version: 8,
            con_id: contract.con_id,
            symbol: contract.symbol,
            sec_type: contract.sec_type,
            last_trade_date_or_contract_month: contract.last_trade_date_or_contract_month,
            strike: contract.strike,
            right: contract.right,
            multiplier: contract.multiplier,
            exchange: contract.exchange,
            primary_exchange: contract.primary_exchange,
            currency: contract.currency,
            local_symbol: contract.local_symbol,
            trading_class: contract.trading_class,
            include_expired: contract.include_expired,
            sec_id_type: contract.sec_id_type,
            sec_id: contract.sec_id,
        };

        let mut buf = Vec::with_capacity(1024);
        let _ = ser::to_writer(&msg, &mut buf)?;

        self.sink.write().await.send(vec![Bytes::from(buf)]).await?;

        Ok(r)
    }

    #[instrument(skip(self))]
    pub async fn req_sec_def_opt_params(
        &self,
        underlying: String,
        exchange: String,
        underlying_sec_type: String,
        underlying_con_id: i32,
    ) -> Result<UnboundedReceiver<TWSIncommingMessage>, Box<dyn Error + '_>> {
        let (s, r) = tokio::sync::mpsc::unbounded_channel();

        let id = self.req_id.fetch_add(1, atomic::Ordering::Relaxed);
        self.map.write().await.insert(id, s);

        println!("Sending req: {}", id);

        let msg = TWSOutgoingMessage::RequestSecurityDefinitionOptionalParameters {
            req_id: id,
            underlying,
            exchange,
            underlying_sec_type,
            underlying_con_id,
        };

        let mut buf = Vec::with_capacity(1024);
        let _ = ser::to_writer(&msg, &mut buf)?;

        self.sink.write().await.send(vec![Bytes::from(buf)]).await?;

        Ok(r)
    }

    #[instrument(skip(self))]
    pub async fn req_histogram_data(
        &self,
        contract: &Contract,
        use_regular_trading_hours: bool,
        period: &str,
    ) -> Result<UnboundedReceiver<TWSIncommingMessage>, Box<dyn Error + '_>> {
        let (s, r) = tokio::sync::mpsc::unbounded_channel();

        let id = self.req_id.fetch_add(1, atomic::Ordering::Relaxed);
        self.map.write().await.insert(id, s);

        println!("Sending req: {}", id);

        let msg = TWSOutgoingMessage::RequestHistogramData {
            req_id: id,
            con_id: contract.con_id,
            symbol: contract.symbol.clone(),
            sec_type: contract.sec_type.clone(),
            last_trade_date_or_contract_month: contract.last_trade_date_or_contract_month.clone(),
            strike: contract.strike,
            right: contract.right.clone(),
            multiplier: contract.multiplier.clone(),
            exchange: contract.exchange.clone(),
            primary_exchange: contract.primary_exchange.clone(),
            currency: contract.currency.clone(),
            local_symbol: contract.local_symbol.clone(),
            trading_class: contract.trading_class.clone(),
            include_expired: contract.include_expired,
            use_regular_trading_hours,
            period: period.to_owned(),
        };

        let mut buf = Vec::with_capacity(1024);
        let _ = ser::to_writer(&msg, &mut buf)?;

        self.sink.write().await.send(vec![Bytes::from(buf)]).await?;

        Ok(r)
    }

    //reqHistoricalData(int tickerId, Contract contract, string endDateTime, string durationString, string barSizeSetting, string whatToShow, int useRTH, int formatDate, bool keepUpToDate, List<TagValue> chartOptions)
    #[instrument(skip(self))]
    pub async fn req_historical_data<BarSize: ValidBarSize + Debug>(
        &self,
        contract: &Contract,
        end_date_time: &str,
        duration: &str,
        _bar_size: BarSize,
        what_to_show: HistoricalDataType,
        use_regular_trading_hours: bool,
        keep_up_to_date: bool,
    ) -> Result<UnboundedReceiver<TWSIncommingMessage>, Box<dyn Error + '_>> {
        let (s, r) = tokio::sync::mpsc::unbounded_channel();

        let id = self.req_id.fetch_add(1, atomic::Ordering::Relaxed);
        self.map.write().await.insert(id, s);

        debug!(id, "Sending req");

        let msg = TWSOutgoingMessage::RequestHistoricalData {
            req_id: id,
            con_id: contract.con_id,
            symbol: contract.symbol.clone(),
            sec_type: contract.sec_type.clone(),
            last_trade_date_or_contract_month: contract.last_trade_date_or_contract_month.clone(),
            strike: contract.strike,
            right: contract.right.clone(),
            multiplier: contract.multiplier.clone(),
            exchange: contract.exchange.clone(),
            primary_exchange: contract.primary_exchange.clone(),
            currency: contract.currency.clone(),
            local_symbol: contract.local_symbol.clone(),
            trading_class: contract.trading_class.clone(),
            include_expired: contract.include_expired,
            end_date_time: end_date_time.into(),
            bar_size: BarSize::NAME.into(),
            duration: duration.into(),
            use_regular_trading_hours,
            what_to_show: what_to_show.into(),
            format_date: 1,
            keep_up_to_date,
            chart_options: (),
        };

        let mut buf = Vec::with_capacity(1024);
        let _ = ser::to_writer(&msg, &mut buf)?;

        self.sink.write().await.send(vec![Bytes::from(buf)]).await?;

        Ok(r)
    }
}
