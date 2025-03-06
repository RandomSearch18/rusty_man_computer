use std::collections::HashMap;

use rusty_man_computer::value::Value;

#[derive(Debug)]
enum Opcode {
    HLT,
    ADD,
    SUB,
    STA,
    LDA,
    BRA,
    BRZ,
    BRP,
    INP,
    OUT,
    OTC,
    DAT,
}

#[derive(Debug)]
enum Operand {
    Value(Value),
    Label(String),
}

#[derive(Debug)]
enum Line {
    Empty(),
    Instruction {
        label: Option<String>,
        opcode: Opcode,
        operand: Option<Operand>,
    },
}

#[derive(Debug)]
enum ParseError {
    InvalidOpcode(String),
    OperandOutOfRange(i16),
}

fn parse_opcode(string: &str) -> Option<Opcode> {
    match string {
        "HLT" => Some(Opcode::HLT),
        "ADD" => Some(Opcode::ADD),
        "SUB" => Some(Opcode::SUB),
        "STA" => Some(Opcode::STA),
        "LDA" => Some(Opcode::LDA),
        "BRA" => Some(Opcode::BRA),
        "BRZ" => Some(Opcode::BRZ),
        "BRP" => Some(Opcode::BRP),
        "INP" => Some(Opcode::INP),
        "OUT" => Some(Opcode::OUT),
        "OTC" => Some(Opcode::OTC),
        "DAT" => Some(Opcode::DAT),
        _ => None,
    }
}

fn parse_assembly(program: &str) -> Vec<Result<Line, ParseError>> {
    program
        .lines()
        .map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                return Ok(Line::Empty());
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 0 {
                return Ok(Line::Empty());
            }
            // If the first part isn't a valid opcode, use it as a label
            let first_part_as_opcode = parse_opcode(parts[0]);
            let label = match first_part_as_opcode {
                Some(_) => None,
                None => Some(parts[0].to_string()),
            };
            // If we've already found a valid opcode in the first part, use it
            // Otherwise, try parsing the second part as an opcode
            let opcode = match first_part_as_opcode {
                Some(opcode) => opcode,
                None => {
                    let string = match parts.get(1) {
                        Some(string) => string,
                        // This means there's only one part: there's nothing to label, so it's just an invalid opcode
                        None => return Err(ParseError::InvalidOpcode(parts[0].to_string())),
                    };
                    match parse_opcode(string) {
                        Some(opcode) => opcode,
                        None => return Err(ParseError::InvalidOpcode(string.to_string())),
                    }
                }
            };
            let operand_part = if label.is_some() {
                parts.get(2)
            } else {
                parts.get(1)
            };
            // If the operand is a valid number, parse it as a Value
            // Else, consider it a label
            let operand = match operand_part {
                Some(string) => match string.parse::<i16>() {
                    Ok(value) => Some(Operand::Value(
                        // If the number doesn't fit within a Value, return an OperandOutOfRange error
                        Value::new(value).map_err(|_| ParseError::OperandOutOfRange(value))?,
                    )),
                    Err(_) => Some(Operand::Label(string.to_string())),
                },
                None => None,
            };
            Ok(Line::Instruction {
                label,
                opcode,
                operand,
            })
        })
        .collect()
}

fn generate_label_table(lines: &Vec<Line>) -> HashMap<String, usize> {
    let mut labels: HashMap<String, usize> = HashMap::new();
    for (index, line) in lines.iter().enumerate() {
        match line {
            Line::Instruction { label, .. } => {
                if let Some(label) = label {
                    labels.insert(label.to_string(), index);
                }
            }
            _ => continue,
        }
    }
    labels
}

fn generate_machine_code(lines: Vec<Line>) -> Result<Vec<Value>, &'static str> {
    let mut output: Vec<Value> = Vec::new();
    let labels = generate_label_table(&lines);
    for line in lines {
        match line {
            Line::Instruction {
                opcode, operand, ..
            } => {
                let operand_num: i16 = match operand {
                    // Specifies the address literally
                    Some(Operand::Value(value)) => value.into(),
                    // Specifies a label that corresponds to an address
                    Some(Operand::Label(label)) => match labels.get(&label) {
                        Some(value) => *value as i16,
                        None => return Err("Label not found"),
                    },
                    // If no operand is provided, we use `000`
                    None => 000,
                };
                match opcode {
                    Opcode::HLT => output.push(Value::from(000)),
                    Opcode::ADD => output.push(Value::from_digits(1, operand_num)?),
                    Opcode::SUB => output.push(Value::from_digits(2, operand_num)?),
                    Opcode::STA => output.push(Value::from_digits(3, operand_num)?),
                    Opcode::LDA => output.push(Value::from_digits(5, operand_num)?),
                    Opcode::BRA => output.push(Value::from_digits(6, operand_num)?),
                    Opcode::BRZ => output.push(Value::from_digits(7, operand_num)?),
                    Opcode::BRP => output.push(Value::from_digits(8, operand_num)?),
                    Opcode::INP => output.push(Value::from_digits(9, 01)?),
                    Opcode::OUT => output.push(Value::from_digits(9, 02)?),
                    Opcode::OTC => output.push(Value::from_digits(9, 22)?),
                    Opcode::DAT => {
                        output.push(Value::new(operand_num).map_err(|_| "DAT: Value out of range")?)
                    }
                }
            }
            Line::Empty() => continue,
        }
    }
    Ok(output)
}

#[derive(Debug)]
enum AssemblerError {
    ParseError(ParseError),
    MachineCodeError(&'static str),
}

fn assemble(program: &str) -> Result<Vec<Value>, AssemblerError> {
    let parsed = parse_assembly(program);
    let mut valid_lines: Vec<Line> = Vec::new();
    // Only go forward with non-empty lines, and raise an error if we encounter an invalid line
    for line in parsed {
        match line {
            Ok(line) => match line {
                Line::Empty() => continue,
                Line::Instruction { .. } => valid_lines.push(line),
            },
            Err(error) => return Err(AssemblerError::ParseError(error)),
        }
    }

    match generate_machine_code(valid_lines) {
        Ok(machine_code) => Ok(machine_code),
        Err(error) => Err(AssemblerError::MachineCodeError(error)),
    }
}

fn main() {
    println!("Rusty-Man Computer Assembler");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_add_program() {
        let program = "
        // Outputs sum of two inputs
        INP
        STA 99
        INP
        ADD 99
        OUT
        HLT
        ";
        assert_eq!(
            assemble(program).unwrap(),
            vec![901, 399, 901, 199, 902, 000]
        )
    }
}
