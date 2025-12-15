use std::collections::HashMap;
use std::env;
use std::io::{self, Read};
use std::process;

struct Config {
    top: usize,
    min_length: usize,
    ignore_case: bool,
    text: Option<String>,
    top_specified: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut config = Config {
        top: 10,
        min_length: 1,
        ignore_case: false,
        text: None,
        top_specified: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "--top" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse() {
                        config.top = n;
                        config.top_specified = true;
                        i += 1;
                    }
                }
            }
            "--min-length" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse() {
                        config.min_length = n;
                        i += 1;
                    }
                }
            }
            "--ignore-case" => {
                config.ignore_case = true;
            }
            arg => {
                if arg.starts_with('-') {
                    eprintln!("error: Unknown option {}", arg);
                    process::exit(2);
                }
                config.text = Some(arg.to_string());
            }
        }
        i += 1;
    }

    let content = match config.text {
        Some(t) => t,
        None => {
            let mut buffer = String::new();
            if io::stdin().read_to_string(&mut buffer).is_err() {
                process::exit(1);
            }
            buffer
        }
    };

    let text_to_process = if config.ignore_case {
        content.to_lowercase()
    } else {
        content
    };

    let mut word_counts: HashMap<String, usize> = HashMap::new();
    let words = text_to_process.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '"');

    for word in words {
        if !word.is_empty() && word.len() >= config.min_length {
            *word_counts.entry(word.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted_words: Vec<(&String, &usize)> = word_counts.iter().collect();

    sorted_words.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    if config.top_specified {
        println!("Top {} words:", config.top);
    } else {
        println!("Word frequency:");
    }

    for (word, count) in sorted_words.iter().take(config.top) {
        println!("{}: {}", word, format_number(**count));
    }
}

fn print_help() {
    println!("Usage: wordfreq [OPTIONS]");
    println!();
    println!("Count word frequency in text");
    println!();
    println!("Arguments:");
    println!("Text to analyze (or use stdin)");
    println!();
    println!("Options:");
    println!("--top Show top N words [default: 10]");
    println!("--min-length Ignore words shorter than N [default: 1]");
    println!("--ignore-case Case insensitive counting");
    println!("-h, --help");
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (count, c) in s.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}
