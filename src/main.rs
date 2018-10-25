extern crate iron;
extern crate staticfile;
extern crate mount;

mod ui_server;

fn main() {
    let address = String::from("127.0.0.1:2183");
    match ui_server::new(&address) {
        Ok(_) => println!("Server started at {}", address),
        Err(e) => {
            println!("Error stating server: {:#?}", e.cause().unwrap().to_string());
            std::process::exit(1);
        }
    }
}
