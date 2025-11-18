use clap::Parser;
use rusty_man_computer::{Args, Command, Computer, ComputerConfig, print_error};
mod assembler;

fn main() -> Result<(), color_eyre::Report> {
    let args = Args::parse();

    match args.command {
        Command::Execute(execute) => {
            let config = ComputerConfig::from_args(execute);
            if let Err(e) = rusty_man_computer::run(config) {
                print_error(&format!("Application error: {}", e));
            };
            Ok(())
        }
        Command::Run { file } => {
            let program = std::fs::read_to_string(file)?;
            let machine_code = assembler::assemble(&program)?;
            let mut computer = Computer::new(ComputerConfig {
                // FIXME
                ram: Some(machine_code),
                ..ComputerConfig::default()
            });
            computer.run();
            Ok(())
        }
    }
}
