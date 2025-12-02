# rust00: Greeting CLI

A minimal greeting program.

## Usage

```
cargo run -p rust00 -- [OPTIONS] [NAME]
```

- `-u`, `--upper`: print the name in uppercase.
- `-r`, `--repeat NUM`: repeat the greeting `NUM` times (default: 1).
- `-h`, `--help`: show usage.
- `NAME`: optional name (defaults to `World`).
