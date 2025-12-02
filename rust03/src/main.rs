use clap::{Parser, Subcommand};
use rand::Rng;
use std::io::{self, BufRead, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;

const P: u64 = 0xFFFFFFFB; // 2^32 - 5, a prime
const G: u64 = 5; // small generator

#[derive(Parser)]
#[command(author, version, about = "Diffie-Hellman stream cipher chat", long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Run as the server and wait for a client connection
    Server {
        #[arg(long, default_value = "0.0.0.0:8080")]
        bind: String,
    },
    /// Run as the client and connect to a server
    Client {
        #[arg(long, default_value = "127.0.0.1:8080")]
        connect: String,
    },
}

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result: u64 = 1;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = ((result as u128 * base as u128) % modulus as u128) as u64;
        }
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
        exp >>= 1;
    }
    result
}

#[derive(Clone)]
struct Keystream {
    state: u64,
}

impl Keystream {
    fn from_secret(secret: u64, label: u64) -> Self {
        Keystream {
            state: secret ^ label ^ 0x6f6dca73,
        }
    }

    fn next_byte(&mut self) -> u8 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.state >> 24) as u8
    }

    fn apply(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.next_byte();
        }
    }
}

fn send_encrypted(
    mut stream: &TcpStream,
    message: &str,
    keystream: &mut Keystream,
) -> io::Result<()> {
    let mut data = message.as_bytes().to_vec();
    keystream.apply(&mut data);
    let len = data.len() as u32;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&data)?;
    Ok(())
}

fn receive_encrypted(mut stream: &TcpStream, keystream: &mut Keystream) -> io::Result<String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data)?;
    keystream.apply(&mut data);
    String::from_utf8(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn derive_keys(peer_public: u64, private_key: u64) -> (Keystream, Keystream, u64) {
    let secret = mod_pow(peer_public, private_key, P);
    let send_stream = Keystream::from_secret(secret, 1);
    let recv_stream = Keystream::from_secret(secret, 2);
    (send_stream, recv_stream, secret)
}

fn exchange_keys(stream: &mut TcpStream) -> io::Result<(Keystream, Keystream)> {
    let private_key: u64 = rand::thread_rng().gen_range(2..P - 1);
    let public_key = mod_pow(G, private_key, P);

    stream.write_all(&public_key.to_le_bytes())?;
    let mut peer_buf = [0u8; 8];
    stream.read_exact(&mut peer_buf)?;
    let peer_public = u64::from_le_bytes(peer_buf);

    let (send_stream, recv_stream, secret) = derive_keys(peer_public, private_key);
    println!("Computed shared secret: 0x{:x}", secret);
    Ok((send_stream, recv_stream))
}

fn handle_connection(mut stream: TcpStream, role: &str) -> io::Result<()> {
    println!("[{role}] Exchange public keys...");
    let (mut send_stream, mut recv_stream) = exchange_keys(&mut stream)?;
    println!("[{role}] Ready. Type messages and press Enter.");

    let mut send_clone = stream.try_clone()?;
    let mut recv_clone = stream;

    let sender = thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let msg = line.unwrap_or_default();
            if msg.is_empty() {
                continue;
            }
            if let Err(err) = send_encrypted(&send_clone, &msg, &mut send_stream) {
                eprintln!("Send error: {err}");
                break;
            }
        }
    });

    let receiver = thread::spawn(move || loop {
        match receive_encrypted(&recv_clone, &mut recv_stream) {
            Ok(text) => println!("[{role}] Peer: {text}"),
            Err(err) => {
                eprintln!("Receive error: {err}");
                break;
            }
        }
    });

    sender.join().ok();
    receiver.join().ok();
    Ok(())
}

fn run_server(bind: &str) -> io::Result<()> {
    let listener = TcpListener::bind(bind)?;
    println!("[server] Listening on {bind}...");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("[server] Client connected: {}", stream.peer_addr()?);
                handle_connection(stream, "server")?;
            }
            Err(err) => eprintln!("[server] Connection failed: {err}"),
        }
    }
    Ok(())
}

fn run_client<A: ToSocketAddrs>(addr: A) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    println!("[client] Connected to server.");

    // Client receives server public key first, so swap order.
    let private_key: u64 = rand::thread_rng().gen_range(2..P - 1);
    let public_key = mod_pow(G, private_key, P);

    let mut peer_buf = [0u8; 8];
    stream.read_exact(&mut peer_buf)?;
    let peer_public = u64::from_le_bytes(peer_buf);
    stream.write_all(&public_key.to_le_bytes())?;

    let (mut send_stream, mut recv_stream, secret) = derive_keys(peer_public, private_key);
    println!("Computed shared secret: 0x{:x}", secret);
    println!("[client] Ready. Type messages and press Enter.");

    let mut send_clone = stream.try_clone()?;
    let mut recv_clone = stream;

    let sender = thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let msg = line.unwrap_or_default();
            if msg.is_empty() {
                continue;
            }
            if let Err(err) = send_encrypted(&send_clone, &msg, &mut send_stream) {
                eprintln!("Send error: {err}");
                break;
            }
        }
    });

    let receiver = thread::spawn(move || loop {
        match receive_encrypted(&recv_clone, &mut recv_stream) {
            Ok(text) => println!("[client] Peer: {text}"),
            Err(err) => {
                eprintln!("Receive error: {err}");
                break;
            }
        }
    });

    sender.join().ok();
    receiver.join().ok();
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Server { bind } => run_server(&bind),
        Mode::Client { connect } => run_client(connect),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_pow_matches_pow() {
        let p = 1_000_003u64;
        assert_eq!(mod_pow(7, 12345, p), 766_228);
    }

    #[test]
    fn keystream_is_reversible() {
        let mut ks = Keystream::from_secret(0x1234, 1);
        let mut data = b"hello".to_vec();
        ks.apply(&mut data);
        ks = Keystream::from_secret(0x1234, 1);
        ks.apply(&mut data);
        assert_eq!(data, b"hello");
    }
}
