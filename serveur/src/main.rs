extern crate random_word;
extern crate diacritics;
use random_word::Lang;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
// Définition d'un trait Joueur.
trait Joueur {
    fn jouer(&mut self, lettre: char) -> bool;
}

impl Joueur for GameState {
    fn jouer(&mut self, lettre: char) -> bool {
        if self.guessed_letters.lock().unwrap().contains(&lettre) {
            return false;
        }

        if self.word_to_guess.contains(lettre) {
            for (i, letter) in self.word_to_guess.chars().enumerate() {
                if letter == lettre {
                    self.current_state[i] = lettre;
                }
            }
            self.guessed_letters.lock().unwrap().insert(lettre);
            true
        } else {
            //self.attempts -= 1;
            self.guessed_letters.lock().unwrap().insert(lettre);
            false
        }
    }
}
enum Message {
    Broadcast(String),
    UpdateGuessedLetters(Arc<Mutex<HashSet<char>>>),
}
struct GameState {
    word_to_guess: String,
    current_state: Vec<char>,
    guessed_letters: Arc<Mutex<HashSet<char>>>,
    attempts: i32,
}

fn handle_client(mut stream: TcpStream, word_to_guess: String, game_state: Arc<Mutex<GameState>>,broadcast_tx: Sender<Message>) {


    while game_state.lock().unwrap().attempts > 0 && game_state.lock().unwrap().current_state.contains(&'_') {
        let mut buffer = [0; 1];
        if let Err(err) = stream.read(&mut buffer) {
            eprintln!("Error reading from stream: {}", err);
            break;
        }
        let guess = buffer[0] as char;
       
        let broadcast_message = format!("Player has guessed: {}", guess);
        broadcast_tx.send(Message::Broadcast(broadcast_message)).unwrap();
       
        let guessed_letters_clone = {
            let locked_game_state = game_state.lock().unwrap();
            locked_game_state.guessed_letters.clone()
        }; // Add a semicolon to return the value
        
       
        let  mut locked_game_state = game_state.lock().unwrap();
        broadcast_tx.send(Message::UpdateGuessedLetters(locked_game_state.guessed_letters.clone())).unwrap();
        if guessed_letters_clone.lock().unwrap().contains(&guess) {
            if let Err(err) = stream.write(b"You already guessed this letter!\n") {
                eprintln!("Error writing to stream: {}", err);
                break;
            }
            continue;
        }

 

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

        guessed_letters_clone.lock().unwrap().insert(guess);

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

    } else {
        if let Err(err) = stream.write(b"You lost! The word was ") {
            eprintln!("Error writing to stream: {}", err);
        }
        if let Err(err) = stream.write(word_to_guess.as_bytes()) {
            eprintln!("Error writing to stream: {}", err);
        }

    }



}



fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on port 8080...");
    let listener = Arc::new(Mutex::new(listener));
    let word = diacritics::remove_diacritics( random_word::gen(Lang::Fr));
    
    let word_to_guess = word.to_string();
    let (broadcast_tx, broadcast_rx) = channel();
    let initial_state = Arc::new(Mutex::new(GameState {
        word_to_guess: word_to_guess.clone(),
        current_state: vec!['_'; word_to_guess.len()],
        guessed_letters: Arc::new(Mutex::new(HashSet::new())),
        attempts: 7,
    }));

    thread::spawn ({
        let listener = listener.clone();
    
    move || {
        let listener = listener.lock().unwrap();
        let listener_clone = listener.try_clone().expect("Failed to clone listener");
        let clients: Arc<Mutex<Vec<TcpStream>>>= Arc::new(Mutex::new(Vec::new()));

        for stream in listener_clone.incoming() {
            let stream = stream.unwrap();
            let mut clients = clients.lock().unwrap();
            clients.push(stream.try_clone().expect("Failed to clone stream"));
        }

        for message in broadcast_rx {
            match message {
                Message::Broadcast(msg) => {
                    let clients = clients.lock().unwrap();
                    for client in &*clients {
                        let mut client = client.try_clone().expect("Failed to clone the client"); // Clone the client inside the loop
                        client.write(msg.as_bytes()).unwrap();
                        client.write(b"\n").unwrap();
                    }
                }
                _ => {
                    // Handle unexpected messages (optional)
                    // You can add specific handling for Message::UpdateGuessedLetters here if needed
                }
                // Handle other message types if needed

            }
        }
    }
    });
    for stream in listener.lock().unwrap().incoming() {
        
    
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Error accepting incoming connection: {}", err);
                continue;
            }
        };
        let word_to_guess = word_to_guess.clone();
        let game_state = initial_state.clone();
        let broadcast_tx = broadcast_tx.clone();
        thread::spawn(move || {
            handle_client(stream, word_to_guess, game_state,broadcast_tx);


            
            });
        }
    }
