use clap::Parser;
use rusty_man_computer::{Args, ExecuteLegacy, ComputerConfig, print_error};

fn main() -> () {
    let args = Args::parse();
    let command = args.command.or_else(|| {
        ExecuteLegacy {
            
        }
    })

    println!("Little Man Computer implemented in Rust!");
    if let Err(e) = rusty_man_computer::run(config) {
        print_error(&format!("Application error: {}", e));
    }
}
