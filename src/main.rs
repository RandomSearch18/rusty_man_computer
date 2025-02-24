use clap::Parser;
use rusty_man_computer::{print_error, Args, Config};

fn main() -> () {
    let config = Config::from_args(Args::parse());

    println!("Little Man Computer implemented in Rust!");
    if let Err(e) = rusty_man_computer::run(config) {
        print_error(&format!("Application error: {}", e));
    }
}
