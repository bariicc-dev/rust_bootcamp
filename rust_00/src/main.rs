use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut name = String::from("World");
    let mut count = 1;
    let mut uppercase = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg == "-h" || arg == "--help" {
            print_help();
            return;
        } else if arg == "--upper" {
            uppercase = true;
        } else if arg == "--repeat" {
            if i + 1 < args.len() {
                let next_arg = &args[i + 1];
                count = next_arg.parse().expect("Error: --repeat expects a number");
                i += 1;
            } else {
                eprintln!("Error: --repeat requires a value");
                return;
            }
        } else if arg.starts_with('-') {
            eprintln!("error: Unknown option {}", arg);
            std::process::exit(2);
        } else {
            name = arg.clone();
        }

        i += 1;
    }

    for _ in 0..count {
        let mut greeting = format!("Hello, {}!", name);

        if uppercase {
            greeting = greeting.to_uppercase();
        }

        println!("{}", greeting);
    }
}

fn print_help() {
    println!("Usage: hello [OPTIONS] [NAME]");
    println!();
    println!("Arguments:");
    println!("  [NAME] Name to greet [default: World]");
    println!();
    println!("Options:");
    println!("  --upper Convert to uppercase");
    println!("  --repeat Repeat greeting N times [default: 1]");
    println!("  -h, --help Print help");
}
