use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead},
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a filename to write the binary data to");
        return Ok(());
    }
    let filename = &args[1];

    // Let the user paste in a string
    println!("Paste in the memory data:");
    let mut line = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut line)?;

    // Split the string into a vector i16 signed ints
    let memory_data_items: Vec<i16> = line
        .split_whitespace()
        .map(|s| s.parse::<i16>().unwrap())
        .collect();

    // memory_data_items[0].to_be_bytes()
    let memory_data_bytes: Vec<u8> = memory_data_items
        .iter()
        .flat_map(|&i| i.to_be_bytes().to_vec())
        .collect();

    // Write the memory data to a binary file
    fs::write(filename, memory_data_bytes)?;

    Ok(())
}
