use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::io::BufRead;
fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Connected to the server.");

    let mut stream_clone = stream.try_clone()?; // Clone the stream for the input thread

    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let input = line.unwrap();
            stream_clone.write(input.to_lowercase().as_bytes()).unwrap();
        }
    });

    let mut buffer = [0; 128];
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let response = String::from_utf8_lossy(&buffer[0..bytes_read]);
        print!("{}", response);
        buffer = [0; 128];
    }

    Ok(())
}