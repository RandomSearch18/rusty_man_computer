# Rusty-Man Computer

[![Build and test program](https://github.com/RandomSearch18/rusty_man_computer/actions/workflows/build.yml/badge.svg)](https://github.com/RandomSearch18/rusty_man_computer/actions/workflows/build.yml)

This project is an emulator for Little Man Computer (LMC) ([Wikipedia](https://en.wikipedia.org/wiki/Little_man_computer)), written in Rust. Its behaviour is based on [Peter Higginson's online LMC simulator](https://peterhigginson.co.uk/lmc/).

It contains three tools:

- **`rusty_man_computer`** is the main program. It reads a binary file containing assembled LMC code and runs it.
- **`rmc_assemble`** is an assembler. It takes a file containing LMC assembly code and converts it to a file that can be read by the `rusty_man_computer` emulator (i.e. a Rusty-Man Computer machine code file).
- **`bin_creator`** is a utility that lets you paste in the contents of LMC's memory (from the online simulator), and converts it to a Rusty-Man Computer machine code file.

## Usage

### Reccomended: Pre-compiled binaries

Pre-compiled binaries (executables) are available for Windows and Linux. Download them from the **[releases page](https://github.com/RandomSearch18/rusty_man_computer/releases/latest)**.

You can download an example program from [the `demos` directory](https://github.com/RandomSearch18/rusty_man_computer/tree/master/demos), and run it as below (adjusting the binary name to match the name of the file you've downloaded):

```batch
.\rusty-man-computer-0.4.0-x86_64-pc-windows-gnu.exe --ram add.bin
```

I'd suggest renaming the binary file to `rusty-man-computer` or `rusty-man-computer.exe` (on Windows) to make things easier to type.

### Try it online: Run in CodeSandbox

You can try Rusty-Man Computer in your browser by visiting the **[💻 CodeSandbox demo](https://codesandbox.io/p/github/RandomSearch18/rusty_man_computer/)**.

If you see "Setup in progress" in the top left of the CodeSandbox UI, it's building a microVM image for you. You'll have to wait for this to complete (you can click on it to see progress). You _might_ have to refresh after it's finished building.

Once CodeSandbox has loaded, press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>\`</kbd> to open a new terminal, where you can use `cargo run` to run the [demo programs](#running-the-demo-programs) below :D

### Alternative: Run using Cargo

If you have [Rust installed](https://rust-lang.org/tools/install/), you can run the programs by cloning this repository and using `cargo run`, e.g.

```bash
cargo run --bin rusty_man_computer -- demos/add.bin
```

### Running the demo programs

#### Addition

Credit: Peter L Higginson, <https://peterhigginson.co.uk/lmc/>

> Output the sum of two numbers

```bash
rusty-man-computer --ram demos/add.bin
```

#### Addition and subtraction

Credit: Peter L Higginson, <https://peterhigginson.co.uk/lmc/>

> Input three numbers.
> Output the sum of the first two
> and the third minus the first

```bash
rusty-man-computer --ram demos/add-subtract.bin
```

#### Basic ASCII characters

Credit: Peter L Higginson, <https://peterhigginson.co.uk/lmc/>

```bash
rusty-man-computer --ram demos/ascii.bin
```

#### Basic ASCII character table

Credit: Peter L Higginson, <https://peterhigginson.co.uk/lmc/>

```bash
rusty-man-computer --ram demos/ascii_table.bin
```

#### Factorial

Credit: 101computing.net, <https://www.101computing.net/LMC/>

Computes the factorial of the given input number. Note that above $6!$, the output will be wrong due to 999 being the highest representable number.

```bash
rusty-man-computer --ram demos/factorial.bin
```

### Running your own programs

If you have a program written in LMC assembly, you can assemble it to a Rusty-Man Computer machine code file using the `rmc_assemble` binary.

```bash
rmc_assemble my_program.lmc --output my_program.bin
```

Then you can run the program in a similar way to the demos above, e.g.

```bash
rusty-man-computer --ram my_program.bin
```

### Command-line arguments

- `--ram` specifies a path to a `.bin` file that's used to populate the RAM of the emulator. Essentially, it's the program that you want to run. If you omit this argument, then the emulator will start with empty memory, and not do anything.
- `--output-only` is a flag that disables printing the emulated computer's state every clock cycle. Output is simply printed as it's generated, and input prompts are still displayed.
- `--help` (`-h`) prints the help message, which shows a list of command-line arguments.
- `--version` (`-V`) prints the program name and version.

## Screenshots

### Printing every ASCII character

This screenshot only shows the last few clock cycles, after all the characters have been printed.

Each clock cycle, the contents of the registers are shown on the first line, the next line is the output, and then the contents of memory is printed, formatted in the same way as the online simulator (left to right, and then down).

![Screenshot of some of the output from the emulator running in a terminal](assets/terminal-demo-1.png)

## Features

Rusty-Man Computer shows the whole state of the computer at the end of every clock cycle, including the contents of the registers, all memory values, the output, and if a branch instruction has been called. This verbose output should make it easy to see what the emulator is doing, and to track your code as it runs.

The emulator aims to be 100% compatible with [the Peter Higginson implementation](https://peterhigginson.co.uk/lmc/help_new.html) (LMC v1.5b, as of November 2025). All instructions and behaviour present in v1.5 have been implemented, so programs should run exactly as they do on the online simulator.

## Supported platforms

Currently, I support and offer pre-compiled binaries for the following platforms:

- x86_64 Linux (`x86_64-unknown-linux-gnu`)
- x86_64 Windows (`x86_64-pc-windows-gnu`)
- ARM64 Linux (`aarch64-unknown-linux-gnu`)

I also plan to add pre-compiled binaries for macOS soon:

- x86_64 macOS (`x86_64-apple-darwin`)

If you have a different system, you will probably be able to build the program using the Rust compiler yourself, and it should work fine. If you have any issues, I'd be happy to try to help with compiling, or fix bugs, whatever system you're on.
