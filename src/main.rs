
// By InvalidSE
// https://github.com/invalidse

use std::io;
use std::str::FromStr;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use chess;

#[derive(Debug, Serialize, Deserialize)]
struct Games {
    games: Vec<Game>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Game {
    url: String,
    pgn: Option<String>,
    time_control: String,
    end_time: i64,
    rated: bool,
    tcn: String,
    uuid: String,
    initial_setup: String,
    fen: String,
    time_class: String,
    rules: String,
    white: Player,
    black: Player
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Player {
    rating: i32,
    result: String,
    #[serde(rename = "@id")]  // rename the json key to "id"
    id: String,  // the user's ID -- the json key is "@id"
    username: String,
    uuid: String
}

fn main() {
    // Get username from user
    println!("Please enter your username: ");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read line");
    username = username.trim().to_string();
    println!("Hello, {} -- Getting user details", username);

    // Get all user's games
    let all_games = get_user_games(username.clone());

    // Count the number of bongclouds
    let bongclouds: i32 = count_bongclouds(all_games.clone(), username.clone());
    println!("Bongclouds: {}", bongclouds);

    // Count the number of times the user could have played en passant, and the number of times they did
    let (en_passant_possible, en_passant_played) = count_en_passant(all_games, username);
}

fn count_bongclouds(all_games: Vec<Game>, username: String) -> i32 {
    let mut bongclouds = 0;
    for game in all_games {

        // Check if the game was a standard game
        if game.initial_setup != "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" {
            continue;
        }

        // Get the PGN
        if game.pgn.is_none(){continue;}
        let pgn = game.pgn.unwrap();

        // Check if the bongcloud was played
        if pgn.contains(" 2. Ke2") {
            // println!("{} played the bongcloud in game {}", game.white.username, game.url);
            if game.white.username.to_lowercase() == username.to_lowercase() {
                bongclouds += 1;
            }
        }
        if pgn.contains(" 2... Ke7") {
            // println!("{} played the bongcloud in game {}", game.black.username, game.url);
            if game.black.username.to_lowercase() == username.to_lowercase() {
                bongclouds += 1;
            }
        }
    }
    return bongclouds;
}

fn get_user_games(username: String) -> Vec<Game> {
    // Make request to chess.com API to get user archives
    let archives_url = format!("https://api.chess.com/pub/player/{}/games/archives", username);
    let response = reqwest::blocking::get(&archives_url).unwrap().text().unwrap();
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    let archives = match json["archives"].as_array() {
        Some(archives) => archives,
        None => {
            println!("No archives found for user {}", username);
            return Vec::new();
        }
    };

    // List of games
    let mut all_games: Vec<Game> = Vec::new();

    // For each archive, get the games
    for archive in archives {
        let archive_url = archive.as_str().unwrap();
        let response = reqwest::blocking::get(archive_url).unwrap().text().unwrap();
        let games = serde_json::from_str::<Games>(&response).unwrap().games;
    
        // Add games to list
        all_games.extend(games);
    }
    return all_games;
}

fn count_en_passant(all_games: Vec<Game>, username: String) -> (i32, i32) {
    let mut en_passant_possible = 0;
    let mut en_passant_played = 0;
    for game in all_games {
        println!("Game: {}", game.url);

        // Get the PGN
        if game.pgn.is_none(){continue;}
        let pgn = game.pgn.unwrap();

        // Check if the en passant was played by making a chess board and checking if the move is legal
        // let board = chess::Board::from_str(game.fen.as_str()).unwrap();
        let mut board = chess::Board::default();

        // This means we need to cut the PGN into moves and remove the excess
        if pgn.contains("[Variant "){continue;}

        // Split the PGN into moves
        let pgn = pgn.split("\n1. ").collect::<Vec<&str>>()[1];
        let split_pgn = pgn.split_whitespace();

        // Cut out every one that contains either a "." or a "{" or "}" or "1-0" or "0-1" or "1/2-1/2"
        let moves = split_pgn.filter(|x: &&str| !x.contains(".") && !x.contains("{") && !x.contains("}") && !x.contains("1-0") && !x.contains("0-1") && !x.contains("1/2-1/2")).collect::<Vec<&str>>();

        // remove any "=" from any of the moves
        let moves = moves.iter().map(|x| x.replace("=", "")).collect::<Vec<String>>();
        
        // Play the moves
        for m in moves {
            // Make a move
            println!("Move: {}", m);
            let mv = chess::ChessMove::from_san(&board, &m).expect("Invalid move");
            board = board.make_move_new(mv);

            // Check if en passant is possible
            if board.en_passant() == Some(mv.get_dest()){
                println!("En passant possible");

                // Check who's move it is
                if board.side_to_move() == chess::Color::White {
                    if game.white.username.to_lowercase() == username.to_lowercase() {
                        en_passant_possible += 1;
                    }
                } else {
                    if game.black.username.to_lowercase() == username.to_lowercase() {
                        en_passant_possible += 1;
                    }
                }

            }

        }
        
    }
    return (en_passant_possible, en_passant_played);
}
