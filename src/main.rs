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

            // Initialize memory with the machine code
            let mut machine_code_array: [Value; 100] = [Value::zero(); 100];
            if &machine_code.len() > &machine_code_array.len() {
                return Err(eyre!("Program too large to fit in memory"));
            }
            for i in 0..machine_code.len() {
                machine_code_array[i] = machine_code[i];
            }

            let mut computer = Computer::new(ComputerConfig {
                ram: machine_code_array,
                ..ComputerConfig::default()
            });
            computer.run();
            Ok(())
        }
    }
}
