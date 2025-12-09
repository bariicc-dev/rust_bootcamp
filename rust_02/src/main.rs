use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    process,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = match parse_args(&args) {
        Ok(cfg) => cfg,
        Err(msg) => {
            eprintln!("{msg}");
            print_usage();
            process::exit(1);
        }
    };

    if let Err(err) = run(config) {
        eprintln!("{err}");
        process::exit(1);
    }
}

#[derive(Debug, PartialEq)]
enum Command {
    Read {
        file: String,
        offset: u64,
        size: usize,
    },
    Write {
        file: String,
        offset: u64,
        hex: String,
    },
}

fn parse_args(args: &[String]) -> Result<Command, String> {
    match args {
        [action, file, offset, rest @ ..] if action == "read" => {
            let size = match rest.first() {
                Some(value) => value
                    .parse::<usize>()
                    .map_err(|_| "Size must be a positive integer".to_string())?,
                None => return Err("Missing size for read".to_string()),
            };
            let offset = parse_offset(offset)?;
            Ok(Command::Read {
                file: file.clone(),
                offset,
                size,
            })
        }
        [action, file, offset, hex] if action == "write" => {
            let offset = parse_offset(offset)?;
            Ok(Command::Write {
                file: file.clone(),
                offset,
                hex: hex.clone(),
            })
        }
        _ => Err("Invalid arguments".to_string()),
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Read { file, offset, size } => {
            let bytes = read_bytes(&file, offset, size)?;
            print_hex(&bytes);
        }
        Command::Write { file, offset, hex } => {
            let bytes = parse_hex_bytes(&hex)?;
            write_bytes(&file, offset, &bytes)?;
            println!("Wrote {} bytes", bytes.len());
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  rust02 read <file> <offset> <size>");
    println!("  rust02 write <file> <offset> <hex>");
}

fn parse_offset(text: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .map_err(|_| "Offset must be a non-negative integer".to_string())
}

fn parse_hex_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let cleaned = hex.trim();
    if cleaned.is_empty() {
        return Err("Hex string cannot be empty".to_string());
    }

    if cleaned.len() % 2 != 0 {
        return Err("Hex string must have an even length".to_string());
    }

    let mut bytes = Vec::with_capacity(cleaned.len() / 2);
    for pair in cleaned.as_bytes().chunks(2) {
        let hi = (pair[0] as char)
            .to_digit(16)
            .ok_or_else(|| "Invalid hex string".to_string())?;
        let lo = (pair[1] as char)
            .to_digit(16)
            .ok_or_else(|| "Invalid hex string".to_string())?;
        bytes.push(((hi << 4) | lo) as u8);
    }

    Ok(bytes)
}

fn read_bytes(path: &str, offset: u64, size: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|err| err.to_string())?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|err| err.to_string())?;

    let mut buffer = vec![0u8; size];
    let read = file.read(&mut buffer).map_err(|err| err.to_string())?;
    buffer.truncate(read);
    Ok(buffer)
}

fn write_bytes(path: &str, offset: u64, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .map_err(|err| err.to_string())?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|err| err.to_string())?;
    file.write_all(bytes).map_err(|err| err.to_string())
}

fn print_hex(bytes: &[u8]) {
    let output: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
    println!("{}", output.join(" "));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_read_command() {
        let args = vec![
            "read".to_string(),
            "file.bin".to_string(),
            "4".to_string(),
            "8".to_string(),
        ];
        let parsed = parse_args(&args).unwrap();
        assert_eq!(
            parsed,
            Command::Read {
                file: "file.bin".to_string(),
                offset: 4,
                size: 8,
            }
        );
    }

    #[test]
    fn parses_write_command() {
        let args = vec![
            "write".to_string(),
            "file.bin".to_string(),
            "0".to_string(),
            "0a0b".to_string(),
        ];
        let parsed = parse_args(&args).unwrap();
        assert_eq!(
            parsed,
            Command::Write {
                file: "file.bin".to_string(),
                offset: 0,
                hex: "0a0b".to_string(),
            }
        );
    }

    #[test]
    fn rejects_bad_args() {
        let args = vec!["read".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn converts_hex_to_bytes() {
        let bytes = parse_hex_bytes("4869").unwrap();
        assert_eq!(bytes, b"Hi");
    }

    #[test]
    fn rejects_invalid_hex() {
        assert!(parse_hex_bytes("zz").is_err());
    }

    #[test]
    fn writes_and_reads_back() {
        let path = "./tmp_rust02.bin";
        write_bytes(path, 0, b"Hello").unwrap();
        let read = read_bytes(path, 0, 5).unwrap();
        assert_eq!(read, b"Hello");
        fs::remove_file(path).ok();
    }
}
