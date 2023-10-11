extern crate random_word;
extern crate diacritics;
use random_word::Lang;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

// Définition d'un trait Joueur.
trait Joueur {
    fn jouer(&mut self, lettre: char) -> bool;
}

impl Joueur for GameState {
    fn jouer(&mut self, lettre: char) -> bool {
        if self.guessed_letters.contains(&lettre) {
            return false;
        }

        if self.word_to_guess.contains(lettre) {
            for (i, letter) in self.word_to_guess.chars().enumerate() {
                if letter == lettre {
                    self.current_state[i] = lettre;
                }
            }
            self.guessed_letters.insert(lettre);
            true
        } else {
            //self.attempts -= 1;
            self.guessed_letters.insert(lettre);
            false
        }
    }
}

struct GameState {
    word_to_guess: String,
    current_state: Vec<char>,
    guessed_letters: HashSet<char>,
    attempts: i32,
}

fn handle_client(mut stream: TcpStream, word_to_guess: String, game_state: Arc<Mutex<GameState>>) {
    let mut guessed_letters = HashSet::new();
    let word_chars: Vec<char> = word_to_guess.chars().collect();
    let word_len = word_to_guess.len();

    while game_state.lock().unwrap().attempts > 0 && game_state.lock().unwrap().current_state.contains(&'_') {
        let mut buffer = [0; 1];
        stream.read(&mut buffer).unwrap();
        let guess = buffer[0] as char;

        if guessed_letters.contains(&guess) {
            stream.write(b"You already guessed this letter!\n").unwrap();
            continue;
        }

        let mut locked_game_state = game_state.lock().unwrap();

        if locked_game_state.jouer(guess) {
            stream.write(b"Correct guess!\n").unwrap();
        } else {
            stream.write(b"Wrong guess!\n").unwrap();

            match locked_game_state.attempts {
                1 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=========\n").unwrap(); }
                2 => { stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=========\n").unwrap();}
                3 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=========\n").unwrap();}
                4 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=========\n").unwrap();}
                5 => {stream.write(b"  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=========\n").unwrap();}
                6 => {stream.write(b"  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=========\n").unwrap();}
                _ => {stream.write(b"  +---+\n  |   |\n      |\n      |\n      |\n      |\n=========\n").unwrap();}
            }

            locked_game_state.attempts -= 1;
        }

        guessed_letters.insert(guess);
        stream.write(&locked_game_state.current_state.iter().collect::<String>().as_bytes()).unwrap();
        stream.write(b"\n").unwrap();
    }

    if game_state.lock().unwrap().attempts > 0 {
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
    let word = diacritics::remove_diacritics( random_word::gen(Lang::Fr));
    
    let word_to_guess = word.to_string();
    let initial_state = Arc::new(Mutex::new(GameState {
        word_to_guess: word_to_guess.clone(),
        current_state: vec!['_'; word_to_guess.len()],
        guessed_letters: HashSet::new(),
        attempts: 7,
    }));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let word_to_guess = word_to_guess.clone();
        let game_state = initial_state.clone();
        thread::spawn(move || {
            handle_client(stream, word_to_guess, game_state);
        });
    }
}