use rusty_man_computer::value::Value;

fn assemble(program: &str) -> Vec<Value> {
    vec![]
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
        assert_eq!(assemble(program), vec![901, 399, 901, 199, 902, 000])
    }
}
