fn color_grey(text: &str) -> String {
    format!("\x1b[90m{}\x1b[0m", text)
}

fn print_ram(ram: [i16; 100]) {
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

fn clock_cycle() {

}

fn main() {
    println!("Little Man Computer implemented in Rust!");
    // Array of 100 i16 ints
    let mut ram: [i16; 100] = [0; 100];

    print_ram(ram);
}
