use clap::{Parser, Subcommand};
use std::{error::Error, fs, io::Write, path::PathBuf};
use value::Value;

pub mod value {
    use std::{
        fmt,
        ops::{AddAssign, RangeInclusive, SubAssign},
    };

    /// Represents a value held by one letterbox (memory cell) in the LMC
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Value(i16);

    impl Value {
        pub const MIN: i16 = -999;
        pub const MAX: i16 = 999;
        pub const RANGE: RangeInclusive<i16> = Self::MIN..=Self::MAX;

        pub fn new(value: i16) -> Result<Value, ()> {
            if value < -999 || value > 999 {
                Err(())
            } else {
                Ok(Value(value))
            }
        }

        /// Creates a new `Value` from an `i16`, wrapping around if the value is out of bounds.
        /// This is useful for handling overflow when adding or subtracting values.
        pub fn wrap_overflow(value: i16) -> Value {
            let positive_overflow = value - Self::MAX;
            if positive_overflow > 0 {
                return Value::new((Self::MIN - 1) + positive_overflow)
                    .expect("Out of bounds after overflow handling");
            };
            let negative_overflow = value + Self::MAX;
            if negative_overflow < 0 {
                return Value::new((Self::MAX + 1) + negative_overflow)
                    .expect("Out of bounds after overflow handling");
            };
            Value::new(value).expect("Out of bounds after overflow handling")
        }

        pub fn zero() -> Value {
            Value::new(0).expect("Failed to create zero value")
        }

        pub fn from_digits(first_digit: i16, last_two_digits: i16) -> Result<Value, &'static str> {
            if !(0..=9).contains(&first_digit) {
                return Err("First digit out of range");
            }
            if !(0..=99).contains(&last_two_digits) {
                return Err("Last two digits out of range");
            }
            Value::new(first_digit * 100 + last_two_digits).or(Err("Value out of range"))
        }

        pub fn first_digit(&self) -> i16 {
            self.0 / 100
        }

        pub fn last_two_digits(&self) -> i16 {
            self.0 % 100
        }

        pub fn is_zero(&self) -> bool {
            self.0 == 0
        }

        pub fn is_negative(&self) -> bool {
            self.0 < 0
        }

        pub fn is_positive(&self) -> bool {
            self.0 > 0
        }

        pub fn is_non_negative(&self) -> bool {
            self.0 >= 0
        }

        pub fn to_string(&self) -> String {
            self.0.to_string()
        }

        /// Converts the value into its big-endian byte representation (2 bytes)
        pub fn to_be_bytes(&self) -> [u8; 2] {
            self.0.to_be_bytes()
        }
    }

    impl PartialEq<i16> for Value {
        fn eq(&self, other: &i16) -> bool {
            self.0 == *other
        }
    }

    impl From<Value> for i16 {
        fn from(value: Value) -> i16 {
            value.0
        }
    }

    impl From<Value> for char {
        fn from(value: Value) -> char {
            value.0 as u8 as char
        }
    }

    impl From<i8> for Value {
        fn from(value: i8) -> Value {
            // This is fine because any i8 value will be within -999 to 999
            Value(value as i16)
        }
    }

    impl TryFrom<i16> for Value {
        type Error = &'static str;

        fn try_from(value: i16) -> Result<Self, Self::Error> {
            Value::new(value).or(Err("Value out of range"))
        }
    }

    impl AddAssign for Value {
        fn add_assign(&mut self, other: Value) {
            *self = Value::wrap_overflow(self.0 + other.0);
        }
    }

    impl SubAssign for Value {
        fn sub_assign(&mut self, other: Value) {
            *self = Value::wrap_overflow(self.0 - other.0);
        }
    }

    // Thank you to https://stackoverflow.com/a/77841395/11519302 for showing me how to do this
    impl fmt::Display for Value {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(formatter)
        }
    }
}

