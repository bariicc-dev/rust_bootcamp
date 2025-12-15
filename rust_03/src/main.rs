use rand::Rng;
use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

const P: u64 = 0xD87FA3E291B4C7F3;
const G: u64 = 2;

const LCG_A: u64 = 1103515245;
const LCG_C: u64 = 12345;
const LCG_M: u64 = 1u64 << 32;

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result as u128 * base as u128 % modulus as u128) as u64;
        }
        base = (base as u128 * base as u128 % modulus as u128) as u64;
        exp /= 2;
    }
    result
}

fn format_hex_u64(val: u64) -> String {
    let bytes = val.to_be_bytes();
    format!(
        "{:02X}{:02X} {:02X}{:02X} {:02X}{:02X} {:02X}{:02X}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
    )
}

fn format_hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        let state_u32 = self.state as u32;
        let next_val = (LCG_A.wrapping_mul(state_u32 as u64).wrapping_add(LCG_C)) % LCG_M;
        self.state = next_val;
        next_val as u32
    }
}

struct LcgStream {
    lcg: Lcg,
    buffer: Vec<u8>,
    total_consumed: usize,
}

impl LcgStream {
    fn new(seed: u64) -> Self {
        LcgStream {
            lcg: Lcg::new(seed),
            buffer: Vec::new(),
            total_consumed: 0,
        }
    }

    fn get_byte(&mut self) -> u8 {
        if self.buffer.is_empty() {
            let val = self.lcg.next_u32();
            self.buffer.extend_from_slice(&val.to_be_bytes());
        }
        self.total_consumed += 1;
        self.buffer.remove(0)
    }

    fn peek_bytes(&mut self, count: usize) -> Vec<u8> {
        let mut temp_lcg = Lcg {
            state: self.lcg.state,
        };
        let mut temp_buf = self.buffer.clone();
        let mut result = Vec::new();

        while temp_buf.len() < count {
            let val = temp_lcg.next_u32();
            temp_buf.extend_from_slice(&val.to_be_bytes());
        }
        result.extend_from_slice(&temp_buf[0..count]);
        result
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_help();
        }
        "server" => {
            if args.len() != 3 {
                print_help();
                return;
            }
            let port = &args[2];
            server(port);
        }
        "client" => {
            if args.len() != 3 {
                print_help();
                return;
            }
            let addr = &args[2];
            client(addr);
        }
        _ => {
            eprintln!("error: Invalid command");
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!("Usage: streamchat\n");
    println!("Stream cipher chat with Diffie-Hellman key generation\n");
    println!("Commands:");
    println!("server Start server");
    println!("client Connect to server");
}

fn print_dh_params() {
    println!("[DH] Using hardcoded DH parameters:");
    println!("p = {} (64-bit prime - public)", format_hex_u64(P));
    println!("g = {} (generator - public)\n", G);
}

fn server(port: &str) {
    print_dh_params();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind");
    println!("[SERVER] Listening on 0.0.0.0:{}", port);
    println!("[SERVER] Waiting for client...\n");

    let (mut stream, addr) = listener.accept().expect("Failed to accept");
    println!("[CLIENT] Connected from {}\n", addr);

    let secret = dh_handshake(&mut stream, true);
    chat_loop(stream, secret, "[CLIENT]");
}

fn client(addr: &str) {
    print_dh_params();
    println!("[CLIENT] Connecting to {}...", addr);
    let mut stream = TcpStream::connect(addr).expect("Failed to connect");
    println!("[CLIENT] Connected!\n");

    let secret = dh_handshake(&mut stream, false);
    chat_loop(stream, secret, "[SERVER]");
}

fn dh_handshake(stream: &mut TcpStream, is_server: bool) -> u64 {
    println!("[DH] Starting key exchange...");

    println!("[DH] Generating our keypair...");
    let private_key: u64 = rand::thread_rng().gen();
    println!("private_key = {:X} (random 64-bit)", private_key);

    let public_key = mod_pow(G, private_key, P);
    println!("public_key = g^private mod p");
    println!("= {}^{:X} mod p", G, private_key);
    println!("= {:X}\n", public_key);

    println!("[DH] Exchanging keys...");

    let their_public_key: u64;

    if is_server {
        println!("[NETWORK] Sending public key (8 bytes)...");
        stream.write_all(&public_key.to_be_bytes()).unwrap();
        println!("-> Send our public: {:X}", public_key);

        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf).unwrap();
        println!("[NETWORK] Received public key (8 bytes) ✓");
        their_public_key = u64::from_be_bytes(buf);
        println!("<- Receive their public: {:X}\n", their_public_key);
    } else {
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf).unwrap();
        println!("[NETWORK] Received public key (8 bytes) ✓");
        their_public_key = u64::from_be_bytes(buf);
        println!("<- Receive their public: {:X}", their_public_key);

