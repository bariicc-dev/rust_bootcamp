use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        print_help();
        return;
    }

    let mut file_path = String::new();
    let mut mode = String::new();
    let mut write_data = String::new();
    let mut offset: u64 = 0;
    let mut size: u64 = 0;
    let mut size_set = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-f" | "--file" => {
                if i + 1 < args.len() {
                    file_path = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Error: Missing file path");
                    process::exit(1);
                }
            }
            "-r" | "--read" => {
                mode = "read".to_string();
            }
            "-w" | "--write" => {
                mode = "write".to_string();
                if i + 1 < args.len() {
                    write_data = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Error: Missing hex string to write");
                    process::exit(1);
                }
            }
            "-o" | "--offset" => {
                if i + 1 < args.len() {
                    offset = parse_offset(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("Error: Missing offset value");
                    process::exit(1);
                }
            }
            "-s" | "--size" => {
                if i + 1 < args.len() {
                    size = args[i + 1].parse().expect("Invalid size");
                    size_set = true;
                    i += 1;
                } else {
                    eprintln!("Error: Missing size value");
                    process::exit(1);
                }
            }
            _ => {
                eprintln!("error: Unknown argument {}", arg);
                process::exit(2);
            }
        }
        i += 1;
    }

    if file_path.is_empty() {
        print_help();
        return;
    }

    if mode == "read" {
        do_read(&file_path, offset, size, size_set);
    } else if mode == "write" {
        do_write(&file_path, offset, &write_data);
    } else {
        print_help();
    }
}

fn print_help() {
    println!("Usage: hextool [OPTIONS]");
    println!();
    println!("Read and write binary files in hexadecimal");
    println!();
    println!("Options:");
    println!("-f, --file Target file");
    println!("-r, --read Read mode (display hex)");
    println!("-w, --write Write mode (hex string to write)");
    println!("-o, --offset Offset in bytes (decimal or 0x hex)");
    println!("-s, --size Number of bytes to read");
    println!("-h, --help Print help");
}

fn parse_offset(s: &str) -> u64 {
    if let Some(stripped) = s.strip_prefix("0x") {
        u64::from_str_radix(stripped, 16).expect("Invalid hex offset")
    } else {
        s.parse().expect("Invalid decimal offset")
    }
}

fn do_read(path: &str, offset: u64, size: u64, size_set: bool) {
    let mut file = File::open(path).expect("Failed to open file");

    file.seek(SeekFrom::Start(offset)).expect("Failed to seek");

    let mut buffer = Vec::new();
    if size_set {
        let mut handle = file.take(size);
        handle
            .read_to_end(&mut buffer)
            .expect("Failed to read file");
    } else {
        file.read_to_end(&mut buffer).expect("Failed to read file");
    }

    let mut current_offset = offset;

    for chunk in buffer.chunks(16) {
        print!("{:08x}: ", current_offset);

        for byte in chunk {
            print!("{:02x} ", byte);
        }

        if chunk.len() < 16 {
            for _ in 0..(16 - chunk.len()) {
                print!("   ");
            }
        }

        print!("|");

        for byte in chunk {
            if *byte >= 0x20 && *byte <= 0x7E {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }

        println!("|");
        current_offset += chunk.len() as u64;
    }
}

fn do_write(path: &str, offset: u64, hex_string: &str) {
    let mut bytes = Vec::new();

    for i in (0..hex_string.len()).step_by(2) {
        if i + 2 <= hex_string.len() {
            let byte_str = &hex_string[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16).expect("Invalid hex string");
            bytes.push(byte);
        }
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .expect("Failed to open file");

    file.seek(SeekFrom::Start(offset)).expect("Failed to seek");

    file.write_all(&bytes).expect("Failed to write");
    println!(
        "Successfully written {} bytes at offset 0x{:08x}",
        bytes.len(),
        offset
    );

    print!("Hex: ");
    for (i, b) in bytes.iter().enumerate() {
        print!("{:02x}", b);
        if i < bytes.len() - 1 {
            print!(" ");
        }
    }
    println!();

    print!("ASCII: ");
    for b in &bytes {
        if *b >= 0x20 && *b <= 0x7E {
            print!("{}", *b as char);
        } else {
            print!(".");
        }
    }
    println!();
    println!();
}
