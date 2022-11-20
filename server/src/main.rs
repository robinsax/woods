use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{UNIX_EPOCH, SystemTime};

use log::{LevelFilter, info, debug};
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use tokio::task;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};
use futures_util::{StreamExt, SinkExt, TryStreamExt};
use futures_util::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use serde_json;

use common::runtime::{Runtime, RuntimeMessage, RuntimeRole, RuntimeIo};
use common::input::Input;

#[derive(StructOpt, Debug)]
struct CLIOpts {
    #[structopt(long)]
    addr: String
}

struct WsRuntimeIoImpl {
    txs: Option<Vec<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    rx_queue: Option<Vec<RuntimeMessage>>,
    tx_queue: Vec<RuntimeMessage>
}

impl WsRuntimeIoImpl {
    fn new_static() -> &'static Mutex<Self> {
        Box::leak(Box::new(Mutex::new(Self {
            txs: Some(Vec::new()),
            rx_queue: Some(Vec::new()),
            tx_queue: Vec::new()
        })))
    }

    fn push_tx(&mut self, tx: SplitSink<WebSocketStream<TcpStream>, Message>) {
        // TODO: Race with tx retain checks.
        self.txs.as_mut().expect("txs unset").push(tx);
    }

    fn push_recvd(&mut self, data: String) {
        if data == "ping" {
            return;
        }

        if let Ok(message) = serde_json::from_str::<RuntimeMessage>(&data) {
            info!("q rx {:?}", message);

            self.rx_queue.as_mut().expect("input pool unset recv").push(message);
        }
        else {
            info!("client recv invalid {:?}", data);
        }
    }

    async fn io_tick(&mut self) {
        for inbound in self.tx_queue.iter() {
            let binary = serde_json::to_string(&inbound).expect("update ser failed");

            let mut retain_txs = Vec::new();
            let cur_txs = self.txs.take().expect("txs unset");

            for mut tx in cur_txs.into_iter() {
                debug!("tx ws {:?}", binary);

                match tx.send(Message::from(binary.clone())).await {
                    Ok(_) => retain_txs.push(tx),
                    Err(err) => info!("client drop via tx fail {:?}", err)
                };
            }

            self.txs = Some(retain_txs);
        }

        self.tx_queue = Vec::new();
    }

    fn rx(&mut self) -> Vec<RuntimeMessage> {
        let rx_queue = self.rx_queue.take().expect("input pool unset");

        self.rx_queue = Some(Vec::new());

        rx_queue
    }

    fn tx(&mut self, message: RuntimeMessage) {
        debug!("q tx {:?}", message);
        self.tx_queue.push(message);
    }
}

struct WsRuntimeIo {
    inner: &'static Mutex<WsRuntimeIoImpl>
}

impl WsRuntimeIo {
    fn new_static() -> &'static Self {
        Box::leak(Box::new(Self {
            inner: WsRuntimeIoImpl::new_static()
        }))
    }

    async fn io_loop(&self) {
        loop {
            let mut inner_impl = self.inner.lock().expect("poison");

            inner_impl.io_tick().await;

            drop(inner_impl);

            sleep(Duration::from_millis(35)).await;
        }
    }

    fn push_tx(&self, tx: SplitSink<WebSocketStream<TcpStream>, Message>) {
        let mut inner_impl = self.inner.lock().expect("poison");

        inner_impl.push_tx(tx);
    }

    fn push_recvd(&self, message: String) {
        let mut inner_impl = self.inner.lock().expect("poison");

        inner_impl.push_recvd(message);
    }
}

impl RuntimeIo for WsRuntimeIo {
    fn rx(&self) -> (Vec<Input>, Vec<RuntimeMessage>) {
        let mut inner_impl = self.inner.lock().expect("poison");

        (Vec::new(), inner_impl.rx())
    }

    fn tx(&self, message: RuntimeMessage) {
        let mut inner_impl = self.inner.lock().expect("poison");

        inner_impl.tx(message);
    }
}

// TODO: Push based IO.
#[tokio::main]
async fn main() {
    SimpleLogger::new().env().with_level(LevelFilter::Info).init().unwrap();

    let cli_opts = CLIOpts::from_args();

    info!("starting runtime");
    debug!("with options {:?}", cli_opts);

    let io = WsRuntimeIo::new_static();
    let runtime: &'static Mutex<Runtime> = Box::leak(Box::new(Mutex::new(Runtime::new(io, RuntimeRole::Master))));

    let addr = cli_opts.addr.parse::<SocketAddr>().expect("invalid addr");
    let listener = TcpListener::bind(addr).await.expect("bind fail");

    task::spawn(async {
        loop {
            runtime.lock().expect("runtime poison; io tick").io_tick();

            sleep(Duration::from_millis(25)).await;
        }
    });

    task::spawn(async {
        let mut last_t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        loop {
            let cur_t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

            runtime
                .lock().expect("runtime poison; systems tick")
                .systems_tick((cur_t - last_t).as_nanos() as f64 / 1000000000.0);

            last_t = cur_t;

            sleep(Duration::from_millis(15)).await;
        }
    });

    task::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.expect("conn accept fail");
    
            task::spawn(async move {
                let ws = tokio_tungstenite::accept_async(stream)
                    .await
                    .expect("ws bind fail");
    
                let (write, read) = ws.split();

                io.push_tx(write);
    
                let termination = read
                    .try_for_each(move |f| async move {
                        io.push_recvd(f.to_string());
    
                        Ok(())
                    })
                    .await;

                if let Err(err) = termination {
                    info!("client drop via rx {:?}", err);
                }
            });
        }
    });

    io.io_loop().await;
}
