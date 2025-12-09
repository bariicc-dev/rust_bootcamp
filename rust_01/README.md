# rust01: Word Frequency Analyzer

Counts words from standard input with configurable options.

## Usage

```
cargo run -p rust01 -- [OPTIONS] < input.txt
```

- `-t`, `--top NUM`: show at most `NUM` words (default: 10).
- `-i`, `--ignore-case`: ignore case when counting words.
- `-m`, `--min NUM`: only include words appearing at least `NUM` times (default: 1).
- `-h`, `--help`: show usage.
