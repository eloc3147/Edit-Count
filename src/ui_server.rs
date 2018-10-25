use iron::Iron;
use iron::Listening;
use staticfile::Static;
use mount::Mount;

use std::env;
use std::error::Error;
use std::path::PathBuf;

fn exe_dir() -> Result<PathBuf, Box<Error>> {
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => return Err(format!("Unable to find executable path: {}", e).into()),
    };
    let static_path = match exe_path.parent() {
        Some(path) => path,
        None => return Err(format!("Unable to find executable directory path").into()),
    };
    return Ok(static_path.into());
}

pub fn new(address: &String) -> Result<Listening, Box<Error>> {
    let static_path = exe_dir()?.join("static");
    
    let mut mount = Mount::new();
    mount.mount("/", Static::new(static_path));
    match Iron::new(mount).http(address) {
        Ok(ok) => Ok(ok),
        Err(e) => Err(format!("Failed to start server: {}",
            e.to_string().to_owned()).into()),
    }
}
