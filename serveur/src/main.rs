extern crate random_word;
use random_word::Lang;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(
    mut stream: TcpStream,
    word_to_guess: String,
    game_state: Arc<Mutex<GameState>>,
    sender: std::sync::mpsc::Sender<String>,
) {
    let mut guessed_letters = Vec::new();
    let mut attempts = 7;
    let word_chars: Vec<char> = word_to_guess.chars().collect();

    while attempts > 0 {
        let mut buffer = [0; 1];
        stream.read(&mut buffer).unwrap();
        let guess = buffer[0] as char;

        if guessed_letters.contains(&guess) {
            stream.write(b"You already guessed this letter!\n").unwrap();
            stream.flush().unwrap(); 
            continue;
        }

        let mut locked_game_state = game_state.lock().unwrap();

        if locked_game_state.word_to_guess.contains(guess) {
            stream.write(b"Correct guess!\n").unwrap();
            stream.flush().unwrap(); 
            for (i, letter) in word_chars.iter().enumerate() {
                if letter == &guess {
                    locked_game_state.current_state[i] = guess;
                    stream.flush().unwrap(); 
                }
            }
        } else {
            stream.write(b"Wrong guess!\n").unwrap();
            stream.flush().unwrap();
            // match attempts {
            //     1 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=========\n").unwrap(); }
            //     2 => { stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=========\n").unwrap();}
            //     3 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=========\n").unwrap();}
            //     4 => {stream.write(b"  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=========\n").unwrap();}
            //     5 => {stream.write(b"  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=========\n").unwrap();}
            //     6 => {stream.write(b"  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=========\n").unwrap();}
            //     _ => {stream.write(b"  +---+\n  |   |\n      |\n      |\n      |\n      |\n=========\n").unwrap();}
            // }

            
            attempts -= 1;
        }

        guessed_letters.push(guess);
        let current_state = locked_game_state.current_state.iter().collect::<String>();
        stream.write(current_state.as_bytes()).unwrap();
        stream.write(b"\n").unwrap();
        stream.flush().unwrap(); 
        sender.send(format!("Player: {} - {}", stream.peer_addr().unwrap(), current_state)).unwrap();
        if !locked_game_state.current_state.contains(&'_') {
            // Le joueur a deviné le mot
            sender.send(format!("Player won: {}", stream.peer_addr().unwrap())).unwrap();
            return;
        }
    }

    // Le joueur a épuisé ses tentatives
    sender.send(format!("Player lost: {}", stream.peer_addr().unwrap())).unwrap();
}

struct GameState {
    word_to_guess: String,
    current_state: Vec<char>,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080...");

    let word = random_word::gen(Lang::Fr);
    let word_to_guess = word.to_string();
    let initial_state = Arc::new(Mutex::new(GameState {
        word_to_guess: word_to_guess.clone(),
        current_state: vec!['_'; word_to_guess.len()],
    }));

    let (sender, receiver) = std::sync::mpsc::channel::<String>();

    // Démarrer un thread pour gérer les mises à jour du jeu
    thread::spawn(move || {
        for msg in receiver {
            // Envoyer des mises à jour du jeu à tous les joueurs
            
            println!("{}", msg);
        }
    });

    for stream in listener.incoming() {
        let stream = stream.expect("failed");
        let word_to_guess = word_to_guess.clone();
        let game_state = initial_state.clone();
        let sender = sender.clone();

        thread::spawn(move || {
            handle_client(stream, word_to_guess, game_state, sender);
        });
    }
}
