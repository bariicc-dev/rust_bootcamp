use std::env;
use std::process;

#[derive(Debug)]
struct Args {
    name: String,
    upper: bool,
    repeat: u32,
}

fn main() {
    let args = parse_args(env::args().skip(1));
    let name = if args.upper {
        args.name.to_uppercase()
    } else {
        args.name
    };

    for _ in 0..args.repeat {
        println!("Hello, {}!", name);
    }
}

fn parse_args<I>(mut iter: I) -> Args
where
    I: Iterator<Item = String>,
{
    let mut name: Option<String> = None;
    let mut upper = false;
    let mut repeat: u32 = 1;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-u" | "--upper" => upper = true,
            "-r" | "--repeat" => {
                repeat = match iter.next() {
                    Some(value) => value.parse().unwrap_or_else(|_| {
                        eprintln!("Invalid value for --repeat. Expected a positive integer.");
                        process::exit(1);
                    }),
                    None => {
                        eprintln!("Missing value for --repeat option.");
                        process::exit(1);
                    }
                };
            }
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            _ => {
                name = Some(arg);
            }
        }
    }

    Args {
        name: name.unwrap_or_else(|| "World".to_string()),
        upper,
        repeat,
    }
}

fn print_usage() {
    println!(
        "Usage: hello [OPTIONS] [NAME]\n\nOptions:\n  -u, --upper         Convert the name to uppercase\n  -r, --repeat NUM    Repeat the greeting NUM times (default: 1)\n  -h, --help          Show this help message"
    );
}
