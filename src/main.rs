use clap::Parser;
use rusty_man_computer::{Args, Config, print_error};

fn main() -> () {
    let config = Config::from_args(Args::parse());

    println!("Little Man Computer implemented in Rust!");
    if let Err(e) = rusty_man_computer::run(config) {
        print_error(&format!("Application error: {}", e));
    }
}
