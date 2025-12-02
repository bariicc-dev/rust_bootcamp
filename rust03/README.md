# rust03: Diffie-Hellman Stream Cipher Chat

A minimal demo chat that performs a Diffie-Hellman key exchange and then shares messages over TCP using a simple XOR stream cipher derived from the shared secret.

## Usage

Terminal 1 (server):
```bash
cargo run -p rust03 -- server --bind 0.0.0.0:8080
```

Terminal 2 (client):
```bash
cargo run -p rust03 -- client --connect 127.0.0.1:8080
```

Type messages and press Enter to send. Messages are encrypted with a keystream seeded from the shared secret. This is an educational demo only.
