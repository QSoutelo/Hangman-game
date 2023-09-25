use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::collections::HashSet;

fn handle_client(mut stream: TcpStream, word_to_guess: String) {
    let mut guessed_letters = HashSet::new();
    let mut attempts = 7;

    let word_len = word_to_guess.len();
    let mut current_state = vec!['_'; word_len];

    while attempts > 0 && current_state.contains(&'_') {
        let mut buffer = [0; 1];
        stream.read(&mut buffer).unwrap();
        let guess = buffer[0] as char;

        if guessed_letters.contains(&guess) {
            stream.write(b"You already guessed this letter!\n").unwrap();
            continue;
        }

        if word_to_guess.contains(guess) {
            stream.write(b"Correct guess!\n").unwrap();
            for (i, letter) in word_to_guess.chars().enumerate() {
                if letter == guess {
                    current_state[i] = guess;
                }
            }
        } else {
            stream.write(b"Wrong gfuess!\n").unwrap();
            //depending on how much attempts left print a different message

            match attempts {
                1 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=========").unwrap(); }
                2 => { stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=========").unwrap();}
                3 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=========").unwrap();}
                4 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=========").unwrap();}
                5 => {stream.write(b"  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=========").unwrap();}
                6 => {stream.write(b"  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=========").unwrap();}
                _ => {stream.write(b"  +---+\n  |   |\n      |\n      |\n      |\n      |\n=========").unwrap();}
            }

            attempts -= 1;
        }

        guessed_letters.insert(guess);
        stream.write(&current_state.iter().collect::<String>().as_bytes()).unwrap();
        stream.write(b"\n").unwrap();
    }

    if attempts > 0 {
        stream.write(b"You won!\n").unwrap();
    } else {
        stream.write(b"You lost! The word was ").unwrap();
        stream.write(word_to_guess.as_bytes()).unwrap();
        stream.write(b"\n").unwrap();
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080...");

    let word_to_guess = "rust".to_string();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let word_to_guess = word_to_guess.clone();
        thread::spawn(move || {
            handle_client(stream, word_to_guess);
        });
    }
}
