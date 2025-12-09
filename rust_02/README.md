# rust02: Hex Read/Write Utility

Read or write raw bytes in hexadecimal form.

## Usage

```
cargo run -p rust02 -- read <file> <offset> <size>
cargo run -p rust02 -- write <file> <offset> <hex>
```

- `read`: prints `<size>` bytes from `<file>` starting at `<offset>`.
- `write`: writes bytes from `<hex>` (pairs like `0a0b`) to `<file>` starting at `<offset>`.
- Offsets are decimal and must be non-negative.
- Hex input must be even-length; spaces are not allowed.
