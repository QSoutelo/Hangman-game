use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::io::BufRead;

fn handle_input(mut stream: TcpStream) -> io::Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line?;
        stream.write(input.to_lowercase().as_bytes())?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Connected to the server.");

    let stream_clone = stream.try_clone()?;
    let input_thread = thread::spawn(move || {
        if let Err(err) = handle_input(stream_clone) {
            eprintln!("Error in input thread: {}", err);
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

    input_thread.join().expect("Input thread panicked.");

    Ok(())
}
