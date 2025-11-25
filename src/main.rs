use clap::Parser;
use color_eyre::eyre::eyre;
use rusty_man_computer::{Args, Command, Computer, ComputerConfig, print_error, value::Value};
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
            let machine_code_array: &[Value; 100] = &machine_code.clone().try_into().map_err(|_| {
                eyre!("Assembled machine code does ({} letterboxes) not fit into memory (100 letterboxes max).", &machine_code.len())
            })?; 
            let mut computer = Computer::new(ComputerConfig {
                // FIXME
                ram: machine_code_array.clone(),
                ..ComputerConfig::default()
            });
            computer.run();
            Ok(())
        }
    }
}
