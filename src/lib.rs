use clap::Parser;
use std::{error::Error, fs, io::Write, path::PathBuf};

type RAM = [i16; 100];

struct OutputConfig {
    immediately_print_output: bool,
}

struct Output {
    buffer: String,
    config: OutputConfig,
}

impl Output {
    fn new(config: OutputConfig) -> Output {
        Output {
            buffer: String::new(),
            config,
        }
    }

    fn push_char(&mut self, character: char) {
        self.buffer.push(character);
        if self.config.immediately_print_output {
            print!("{}", character);
        }
    }

    fn push_int(&mut self, integer: i16) {
        // If two numbers are printed in a row, separate them with an newline
        // This seems to be what the online LMC simulator does
        let last_digit_was_number = self.chars().last().unwrap_or(' ').is_numeric();
        if last_digit_was_number {
            self.push_char('\n');
        }
        self.buffer.push_str(format!("{}", integer).as_str());
        if self.config.immediately_print_output {
            print!("{}", integer);
        }
    }

    fn chars(&self) -> std::str::Chars {
        self.buffer.chars()
    }

    /// Splits the output into lines of 4 characters maximum.
    /// Does a line break when 4 characters is reached, or a \n is reached
    fn split_into_lines(&self, max_line_length: isize) -> Vec<String> {
        let chars = self.buffer.chars();
        let mut lines = Vec::<String>::new();
        lines.push(String::new());
        let mut current_row = lines.last_mut().unwrap();
        let mut row_length = 0;
        for char in chars {
            if char == '\n' {
                lines.push(String::new());
                current_row = lines.last_mut().unwrap();
                row_length = 0;
                continue;
            }
            if row_length >= max_line_length {
                lines.push(String::new());
                current_row = lines.last_mut().unwrap();
                row_length = 0;
            }
            // Add our character to the current row
            current_row.push(char);
            row_length += 1;
        }
        lines
    }

    /// Prints the output on one line by separating the output lines with a pipe
    fn print_on_one_line(&self) {
        const LINE_WIDTH: isize = 4;
        let rows = self.split_into_lines(LINE_WIDTH);
        println!("{}", rows.join(&color_gray("|")));
    }
}

struct Registers {
    program_counter: usize,
    instruction_register: i16,
    address_register: usize,
    accumulator: i16,
}

const BOLD: &str = "\x1b[1m";
const GRAY: &str = "\x1b[90m";
const RED: &str = "\x1b[31m";
const FORMAT_END: &str = "\x1b[0m";

/// Wrap the provided test in a grey/gray color code
fn color_gray(text: &str) -> String {
    [GRAY, text, FORMAT_END].concat()
}

/// Wrap the provided test in a grey/gray color code. Used for errors
fn color_red(text: &str) -> String {
    [RED, text, FORMAT_END].concat()
}

fn bold(text: &str) -> String {
    [BOLD, text, FORMAT_END].concat()
}

pub fn print_error(error: &str) {
    eprintln!("{}", color_red(error));
}

fn print_ram(ram: &RAM) {
    let columns = 10;
    for (i, &cell) in ram.iter().enumerate() {
        if cell == 0 {
            // Print in gray
            print!("{} ", color_gray("000"));
        } else {
            print!("{:#03} ", cell);
        }

        if (i + 1) % columns == 0 {
            println!();
        }
    }
}

fn print_registers(registers: &Registers) {
    println!(
        "PC: {}, Instruction: {}, Addr: {}, Acc: {}",
        bold(&format!("{:02}", registers.program_counter)),
        bold(&format!("{:01}", registers.instruction_register)),
        bold(&format!("{:02}", registers.address_register)),
        bold(&format!("{:03}", registers.accumulator))
    );
}

enum ReadInputError {
    Unrecoverable(std::io::Error),
    Validation,
}

fn read_input() -> Result<i16, ReadInputError> {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => match input.trim().parse() {
            Ok(num) => {
                if num >= -999 && num <= 999 {
                    return Ok(num);
                } else {
                    print_error("Please input an integer between -999 and 999");
                    return Err(ReadInputError::Validation);
                }
            }
            Err(_) => {
                print_error("Please input a valid integer between -999 and 999");
                return Err(ReadInputError::Validation);
            }
        },
        Err(error) => {
            print_error("Error: Failed to read input");
            return Err(ReadInputError::Unrecoverable(error));
        }
    }
}

fn read_input_until_valid(prompt: &str) -> Result<i16, ()> {
    loop {
        print!("{}", prompt);
        std::io::stdout().flush().unwrap_or(());
        print!("{}", FORMAT_END);
        match read_input() {
            Ok(num) => return Ok(num),
            Err(ReadInputError::Unrecoverable(_)) => return Err(()),
            Err(ReadInputError::Validation) => continue,
        }
    }
}

fn apply_overflow(integer: &mut i16) {
    let positive_overflow = *integer - 999;
    if positive_overflow > 0 {
        *integer = -1000 + positive_overflow;
    }
    let negative_overflow = *integer + 999;
    if negative_overflow < 0 {
        *integer = 1000 + negative_overflow;
    }
}

