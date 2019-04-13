use crate::counter::CounterHandle;
use crate::worker::{Worker, WorkerError, WorkerResult};
use crate::CountUpdateEvent;
use derive_new::new;
use failure::{format_err, Error, ResultExt};
use futures::future::{result, FutureResult};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{stream, Future, Sink, Stream};
use json::{parse, stringify, JsonValue};
use std::fmt::Debug;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use tokio::runtime::TaskExecutor;
use websocket::r#async::Server;
use websocket::server::InvalidConnection;
use websocket::OwnedMessage;

#[derive(new)]
pub struct WebsocketsServer {
    port: u16,
    cue_rx: Receiver<CountUpdateEvent>,
    counter_handle: CounterHandle,
}

impl Worker for WebsocketsServer {
    type W = WebsocketsServer;
    const NAME: &'static str = "Websockets Server";

    fn work(self) -> WorkerResult {
        let mut runtime = tokio::runtime::Builder::new().build()?;
        let reactor = runtime.reactor().clone();
        let executor = runtime.executor();

        // Start HTTP server
        // TODO: CHange this to use the existing HTTP server
        let server = Server::bind(format!("127.0.0.1:{}", self.port), &reactor)
            .context("Unable to bind to websocket port")?;

        let cue_dispatcher = CueDispatcher::new();
        cue_dispatcher.start(self.cue_rx);

        let counter_handle = self.counter_handle;

        // A stream of incoming connections
        let server_f = server
            .incoming()
            // Drop the stream in the case of an error
            .map_err(|InvalidConnection { error, .. }| -> Error { error.into() })
            .for_each(move |(upgrade, addr)| -> WorkerResult {
                println!("Got a connection from: {}", addr);
                // Ensure the correct protocol is being used
                if !upgrade.protocols().iter().any(|s| s == "ec-ws") {
                    spawn_future(upgrade.reject(), "Upgrade Rejection", &executor);
                    return Ok(());
                }

                let response_rx = cue_dispatcher
                    .subscribe()?
                    .map_err(|_| format_err!("Error encountered on response stream"));

                let server_counter_handle = counter_handle.clone();

                let connection_f = upgrade
                    .use_protocol("ec-ws")
                    .accept()
                    .map_err(|e| -> Error { e.into() })
                    .and_then(move |(s, _)| {
                        let (sink, stream) = s.split();
                        let handle = server_counter_handle.clone();

                        stream
                            .take_while(|m| Ok(!m.is_close()))
                            .map_err(|e| -> Error { e.into() })
                            .and_then(move |m| match m {
                                OwnedMessage::Ping(p) => Ok(Some(OwnedMessage::Pong(p))),
                                OwnedMessage::Text(string) => parse_packet(&string, &handle),
                                _ => Ok(None),
                            })
                            .filter_map(|m| m)
                            .select(response_rx)
                            .forward(sink)
                            .and_then(|(_, sink)| {
                                sink.send(OwnedMessage::Close(None))
                                    .map_err(|e| -> Error { e.into() })
                            })
                    });

                spawn_future(connection_f, "Client Status", &executor);
                Ok(())
            });

        Ok(runtime.block_on(server_f)?)
    }
}

#[derive(new)]
struct CueDispatcher {
    #[new(default)]
    subscribers: Arc<Mutex<Vec<UnboundedSender<OwnedMessage>>>>,
}

impl CueDispatcher {
    pub fn start(
        &self,
        cue_rx: Receiver<CountUpdateEvent>,
    ) -> Box<Future<Item = (), Error = Error>> {
        let subs = self.subscribers.clone();

        // Process each incomming CUE
        let f = stream::iter_ok(cue_rx).for_each(move |update| -> FutureResult<(), Error> {
            let json: JsonValue = update.into();
            let message = OwnedMessage::Text(json.dump());
            let mut lock = match subs.lock() {
                Ok(l) => l,
                Err(_) => {
                    return result(Err(WorkerError::new_resource_poisoned(
                        "CueDispatcher.subscribers".to_string(),
                    )
                    .into()));
                }
            };

            // Send the update to all subscribers, dropping any that have hung up
            let mut new_subs = Vec::with_capacity(lock.len());
            for sub in lock.drain(..) {
                if let Ok(s) = sub.send(message.clone()).wait() {
                    new_subs.push(s);
                }
            }

            *lock = new_subs;

            result(Ok(()))
        });

        Box::new(f)
    }

    pub fn subscribe(&self) -> Result<UnboundedReceiver<OwnedMessage>, Error> {
        let (tx, rx) = unbounded();
        self.subscribers
            .lock()
            .or(Err(WorkerError::new_resource_poisoned(
                "CueDispatcher.subscribers".to_string(),
            )))?
            .push(tx);

        Ok(rx)
    }
}

unsafe impl Send for CueDispatcher {}

fn parse_packet(
    payload: &str,
    counter_handle: &CounterHandle,
) -> Result<Option<OwnedMessage>, Error> {
    let message = parse(payload)
        .ok()
        // Unwrap and filter for valid commands
        .and_then(|c| {
            if let JsonValue::Object(object) = c {
                if let Some(command) = object.get("command") {
                    return Some(command.to_string());
                }
            }
            None
        })
        // Match the command
        .and_then(|c| match c.as_str() {
            "fullcount" => Some(OwnedMessage::Text(stringify(
                counter_handle
                    .full_count()
                    .expect("Unable to fetch full count"),
            ))),
            _ => None,
        });

    Ok(message)
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, executor: &TaskExecutor)
where
    F: Future<Item = I, Error = E> + 'static + Send,
    E: Debug,
{
    executor.spawn(
        f.map_err(move |e| println!("{}: '{:?}'", desc, e))
            .map(move |_| println!("{}: Finished.", desc)),
    );
}
