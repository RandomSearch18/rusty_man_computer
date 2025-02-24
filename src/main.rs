use std::env;

use rusty_man_computer::{print_error, Config};

fn main() -> () {
    println!("Little Man Computer implemented in Rust!");
    let args: Vec<String> = env::args().collect();
    let config = Config::from_args(&args);

    if let Err(e) = rusty_man_computer::run(config) {
        print_error(&format!("Application error: {}", e));
    }
}
