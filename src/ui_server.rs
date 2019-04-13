pub mod websockets_server;

use self::websockets_server::WebsocketsServer;
use crate::counter::CounterHandle;
use crate::worker::{Worker, WorkerResult};
use crate::CountUpdateEvent;
use derive_new::new;
use gotham::handler::assets::FileOptions;
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

#[derive(new)]
pub struct UIServer {
    web_port: u16,
    ws_port: u16,
    cue_rx: Receiver<CountUpdateEvent>,
    counter_handle: CounterHandle,
}

impl Worker for UIServer {
    type W = UIServer;
    const NAME: &'static str = "UI Server";

    fn work(self) -> WorkerResult {
        // Start Websockets server
        let wss = WebsocketsServer::new(self.ws_port, self.cue_rx, self.counter_handle).start()?;

        let static_path = exe_dir()
            .expect("Unable to find static files directory")
            .join("static");

        let router = build_simple_router(|route| {
            route.get("/").to_file(static_path.join("index.html"));

            route.get("assets/*").to_dir(
                FileOptions::new(static_path.join("assets/"))
                    .with_gzip(true)
                    .build(),
            );
        });

        gotham::start(format!("127.0.0.1:{}", self.web_port), router);
        wss.join()?;

        Ok(())
    }
}

fn exe_dir() -> Result<PathBuf, Box<Error>> {
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => return Err(format!("Unable to find executable path: {}", e).into()),
    };
    let static_path = match exe_path.parent() {
        Some(path) => path,
        None => return Err("Unable to find executable directory path".into()),
    };

    Ok(static_path.into())
}
