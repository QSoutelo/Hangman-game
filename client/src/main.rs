
use std::io::{self, prelude::*, BufReader, Write};
use std::net::TcpStream;
use std::str;
use std::thread;
use std::sync::mpsc;
fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Connected to the server.");

    let mut stream_clone = stream.try_clone()?; // Clone the stream for the input thread
    let (output_sender, output_receiver) = mpsc::channel();
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let input = line.unwrap();
            stream_clone.write(input.to_lowercase().as_bytes()).unwrap();
        }
    });

    let mut buffer = [0; 128];
    loop {
        // Lire les mises à jour du jeu depuis le serveur
        let mut reader = BufReader::new(&stream);
        let mut buffer: Vec<u8> = Vec::new();
        reader.read_until(b'\n', &mut buffer)?;
        let received_message = format!("Received: {}", str::from_utf8(&buffer).unwrap());
        output_sender.send(received_message).unwrap();
       //println!("\n read from server: {}", str::from_utf8(&buffer).unwrap());
        println!("");
        
        // Lire la réponse du serveur
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let response = String::from_utf8_lossy(&buffer[0..bytes_read]);
        print!("{}", response);
    }

    Ok(())
}