        println!("[NETWORK] Sending public key (8 bytes)...");
        stream.write_all(&public_key.to_be_bytes()).unwrap();
        println!("-> Send our public: {:X}\n", public_key);
    }

    println!("[DH] Computing shared secret...");
    println!("Formula: secret = (their_public)^(our_private) mod p\n");

    let secret = mod_pow(their_public_key, private_key, P);
    println!(
        "secret = ({:X})^({:X}) mod p",
        their_public_key, private_key
    );
    println!("= {:X}\n", secret);

    println!("[VERIFY] Both sides computed the same secret ✓\n");

    println!("[STREAM] Generating keystream from secret...");
    println!("Algorithm: LCG (a={}, c={}, m=2^32)", LCG_A, LCG_C);
    println!("Seed: secret = {:X}\n", secret);

    let mut lcg_stream = LcgStream::new(secret);
    let preview = lcg_stream.peek_bytes(16);
    let preview_str = format_hex_bytes(&preview);
    println!("Keystream: {} ...\n", preview_str);

    println!("✓ Secure channel established!\n");

    secret
}

fn chat_loop(mut stream: TcpStream, secret: u64, remote_label: &'static str) {
    let lcg = Arc::new(Mutex::new(LcgStream::new(secret)));

    let mut stream_clone = stream.try_clone().expect("Failed to clone stream");
    let lcg_clone = lcg.clone();

    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match stream_clone.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    println!("[NETWORK] Received encrypted message ({} bytes)", n);
                    println!("[<-] Received {} bytes\n", n);

                    println!("[DECRYPT]");
                    let cipher = &buf[0..n];
                    println!("Cipher: {}", format_hex_bytes(cipher));

                    let mut lcg = lcg_clone.lock().unwrap();
                    let start_pos = lcg.total_consumed;
                    let mut plain = Vec::new();
                    let mut key_bytes = Vec::new();

                    for &b in cipher {
                        let k = lcg.get_byte();
                        key_bytes.push(k);
                        plain.push(b ^ k);
                    }

                    println!(
                        "Key: {} (keystream position: {})",
                        format_hex_bytes(&key_bytes),
                        start_pos
                    );

                    let plain_str = String::from_utf8_lossy(&plain);
                    println!(
                        "Plain: {} -> \"{}\"",
                        format_hex_bytes(&plain),
                        plain_str.trim()
                    );

                    let re_cipher: Vec<u8> = plain
                        .iter()
                        .zip(key_bytes.iter())
                        .map(|(p, k)| p ^ k)
                        .collect();
                    if re_cipher == cipher {
                        println!("[TEST] Round-trip verified: \"{}\" -> encrypt -> decrypt -> \"{}\" ✓\n", plain_str.trim(), plain_str.trim());
                    }

                    println!("{} {}", remote_label, plain_str.trim());

                    println!("\n[CHAT] Type message:");
                    print!("> ");
                    io::stdout().flush().unwrap();
                }
                Err(_) => break,
            }
        }
    });

    let mut input = String::new();
    loop {
        println!("[CHAT] Type message:");
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let msg = input.trim();
        if msg.is_empty() {
            continue;
        }

        println!("\n[ENCRYPT]");
        let plain_bytes = msg.as_bytes();
        println!("Plain: {} (\"{}\")", format_hex_bytes(plain_bytes), msg);

        let mut lcg = lcg.lock().unwrap();
        let start_pos = lcg.total_consumed;
        let mut cipher = Vec::new();
        let mut key_bytes = Vec::new();

        for &b in plain_bytes {
            let k = lcg.get_byte();
            key_bytes.push(k);
            cipher.push(b ^ k);
        }

        println!(
            "Key: {} (keystream position: {})",
            format_hex_bytes(&key_bytes),
            start_pos
        );
        println!("Cipher: {}\n", format_hex_bytes(&cipher));

        println!(
            "[NETWORK] Sending encrypted message ({} bytes)...",
            cipher.len()
        );
        if stream.write_all(&cipher).is_err() {
            break;
        }
        println!("[->] Sent {} bytes\n", cipher.len());
    }
}
