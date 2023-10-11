extern crate random_word;
use random_word::Lang;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

fn handle_client(mut stream: TcpStream, word_to_guess: String,game_state: Arc<Mutex<GameState>>,guessed_letters: Arc<Mutex<HashSet<char>>>) {
    //let mut guessed_letters = Arc::new(Mutex::new(HashSet::new()));
    //let mut attempts = 7;
    let word_chars: Vec<char> = word_to_guess.chars().collect();
    let word_len = word_to_guess.len();
    let _current_state = vec!['_'; word_len-1];

    while game_state.lock().unwrap().remaining_attempts > 0 && game_state.lock().unwrap().current_state.contains(&'_') {
        let mut buffer = [0; 1];
        stream.read(&mut buffer).unwrap();
        let guess = buffer[0] as char;

        if guessed_letters.lock().unwrap().contains(&guess) {
            stream.write(b"You already guessed this letter!\n").unwrap();
            continue;
        }
        let mut locked_game_state = game_state.lock().unwrap();

        if locked_game_state.word_to_guess.contains(guess) {
            stream.write(b"Correct guess!\n").unwrap();
            for (i, letter) in word_chars.iter().enumerate() {
                if letter == &guess {
                    locked_game_state.current_state[i] = guess;
                }
            }
        } else {
            stream.write(b"Wrong guess!\n").unwrap();
            //depending on how much attempts left print a different message

            match locked_game_state.remaining_attempts  {
                1 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=========\n").unwrap(); }
                2 => { stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=========\n").unwrap();}
                3 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=========\n").unwrap();}
                4 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=========\n").unwrap();}
                5 => {stream.write(b"  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=========\n").unwrap();}
                6 => {stream.write(b"  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=========\n").unwrap();}
                _ => {stream.write(b"  +---+\n  |   |\n      |\n      |\n      |\n      |\n=========\n").unwrap();}
            }
            guessed_letters.lock().unwrap().insert(guess);
            locked_game_state.remaining_attempts -= 1;
        }

        guessed_letters.lock().unwrap().insert(guess);
        stream.write(&locked_game_state.current_state.iter().collect::<String>().as_bytes()).unwrap();
        stream.write(b"\n").unwrap();
    }

    if game_state.lock().unwrap().remaining_attempts  > 0 {
        stream.write(b"You won!\n").unwrap();
    } else {
        stream.write(b"You lost! The word was ").unwrap();
        stream.write(word_to_guess.as_bytes()).unwrap();
        stream.write(b"\n").unwrap();
    }
}
struct GameState {
    word_to_guess: String,
    current_state: Vec<char>,
    remaining_attempts: i32,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080...");
    let word = random_word::gen(Lang::Fr);

    let word_to_guess = word.to_string();
    let initial_state = Arc::new(Mutex::new(GameState {
        word_to_guess: word_to_guess.clone(),
        current_state: vec!['_'; word_to_guess.len()],
        remaining_attempts : 7,
    }));
    let guessed_letters = Arc::new(Mutex::new(HashSet::new()));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let word_to_guess = word_to_guess.clone();
        let game_state = initial_state.clone();
        let guessed_letters = guessed_letters.clone();
        thread::spawn(move || {
            handle_client(stream, word_to_guess,game_state,guessed_letters);
        });
    }
}