type RAM = [Value; 100];

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

    fn push_int(&mut self, integer: Value) {
        // If two numbers are printed in a row, separate them with an newline
        // This seems to be what the online LMC simulator does
        let last_digit_was_number = self.chars().last().unwrap_or(' ').is_numeric();
        if last_digit_was_number {
            self.push_char('\n');
        }
        self.buffer.push_str(&integer.to_string());
        if self.config.immediately_print_output {
            print!("{}", integer.to_string());
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
    instruction_register: i8,
    address_register: usize,
    accumulator: Value,
}

pub struct Computer {
    // Array of 100 i16 ints. Valid values are -999 to 999
    ram: RAM,
    registers: Registers,
    pub output: Output,
    config: ComputerConfig,
}

impl Computer {
    pub fn new(config: ComputerConfig) -> Computer {
        Computer {
            ram: [Value::zero(); 100],
            registers: Registers {
                program_counter: 0,
                instruction_register: 0,
                address_register: 0,
                accumulator: Value::zero(),
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
                self.ram[target_address] = Value::new((byte as i16) << 8).unwrap();
                touched_addresses += 1;
            } else {
                self.ram[target_address] += Value::new(byte as i16).unwrap();
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
        let instruction_code = instruction.first_digit();
        let instruction_address = instruction.last_two_digits();
        self.registers.instruction_register =
            instruction_code.try_into().expect("Opcode out of range");
        self.registers.address_register = instruction_address as usize;

        // Stage 3: Execute
        self.execute_instruction()
    }

    fn get_input(&mut self) -> Value {
        match &mut self.config.input {
            Some(input) => {
                if input.is_empty() {
                    panic!("No more input values available");
                }
                input.remove(0)
            }
            None => {
                let prompt = format!("INP: Number input: {}", BOLD);
                read_input_until_valid(&prompt).unwrap_or_else(|_| Value::zero())
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
            }
            2 => {
                // SUB - Subtract the contents of the memory address from the Accumulator
                self.registers.accumulator -= self.ram[self.registers.address_register];
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
                if self.registers.accumulator.is_zero() {
                    self.registers.program_counter = self.registers.address_register;
                    if self.config.print_computer_state {
                        println!("BRZ: Jumping to address {}", self.registers.program_counter)
                    }
                }
            }
            8 => {
                // BRP - Branch to the address given if the Accumulator is zero or positive
                if self.registers.accumulator.is_non_negative() {
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
                    let character = self.registers.accumulator.into();
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
            if cell.is_zero() {
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

fn read_input() -> Result<Value, ReadInputError> {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => match input.trim().parse() {
            Ok(num) => match Value::new(num) {
                Ok(value) => return Ok(value),
                Err(_) => {
                    print_error("Please input an integer between -999 and 999");
                    return Err(ReadInputError::Validation);
                }
            },
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

fn read_input_until_valid(prompt: &str) -> Result<Value, ()> {
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

pub struct ComputerConfig {
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
    pub input: Option<Vec<Value>>,
}

impl ComputerConfig {
    pub fn from_args(args: ExecuteArgs) -> ComputerConfig {
        if args.ram_legacy.is_some() && args.ram.is_some() {
            print_error("Warning: Ignoring positional argument and using --ram argument instead.");
            print_error("Specifying a RAM file without --ram is no longer recommended.");
        }

        ComputerConfig {
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

impl Default for ComputerConfig {
    fn default() -> Self {
        ComputerConfig {
            load_ram_file_path: None,
            print_computer_state: true,
            print_raw_output: false,
            input: None,
        }
    }
}

#[derive(Parser)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// executes the provided Rusty-Man machine code
    Execute(ExecuteArgs),
}

#[derive(Parser, Clone)]
pub struct ExecuteArgs {
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

pub fn run(config: ComputerConfig) -> Result<(), Box<dyn Error>> {
    let mut computer = Computer::new(config);
    computer.initialize_ram_from_file()?;
    computer.run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hlt_instruction_works() {
        let mut computer = Computer::new(ComputerConfig::default());
        computer.ram[0] = 000.into();
        let has_halted = !computer.clock_cycle();
        // It should halt after the first clock cycle
        assert!(has_halted);
    }

    #[test]
    fn add_instruction_works() {
        // Test 40 + 2 = 42
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 40.into();
        computer.ram[99] = 2.into(); // Operand
        computer.ram[0] = Value::new(199).unwrap(); // Add address 99 to ACC
        computer.clock_cycle();
        assert_eq!(computer.registers.accumulator, 42);
    }

    #[test]
    fn sub_instruction_works() {
        // Test 42 - 2 = 40
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 42.into();
        computer.ram[99] = 2.into(); // Operand
        computer.ram[0] = Value::new(299).unwrap(); // Subtract address 99 from ACC
        computer.clock_cycle();
        assert_eq!(computer.registers.accumulator, 40);
    }

    #[test]
    fn store_instruction_works() {
        // Test storing 42 in address 99
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 42.into();
        computer.ram[0] = Value::new(399).unwrap(); // Store ACC to address 99
        computer.clock_cycle();
        assert_eq!(computer.ram[99], 42);
    }

    #[test]
    fn load_instruction_works() {
        // Test loading 42 from address 99
        let mut computer = Computer::new(ComputerConfig::default());
        computer.ram[99] = 42.into();
        computer.ram[0] = Value::new(599).unwrap(); // Load ACC from address 99
        computer.clock_cycle();
        assert_eq!(computer.registers.accumulator, 42);
    }

    #[test]
    fn branch_instruction_works() {
        // Test branching/jumping to address 42
        let mut computer = Computer::new(ComputerConfig::default());
        computer.ram[0] = Value::new(642).unwrap(); // Branch to address 42
        computer.clock_cycle();
        assert_eq!(computer.registers.program_counter, 42);
    }

    #[test]
    fn branch_zero_instruction_when_zero() {
        // Test BRZ when the accumulator is zero (so it should branch)
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 0.into();
        computer.ram[0] = Value::new(742).unwrap(); // Branch to address 42 if ACC is zero
        computer.clock_cycle();
        assert_eq!(computer.registers.program_counter, 42);
    }

    #[test]
    fn branch_zero_instruction_when_non_zero() {
        // Test BRZ when the accumulator is non-zero (so it should not branch)
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = (-5).into();
        computer.ram[0] = Value::new(742).unwrap(); // Branch to address 42 if ACC is zero
        computer.clock_cycle();
        assert_eq!(computer.registers.program_counter, 1);
    }

    #[test]
    fn branch_positive_instruction_when_positive() {
        // Test BRP when the accumulator is positive (so it should branch)
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 5.into();
        computer.ram[0] = Value::new(842).unwrap(); // Branch to address 42 if ACC is positive
        computer.clock_cycle();
        assert_eq!(computer.registers.program_counter, 42);
    }

    #[test]
    fn branch_positive_instruction_when_zero() {
        // Test BRP when the accumulator is zero (so it should branch)
        // (boundary data)
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 0.into();
        computer.ram[0] = Value::new(842).unwrap(); // Branch to address 42 if ACC is positive
        computer.clock_cycle();
        assert_eq!(computer.registers.program_counter, 42);
    }

    #[test]
    fn branch_positive_instruction_when_negative() {
        // Test BRP when the accumulator is negative (so it should not branch)
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = (-5).into();
        computer.ram[0] = Value::new(842).unwrap(); // Branch to address 42 if ACC is positive
        computer.clock_cycle();
        assert_eq!(computer.registers.program_counter, 1);
    }

    #[test]
    fn input_instruction_works() {
        // Test inputting 21
        let mut computer = Computer::new(ComputerConfig {
            input: Some(vec![21.into()]),
            ..ComputerConfig::default()
        });
        computer.ram[0] = Value::new(901).unwrap();
        computer.clock_cycle();
        assert_eq!(computer.registers.accumulator, 21);
    }

    #[test]
    fn output_instruction_works() {
        // Test outputting 21
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 21.into();
        computer.ram[0] = Value::new(902).unwrap();
        computer.clock_cycle();
        assert_eq!(computer.output.read_all(), "21");
    }

    #[test]
    fn output_character_instruction_works() {
        // Test outputting ASCII value 104 (h)
        let mut computer = Computer::new(ComputerConfig::default());
        computer.registers.accumulator = 104.into();
        computer.ram[0] = Value::new(922).unwrap();
        computer.clock_cycle();
        assert_eq!(computer.output.read_all(), "h");
    }

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
        output.push_int(Value::from(1));
        output.push_int(Value::from(2));
        output.push_int(Value::from(3));
        let lines = output.split_into_lines(4);
        assert_eq!(lines, vec!["1", "2", "3"]);
    }

    #[test]
    fn output_mixed_numbers_and_characters() {
        let mut output = Output::new(OutputConfig {
            immediately_print_output: false,
        });
        // Part of an ASCII table
        output.push_int(Value::from(33));
        output.push_char(' ');
        output.push_char('!');
        output.push_int(Value::from(34));
        output.push_char(' ');
        output.push_char('"');
        let lines = output.split_into_lines(4);
        assert_eq!(lines, vec!["33 !", "34 \""]);
    }

    #[test]
    fn value_works() {
        // Normal data
        Value::new(0).unwrap();
        Value::new(901).unwrap();
        Value::new(-10).unwrap();
        // Boundary data
        Value::new(999).unwrap();
        Value::new(-999).unwrap();
    }

    #[test]
    fn value_equality_check() {
        // We like our zeroes here
        assert!(Value::zero() == 0);
        assert!(Value::new(0).unwrap() == 0);
        assert!(Value::from(0) == Value::new(0).unwrap());
        assert!(Value::zero().is_zero());
        // Let's add in one normal number too
        assert!(Value::new(21).unwrap() == 21);
    }

    #[test]
    fn value_wraps_overflow() {
        // Boundary data
        let mut value = Value::new(999).unwrap();
        value += Value::from(1);
        assert_eq!(value, -999);
    }

    #[test]
    fn value_wraps_underflow() {
        // Boundary data
        let mut value = Value::new(-999).unwrap();
        value -= Value::from(1);
        assert_eq!(value, 999);
    }

    #[test]
    fn value_wraps() {
        // "Normal" data (if we assume that wrapping is the normal behaviour)
        let mut value = Value::new(990).unwrap();
        value += Value::from(21);
        // Checked against Peter Higginson's LMC simulator (wraps to -988)
        assert_eq!(value, -988);
    }

    #[test]
    fn value_to_string() {
        assert_eq!(Value::from(3).to_string(), "3");
        assert_eq!(Value::new(-123).unwrap().to_string(), "-123");
    }

    #[test]
    fn value_disallows_invalid_values() {
        // Boundary data
        assert!(Value::new(1000).is_err());
        assert!(Value::new(-1000).is_err());
        // Invalid data
        assert!(Value::new(2025).is_err());
    }

    #[test]
    fn value_first_and_last_digits() {
        // Testing the functions used to extract operators and operands from instructions
        let instruction = Value::new(599).unwrap();
        assert_eq!(instruction.first_digit(), 5);
        assert_eq!(instruction.last_two_digits(), 99);
        assert_eq!(Value::zero().first_digit(), 0);
        assert_eq!(Value::zero().last_two_digits(), 0);
    }
}
