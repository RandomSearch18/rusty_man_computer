use std::path::PathBuf;

use rusty_man_computer::{Computer, Config};

#[test]
fn test_ascii_program() {
    let mut computer = Computer::new(Config {
        load_ram_file_path: Some(PathBuf::from("demos/ascii.bin")),
        print_computer_state: false,
        print_raw_output: false,
    });
    computer
        .initialize_ram_from_file()
        .expect("Failed to initialize RAM from file");
    computer.run();
    let output = computer.output.read_all();
    assert_eq!(
        output,
        r##" !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~"##
    );
}
