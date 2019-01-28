use crate::worker::Worker;
use gotham::handler::assets::FileOptions;
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use std::env;
use std::error::Error;
use std::path::PathBuf;

pub struct UIServer {
    address: String,
}

impl UIServer {
    pub fn new(address: String) -> UIServer {
        UIServer { address }
    }
}

impl Worker for UIServer {
    type W = UIServer;
    const NAME: &'static str = "UI Server";

    fn work(self) {
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

        gotham::start(self.address, router);
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
