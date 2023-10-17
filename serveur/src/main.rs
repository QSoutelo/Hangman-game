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
        if let Err(err) = stream.read(&mut buffer) {
            eprintln!("Error reading from stream: {}", err);
            break;
        }
        let guess = buffer[0] as char;

        if guessed_letters.contains(&guess) {
            if let Err(err) = stream.write(b"You already guessed this letter!\n") {
                eprintln!("Error writing to stream: {}", err);
                break;
            }
            continue;
        }

        let mut locked_game_state = game_state.lock().unwrap();

        if locked_game_state.jouer(guess) {
            if let Err(err) = stream.write(b"Correct guess!\n") {
                eprintln!("Error writing to stream: {}", err);
                break;
            }
        } else {
            if let Err(err) = stream.write(b"Wrong guess!\n") {
                eprintln!("Error writing to stream: {}", err);
                break;
            }

            match locked_game_state.attempts {
                1 => {
                    if let Err(err) = stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=========\n") {
                        eprintln!("Error writing to stream: {}", err);
                        break;
                    }
                }
                2 => {
                    if let Err(err) = stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=========\n") {
                        eprintln!("Error writing to stream: {}", err);
                        break;
                    }
                }
                3 => {
                    if let Err(err) = stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=========\n") {
                        eprintln!("Error writing to stream : {}", err);
                        break;
                    }
                }
                4 => {
                    if let Err(err) = stream.write(b"  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=========\n") {
                        eprintln!("Error writing to stream : {}", err);
                        break;
                    }
                }
                5 => {
                    if let Err(err) = stream.write(b"  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=========\n") {
                        eprintln!("Error writing to stream : {}", err);
                        break;
                    }
                }
                6 => {
                    if let Err(err) = stream.write(b"  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=========\n") {
                        eprintln!("Error writing to stream : {}", err);
                        break;
                    }
                }
                _ => {
                    if let Err(err) = stream.write(b"  +---+\n  |   |\n      |\n      |\n      |\n      |\n=========\n") {
                        eprintln!("Error writing to stream : {}", err);
                        break;
                    }
                }
            }

            locked_game_state.attempts -= 1;
        }

        guessed_letters.insert(guess);
        if let Err(err) = stream.write(&locked_game_state.current_state.iter().collect::<String>().as_bytes()) {
            eprintln!("Error writing to stream: {}", err);
            break;
        }
        if let Err(err) = stream.write(b"\n") {
            eprintln!("Error writing to stream: {}", err);
            break;
        }
    }

    if game_state.lock().unwrap().attempts > 0 {
        if let Err(err) = stream.write(b"You won!\n") {
            eprintln!("Error writing to stream: {}", err);
        }
        if let Err(err) = stream.write(b"Do you want to play again? (O/N): \n") {
            eprintln!("Error writing to stream: {}", err);
        }
    } else {
        if let Err(err) = stream.write(b"You lost! The word was ") {
            eprintln!("Error writing to stream: {}", err);
        }
        if let Err(err) = stream.write(word_to_guess.as_bytes()) {
            eprintln!("Error writing to stream: {}", err);
        }
        if let Err(err) = stream.write(b"\n Do you want to play again? (O/N): \n"){
            eprintln!("Error writing to stream: {}", err);
        }
    }


    if ask_to_play_again(&mut stream) {
        // Réinitialisez l'état du jeu et relancez la partie.
        let new_word = diacritics::remove_diacritics(random_word::gen(Lang::Fr));
        let new_game_state = Arc::new(Mutex::new(GameState {
            word_to_guess: new_word.clone(),
            current_state: vec!['_'; new_word.len()],
            guessed_letters: HashSet::new(),
            attempts: 7,
        }));
        handle_client(stream, new_word, new_game_state.clone());
    } else {
        // Fermez la connexion.
        if let Err(err) = stream.write(b"Goodbye!") {
            eprintln!("Error writing to stream: {}", err);
        }
    }
}
fn ask_to_play_again(stream: &mut TcpStream) -> bool {
   
    let mut buffer = [0; 1];
    if let Err(err) = stream.read(&mut buffer) {
        eprintln!("Error reading from stream: {}", err);
    }
    let response = buffer[0] as char;
    response.to_ascii_uppercase() == 'O'
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
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
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Error accepting incoming connection: {}", err);
                continue;
            }
        };
        let word_to_guess = word_to_guess.clone();
        let game_state = initial_state.clone();
        thread::spawn(move || {
            handle_client(stream, word_to_guess, game_state);
        });
    }
}