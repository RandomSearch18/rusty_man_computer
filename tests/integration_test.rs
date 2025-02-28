use std::path::PathBuf;

use rusty_man_computer::{Computer, Config};

#[test]
fn test_ascii_program() {
    let mut computer = Computer::new(Config {
        load_ram_file_path: Some(PathBuf::from("demos/ascii.bin")),
        ..Config::default()
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

#[test]
fn test_add_program() {
    let number_1 = 3;
    let number_2 = -5;
    let expected_output = number_1 + number_2;
    let mut computer = Computer::new(Config {
        load_ram_file_path: Some(PathBuf::from("demos/add.bin")),
        input: Some(vec![number_1.into(), number_2.into()]),
        ..Config::default()
    });
    computer
        .initialize_ram_from_file()
        .expect("Failed to initialize RAM from file");
    computer.run();
    assert_eq!(computer.output.read_all(), expected_output.to_string());
}

#[test]
fn test_add_subtract_program() {
    let number_1 = 10;
    let number_2 = 11;
    let expected_sum = number_1 + number_2;
    let number_3 = 100;
    let expected_difference = number_3 - number_1;
    let mut computer = Computer::new(Config {
        load_ram_file_path: Some(PathBuf::from("demos/add-subtract.bin")),
        input: Some(vec![number_1.into(), number_2.into(), number_3.into()]),
        ..Config::default()
    });
    computer
        .initialize_ram_from_file()
        .expect("Failed to initialize RAM from file");
    computer.run();
    let expected_output = [expected_sum.to_string(), expected_difference.to_string()].join("\n");
    assert_eq!(computer.output.read_all(), expected_output);
}

#[test]
fn test_factorial_program() {
    let input = 6;
    // 6! = 720
    let expected_output = 720;
    let mut computer = Computer::new(Config {
        load_ram_file_path: Some(PathBuf::from("demos/factorial.bin")),
        input: Some(vec![input.into()]),
        ..Config::default()
    });
    computer
        .initialize_ram_from_file()
        .expect("Failed to initialize RAM from file");
    computer.run();
    assert_eq!(computer.output.read_all(), expected_output.to_string());
}
