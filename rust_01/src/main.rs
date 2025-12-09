use std::collections::HashMap;
use std::env;
use std::io::{self, Read};

#[derive(Debug)]
struct Args {
    top: usize,
    ignore_case: bool,
    min: usize,
}

fn main() -> io::Result<()> {
    let args = parse_args(env::args().skip(1));

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let counts = count_words(&input, args.ignore_case);

    let mut items: Vec<(String, usize)> = counts
        .into_iter()
        .filter(|(_, count)| *count >= args.min)
        .collect();

    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    for (word, count) in items.into_iter().take(args.top) {
        println!("{}: {}", word, count);
    }

    Ok(())
}

fn parse_args<I>(mut iter: I) -> Args
where
    I: Iterator<Item = String>,
{
    let mut top: usize = 10;
    let mut ignore_case = false;
    let mut min: usize = 1;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-t" | "--top" => {
                top = parse_value(iter.next(), "--top");
            }
            "-i" | "--ignore-case" => {
                ignore_case = true;
            }
            "-m" | "--min" => {
                min = parse_value(iter.next(), "--min");
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    Args {
        top,
        ignore_case,
        min,
    }
}

fn parse_value(value: Option<String>, flag: &str) -> usize {
    match value {
        Some(v) => v.parse().unwrap_or_else(|_| {
            eprintln!("Invalid value for {}. Expected a positive integer.", flag);
            std::process::exit(1);
        }),
        None => {
            eprintln!("Missing value for {} option.", flag);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!(
        "Usage: wordfreq [OPTIONS]\n\nOptions:\n  -t, --top NUM         Show at most NUM words (default: 10)\n  -i, --ignore-case     Count words without case sensitivity\n  -m, --min NUM         Only include words appearing at least NUM times (default: 1)\n  -h, --help            Show this help message"
    );
}

fn count_words(text: &str, ignore_case: bool) -> HashMap<String, usize> {
    let mut frequencies = HashMap::new();

    for raw_word in text.split_whitespace() {
        if let Some(clean_word) = normalize_word(raw_word, ignore_case) {
            *frequencies.entry(clean_word).or_insert(0) += 1;
        }
    }

    frequencies
}

fn normalize_word(word: &str, ignore_case: bool) -> Option<String> {
    let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
    if trimmed.is_empty() {
        return None;
    }

    let normalized = if ignore_case {
        trimmed.to_lowercase()
    } else {
        trimmed.to_string()
    };

    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_punctuation_and_counts() {
        let input = "Hello, hello! HELLO?";
        let counts = count_words(input, true);
        assert_eq!(counts.get("hello"), Some(&3));
    }

    #[test]
    fn honors_case_when_requested() {
        let input = "Word word WORD";
        let counts = count_words(input, false);
        assert_eq!(counts.get("Word"), Some(&1));
        assert_eq!(counts.get("word"), Some(&1));
        assert_eq!(counts.get("WORD"), Some(&1));
    }

    #[test]
    fn ignores_empty_entries() {
        let input = "...";
        let counts = count_words(input, false);
        assert!(counts.is_empty());
    }
}