fn execute_instruction(
    ram: &mut RAM,
    registers: &mut Registers,
    output: &mut Output,
    config: &Config,
) -> bool {
    match registers.instruction_register {
        0 => {
            // HLT - Stop (Little Man has a rest)
            println!("\n{}", bold("Halted!"));
            return false;
        }
        1 => {
            // ADD - Add the contents of the memory address to the Accumulator
            registers.accumulator += ram[registers.address_register];
            apply_overflow(&mut registers.accumulator);
        }
        2 => {
            // SUB - Subtract the contents of the memory address from the Accumulator
            registers.accumulator -= ram[registers.address_register];
            apply_overflow(&mut registers.accumulator);
        }
        3 => {
            // STA or STO - Store the value in the Accumulator in the memory address given
            ram[registers.address_register] = registers.accumulator;
        }
        4 => {
            // This code is unused and gives an error
            panic!("Opcode 4 is not allowed!");
        }
        5 => {
            // LDA - Load the Accumulator with the contents of the memory address given
            registers.accumulator = ram[registers.address_register];
        }
        6 => {
            // BRA - Branch - use the address given as the address of the next instruction
            registers.program_counter = registers.address_register;
            if config.print_computer_state {
                println!("BRA: Jumping to address {}", registers.program_counter)
            }
        }
        7 => {
            // BRZ - Branch to the address given if the Accumulator is zero
            if registers.accumulator == 0 {
                registers.program_counter = registers.address_register;
                if config.print_computer_state {
                    println!("BRZ: Jumping to address {}", registers.program_counter)
                }
            }
        }
        8 => {
            // BRP - Branch to the address given if the Accumulator is zero or positive
            if registers.accumulator >= 0 {
                registers.program_counter = registers.address_register;
            }
        }
        9 => {
            if registers.address_register == 1 {
                // INP - Take from Input
                let prompt = format!("INP: Number input: {}", BOLD);
                let input_provided = read_input_until_valid(&prompt).unwrap_or_else(|_| 0);
                registers.accumulator = input_provided;
            }
            if registers.address_register == 2 {
                // OUT - Copy to Output
                output.push_int(registers.accumulator);
            }
            if registers.address_register == 22 {
                // OTC - Output accumulator as a character (Non-standard instruction)
                let character = registers.accumulator as u8 as char;
                output.push_char(character);
            }
        }
        _ => {
            panic!("Unhandled opcode: {}", registers.instruction_register);
        }
    }
    true
}

fn clock_cycle(
    ram: &mut RAM,
    registers: &mut Registers,
    output: &mut Output,
    config: &Config,
) -> bool {
    // Stage 1: Fetch
    let ram_index = registers.program_counter;
    registers.program_counter += 1;

    // Stage 2: Decode
    let instruction = ram[ram_index];
    let instruction_code = instruction / 100;
    let instruction_address = instruction % 100;
    registers.instruction_register = instruction_code;
    registers.address_register = instruction_address as usize;

    // Stage 3: Execute
    execute_instruction(ram, registers, output, config)
}

fn load_data_to_ram(ram: &mut RAM, data_bytes: Vec<u8>) {
    let mut touched_addresses = 0;
    for (i, &byte) in data_bytes.iter().enumerate() {
        if i >= ram.len() * 2 {
            break;
        }
        let target_address = i / 2;
        if i % 2 == 0 {
            ram[target_address] = (byte as i16) << 8;
            touched_addresses += 1;
        } else {
            ram[target_address] += byte as i16;
        }
    }
    println!("Loaded data into {} RAM addresses", touched_addresses);
}

pub struct Config {
    pub load_ram_file_path: Option<PathBuf>,
    /// If the register values, output buffer, RAM values, and branch messages should be printed after every clock cycle
    pub print_computer_state: bool,
    /// If output should be directly and immediately printed when a OUT/OTC instruction is executed
    pub print_raw_output: bool,
}

impl Config {
    pub fn from_args(args: Args) -> Config {
        if args.ram_legacy.is_some() && args.ram.is_some() {
            print_error("Warning: Ignoring positional argument and using --ram argument instead.");
            print_error("Specifying a RAM file without --ram is no longer recommended.");
        }

        Config {
            load_ram_file_path: args.ram.or_else(|| {
                eprintln!(
                    "Note: It is recommended to use the --ram argument to specify a RAM file."
                );
                args.ram_legacy
            }),
            print_computer_state: !args.output_only,
            print_raw_output: args.output_only,
        }
    }
}

#[derive(Parser)]
#[command(version)]
pub struct Args {
    // Positional arg for memory file (kept for backwards compatibility)
    #[arg(hide = true)]
    ram_legacy: Option<PathBuf>,
    /// Path to a memory dump (.bin) file to load into RAM
    #[arg(long)]
    ram: Option<PathBuf>,
    /// Only print the output of the LMC, excluding the RAM and register values.
    #[arg(long)]
    output_only: bool,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Array of 100 i16 ints
    let mut ram: RAM = [0; 100];
    // Let's get some registers initialised too
    let mut registers = Registers {
        program_counter: 0,
        instruction_register: 0,
        address_register: 0,
        accumulator: 0,
    };
    let mut output = Output::new(OutputConfig {
        immediately_print_output: config.print_raw_output,
    });

    // If a memory dump (.bin file) has been provided, load it into RAM
    match config.load_ram_file_path {
        Some(ref file_path) => {
            let data = fs::read(file_path)?;
            load_data_to_ram(&mut ram, data);
        }
        None => {
            println!("Initial RAM (.bin) file not provided. RAM will be empty.");
        }
    }

    let mut should_continue = true;
    while should_continue {
        if config.print_computer_state {
            println!();
            print_registers(&registers);
            output.print_on_one_line();
            print_ram(&ram);
        }
        should_continue = clock_cycle(&mut ram, &mut registers, &mut output, &config);
    }

    Ok(())
}
