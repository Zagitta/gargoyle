use ascii::AsAsciiStr;
use atomic::AtomicU64;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::{
    collections::HashMap,
    convert::identity,
    error::Error,
    net::SocketAddr,
    sync::{self, RwLock},
    time::Duration,
    u32,
};
use sync::atomic;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    spawn,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_util::codec::Framed;
use twsapi::core::contract::Contract;

use async_stream::try_stream;

use super::codec::{DecodedMessage, TWSCodec};

pub enum ClientMsg {
    ReqMktData {},
}

pub struct ClientImpl {
    framed: Framed<TcpStream, TWSCodec>,
    receiver: UnboundedReceiver<DecodedMessage>,
    sender: UnboundedSender<DecodedMessage>,
    req_id: AtomicU64,
    map: std::sync::RwLock<HashMap<u64, UnboundedSender<ClientMsg>>>,
}

impl ClientImpl {
    pub async fn new(addr: SocketAddr) -> Result<ClientImpl, Box<dyn Error>> {
        let tcp = TcpStream::connect(addr).await?;
        let (s, r) = unbounded_channel::<DecodedMessage>();
        let mut c = ClientImpl {
            framed: Framed::new(tcp, TWSCodec::new()),
            receiver: r,
            sender: s,
            req_id: AtomicU64::default(),
            map: RwLock::default(),
        };

        let mut bytes = BytesMut::with_capacity(20);
        bytes.put(&b"API\0"[..]);
        const VERSION: &[u8] = b"v151..151";
        bytes.put_u32(VERSION.len() as u32);
        bytes.put(&VERSION[..]);

        c.framed.get_mut().write_all(&bytes[..]).await?;

        //let res = c.framed.next().await?.map(|msg| msg.first()).transpose();

        Err("foo".into())
        /* while let Some(res) = c.framed.next().await {
            match res {
                Ok(msg) => msg.first().map_or_else(Err("Foobar"), |b| {
                    if *b == "151" {
                        Ok(c)
                    } else {
                        Err("foobar")
                    }
                }),
                Err(e) => Err(e),
            }
        } */
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some(res) = self.framed.next().await {
            match res {
                Ok(msg) => {}
                Err(e) => println!("Got err: {:?}", e),
            }
        }
        Ok(())
    }

    pub async fn req_market_data(
        &mut self,
        contract: Contract,
    ) -> Result<UnboundedReceiver<ClientMsg>, Box<dyn Error + '_>> {
        let id = self.req_id.fetch_add(1, atomic::Ordering::AcqRel);

        let (s, r) = tokio::sync::mpsc::unbounded_channel();

        self.map.write().map(|mut map| map.insert(id, s))?;

        self.sender.send(vec![])?;

        Ok(r)
    }
}
