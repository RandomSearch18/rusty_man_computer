use clap::Parser;
use std::{error::Error, fs, io::Write, path::PathBuf};

type RAM = [i16; 100];

struct OutputConfig {
    immediately_print_output: bool,
}

pub struct Output {
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

    pub fn read_all(&self) -> String {
        self.buffer.clone()
    }
}

struct Registers {
    program_counter: usize,
    instruction_register: i16,
    address_register: usize,
    accumulator: i16,
}

pub struct Computer {
    // Array of 100 i16 ints. Valid values are -999 to 999
    ram: RAM,
    registers: Registers,
    pub output: Output,
    config: Config,
}

impl Computer {
    pub fn new(config: Config) -> Computer {
        Computer {
            ram: [0; 100],
            registers: Registers {
                program_counter: 0,
                instruction_register: 0,
                address_register: 0,
                accumulator: 0,
            },
            output: Output::new(OutputConfig {
                immediately_print_output: config.print_raw_output,
            }),
            config: config,
        }
    }

    /// Initialises RAM with the data from the file provided in the config.
    /// If no file is provided, RAM stays empty (untouched).
    pub fn initialize_ram_from_file(&mut self) -> Result<(), Box<dyn Error>> {
        // If a memory dump (.bin file) has been provided, load it into RAM
        match self.config.load_ram_file_path {
            Some(ref file_path) => {
                let data = fs::read(file_path)?;
                let touched_addresses = self.load_data_to_ram(data);
                println!("Loaded {} data cells into RAM", touched_addresses);
                Ok(())
            }
            None => {
                println!("Initial RAM (.bin) file not provided. RAM will be empty.");
                Ok(())
            }
        }
    }

    /// Returns the number of addresses modified
    pub fn load_data_to_ram(&mut self, data_bytes: Vec<u8>) -> i32 {
        let mut touched_addresses = 0;
        for (i, &byte) in data_bytes.iter().enumerate() {
            if i >= self.ram.len() * 2 {
                break;
            }
            let target_address = i / 2;
            if i % 2 == 0 {
                self.ram[target_address] = (byte as i16) << 8;
                touched_addresses += 1;
            } else {
                self.ram[target_address] += byte as i16;
            }
        }
        touched_addresses
    }

    pub fn clock_cycle(&mut self) -> bool {
        // Stage 1: Fetch
        let ram_index = self.registers.program_counter;
        self.registers.program_counter += 1;

        // Stage 2: Decode
        let instruction = self.ram[ram_index];
        let instruction_code = instruction / 100;
        let instruction_address = instruction % 100;
        self.registers.instruction_register = instruction_code;
        self.registers.address_register = instruction_address as usize;

        // Stage 3: Execute
        self.execute_instruction()
    }

    fn get_input(&mut self) -> i16 {
        match &mut self.config.input {
            Some(input) => {
                if input.is_empty() {
                    panic!("No more input values available");
                }
                input.remove(0)
            }
            None => {
                let prompt = format!("INP: Number input: {}", BOLD);
                read_input_until_valid(&prompt).unwrap_or_else(|_| 0)
            }
        }
    }

