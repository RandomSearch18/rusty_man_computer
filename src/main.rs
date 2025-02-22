use std::{env, error::Error, fs, io::Write};

type RAM = [i16; 100];

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

fn print_error(error: &str) {
    println!("{}", color_red(error));
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

fn print_output(output: &String) {
    // Split into "rows" of 3 characters
    let output_vec = output.chars().collect::<Vec<char>>();
    let rows = output_vec.chunks(4);
    // Add pipe characters to separate the rows
    let formatted_output = rows
        .map(|row| bold(&row.iter().collect::<String>()))
        .collect::<Vec<String>>()
        .join(&color_gray("|"));

    println!("{}", formatted_output);
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

fn execute_instruction(ram: &mut RAM, registers: &mut Registers, output: &mut String) -> bool {
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
            println!("BRA: Jumping to address {}", registers.program_counter)
        }
        7 => {
            // BRZ - Branch to the address given if the Accumulator is zero
            if registers.accumulator == 0 {
                registers.program_counter = registers.address_register;
                println!("BRZ: Jumping to address {}", registers.program_counter)
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
                output.push_str(format!("{}", registers.accumulator).as_str());
            }
            if registers.address_register == 22 {
                // OTC - Output accumulator as a character (Non-standard instruction)
                output.push(registers.accumulator as u8 as char);
            }
        }
        _ => {
            panic!("Unhandled opcode: {}", registers.instruction_register);
        }
    }
    true
}

fn clock_cycle(ram: &mut RAM, registers: &mut Registers, output: &mut String) -> bool {
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
    execute_instruction(ram, registers, output)
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

fn main() -> Result<(), Box<dyn Error>> {
    println!("Little Man Computer implemented in Rust!");
    // Array of 100 i16 ints
    let mut ram: RAM = [0; 100];
    // Let's get some registers initialised too
    let mut registers = Registers {
        program_counter: 0,
        instruction_register: 0,
        address_register: 0,
        accumulator: 0,
    };
    let mut output = String::new();

    // If a memory dump (.bin file) has been provided, load it into RAM
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let data = fs::read(filename)?;
        load_data_to_ram(&mut ram, data);
    }

    let mut should_continue = true;
    while should_continue {
        println!();
        print_registers(&registers);
        print_output(&output);
        print_ram(&ram);
        should_continue = clock_cycle(&mut ram, &mut registers, &mut output);
    }

    Ok(())
}
