use clap::Parser;
use rusty_man_computer::{Args, Command, ComputerConfig, print_error};

fn main() -> () {
    let args = Args::parse();

    match args.command {
        Command::Execute(execute) => {
            let config = ComputerConfig::from_args(execute);
            if let Err(e) = rusty_man_computer::run(config) {
                print_error(&format!("Application error: {}", e));
            }
        }
    }
}