    /// Returns `false` if the computer should halt, and `true` otherwise
    fn execute_instruction(&mut self) -> bool {
        match self.registers.instruction_register {
            0 => {
                // HLT - Stop (Little Man has a rest)
                println!("\n{}", bold("Halted!"));
                return false;
            }
            1 => {
                // ADD - Add the contents of the memory address to the Accumulator
                self.registers.accumulator += self.ram[self.registers.address_register];
                apply_overflow(&mut self.registers.accumulator);
            }
            2 => {
                // SUB - Subtract the contents of the memory address from the Accumulator
                self.registers.accumulator -= self.ram[self.registers.address_register];
                apply_overflow(&mut self.registers.accumulator);
            }
            3 => {
                // STA or STO - Store the value in the Accumulator in the memory address given
                self.ram[self.registers.address_register] = self.registers.accumulator;
            }
            4 => {
                // This code is unused and gives an error
                panic!("Opcode 4 is not allowed!");
            }
            5 => {
                // LDA - Load the Accumulator with the contents of the memory address given
                self.registers.accumulator = self.ram[self.registers.address_register];
            }
            6 => {
                // BRA - Branch - use the address given as the address of the next instruction
                self.registers.program_counter = self.registers.address_register;
                if self.config.print_computer_state {
                    println!("BRA: Jumping to address {}", self.registers.program_counter)
                }
            }
            7 => {
                // BRZ - Branch to the address given if the Accumulator is zero
                if self.registers.accumulator == 0 {
                    self.registers.program_counter = self.registers.address_register;
                    if self.config.print_computer_state {
                        println!("BRZ: Jumping to address {}", self.registers.program_counter)
                    }
                }
            }
            8 => {
                // BRP - Branch to the address given if the Accumulator is zero or positive
                if self.registers.accumulator >= 0 {
                    self.registers.program_counter = self.registers.address_register;
                }
            }
            9 => {
                if self.registers.address_register == 1 {
                    // INP - Take from Input
                    self.registers.accumulator = self.get_input();
                }
                if self.registers.address_register == 2 {
                    // OUT - Copy to Output
                    self.output.push_int(self.registers.accumulator);
                }
                if self.registers.address_register == 22 {
                    // OTC - self. accumulator as a character (Non-standard instruction)
                    let character = self.registers.accumulator as u8 as char;
                    self.output.push_char(character);
                }
            }
            _ => {
                panic!("Unhandled opcode: {}", self.registers.instruction_register);
            }
        }
        true
    }

    fn print_registers(&self) {
        println!(
            "PC: {}, Instruction: {}, Addr: {}, Acc: {}",
            bold(&format!("{:02}", self.registers.program_counter)),
            bold(&format!("{:01}", self.registers.instruction_register)),
            bold(&format!("{:02}", self.registers.address_register)),
            bold(&format!("{:03}", self.registers.accumulator))
        );
    }

    fn print_ram(&self) {
        let columns = 10;
        for (i, &cell) in self.ram.iter().enumerate() {
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

    pub fn run(&mut self) {
        let mut should_continue = true;
        while should_continue {
            if self.config.print_computer_state {
                println!();
                self.print_registers();
                self.output.print_on_one_line();
                self.print_ram();
            }
            should_continue = self.clock_cycle();
        }
    }
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

pub struct Config {
    pub load_ram_file_path: Option<PathBuf>,
    /// If the register values, output buffer, RAM values, and branch messages should be printed after every clock cycle
    pub print_computer_state: bool,
    /// If output should be directly and immediately printed when a OUT/OTC instruction is executed
    pub print_raw_output: bool,
    /// Allows specifying input to be given to the emulator programmatically, instead of interactively in the terminal.
    /// You must specify the entire input ahead-of-time.
    /// It is formatted as a vector of integers. Each time the INP instruction is called, the next integer in the vector is used.
    /// Panics if the INP instruction is called after all values have been used.
    /// This feature is most useful when writing tests.
    pub input: Option<Vec<i16>>,
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
            input: None,
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
    let mut computer = Computer::new(config);
    computer.initialize_ram_from_file()?;
    computer.run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_basic_line_wrapping() {
        let mut output = Output::new(OutputConfig {
            immediately_print_output: false,
        });
        output.push_char('a');
        output.push_char('b');
        output.push_char('c');
        output.push_char('d');
        output.push_char('e');
        let lines = output.split_into_lines(4);
        assert_eq!(lines, vec!["abcd", "e"]);
    }

    #[test]
    fn output_numbers_on_separate_lines() {
        let mut output = Output::new(OutputConfig {
            immediately_print_output: false,
        });
        output.push_int(1);
        output.push_int(2);
        output.push_int(3);
        let lines = output.split_into_lines(4);
        assert_eq!(lines, vec!["1", "2", "3"]);
    }

    #[test]
    fn output_mixed_numbers_and_characters() {
        let mut output = Output::new(OutputConfig {
            immediately_print_output: false,
        });
        // Part of an ASCII table
        output.push_int(33);
        output.push_char(' ');
        output.push_char('!');
        output.push_int(34);
        output.push_char(' ');
        output.push_char('"');
        let lines = output.split_into_lines(4);
        assert_eq!(lines, vec!["33 !", "34 \""]);
    }
}
