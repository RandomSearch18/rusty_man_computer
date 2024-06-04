use std::{env, error::Error};

type RAM = [i16; 100];

struct Registers {
    program_counter: usize,
    instruction_register: i16,
    address_register: usize,
    accumulator: i16,
}

fn color_grey(text: &str) -> String {
    format!("\x1b[90m{}\x1b[0m", text)
}

fn bold(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}

fn print_ram(ram: &RAM) {
    let columns = 10;
    for (i, &cell) in ram.iter().enumerate() {
        if cell == 0 {
            // Print in grey
            print!("{} ", color_grey("000"));
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
        bold(&format!("{:03}", registers.instruction_register)),
        bold(&format!("{:02}", registers.address_register)),
        bold(&format!("{:03}", registers.accumulator))
    );
}

fn execute_instruction(ram: &mut RAM, registers: &mut Registers) -> bool {
    match registers.instruction_register {
        0 => {
            // HLT - Stop (Little Man has a rest)
            println!("\n{}", bold("Halted!"));
            return false;
        }
        1 => {
            // ADD - Add the contents of the memory address to the Accumulator
            registers.accumulator += ram[registers.address_register];
        }
        2 => {
            // SUB - Subtract the contents of the memory address from the Accumulator
            registers.accumulator -= ram[registers.address_register];
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
        }
        7 => {
            // BRZ - Branch to the address given if the Accumulator is zero
            if registers.accumulator == 0 {
                registers.program_counter = registers.address_register;
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
                // TODO
            }
            if registers.address_register == 2 {
                // OUT - Copy to Output
                // TODO
            }
            if registers.address_register == 22 {
                // OTC - Output accumulator as a character (Non-standard instruction)
                print!("{}", registers.accumulator as u8 as char);
            }
        }
        _ => {
            panic!("Unhandled opcode: {}", registers.instruction_register);
        }
    }
    true
}

fn clock_cycle(ram: &mut RAM, registers: &mut Registers) -> bool {
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
    execute_instruction(ram, registers)
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

    // If a memory dump (.bin file) has been provided, load it into RAM
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let contents = std::fs::read(filename)?;
        dbg!(&contents);
    }

    let mut should_continue = true;
    while should_continue {
        println!();
        print_registers(&registers);
        print_ram(&ram);
        should_continue = clock_cycle(&mut ram, &mut registers);
    }

    Ok(())
}
