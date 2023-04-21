
// By InvalidSE
// https://github.com/invalidse

use std::{io};
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

    // Count the number of times the user could have played en passant, and the number of times they did
    let (en_passant_possible, en_passant_played) = count_en_passant(all_games, username);
    println!("En Passants: {}/{}, Bongclouds: {}", en_passant_played, en_passant_possible, bongclouds);
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

    // Print amount of archives found
    println!("Found {} archives", archives.len());

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
    println!("Found {} games", all_games.len());
    for game in all_games {
        // println!("Game: {} En Passants: {}/{}", game.url, en_passant_played, en_passant_possible);

        // Get the PGN
        if game.pgn.is_none(){continue;}
        let pgn = game.pgn.unwrap();

        if game.initial_setup != "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" {
            continue;
        }

        // Check if the en passant was played by making a chess board and checking if the move is legal
        // println!("Initial Setup: {}", game.initial_setup.as_str());
        // let mut board = chess::Board::from_str(game.initial_setup.as_str()).unwrap();
        let mut board = chess::Board::default();

        // This means we need to cut the PGN into moves and remove the excess
        if pgn.contains("[Variant "){continue;}

        // If pgn does not have \n1. then the game was aborted
        if !pgn.contains("\n1. "){continue;}

        // Split the PGN into moves
        let pgn = pgn.split("\n1. ").collect::<Vec<&str>>()[1];
        let split_pgn = pgn.split_whitespace();

        // Cut out every one that contains either a "." or a "{" or "}" or "1-0" or "0-1" or "1/2-1/2", then remove the "=" from the moves
        let moves = split_pgn.filter(|x: &&str| !x.contains(".") && !x.contains("{") && !x.contains("}") && !x.contains("1-0") && !x.contains("0-1") && !x.contains("1/2-1/2")).collect::<Vec<&str>>();
        let moves = moves.iter().map(|x| x.replace("=", "")).collect::<Vec<String>>();
        let mut moves = moves.iter().map(|x| x.replace("+", "")).collect::<Vec<String>>();

        // Check for a pawn move to file 4 or 5
        let mut move_num = 0;
        for n in moves.clone() {
            move_num += 1;
            // replace any + in n with ""
            let n = n.replace("+", "");
            if n.len() == 2 {
                // Check last character was a 5
                if n.chars().last().unwrap() == '5' {
                    // check the next move is a pawn taking on the same file but rank 6 (en passant, it will look like "exd6")
                    if moves.len() == move_num {continue;}
                    if moves.clone()[move_num].replace("+", "").chars().count() == 4 {
                        if moves[move_num].replace("+", "").chars().skip(1).collect::<String>() == format!("x{}6", n.chars().next().unwrap()) {
                            // En passant taken by white, lets check if move_num is odd or even
                            if move_num % 2 == 0 {
                                // White took en passant
                                if game.white.username.to_lowercase() == username.to_lowercase() {
                                    en_passant_played += 1;
                                }
                            } else {
                                // Black took en passant
                                if game.black.username.to_lowercase() == username.to_lowercase() {
                                    en_passant_played += 1;
                                }
                            }
                            // println!("In theory, en passant was taken in game {} by {}", game.url, if move_num % 2 == 0 {&game.white.username} else {&game.black.username});
                            
                            // Add " e.p." to the move so that it doesn't crash
                            moves[move_num] = format!("{} e.p.", moves[move_num]);
                        }
                    }
                }
                // Check last character was a 4
                else if n.chars().last().unwrap() == '4' {
                    // check the next move is a pawn taking on the same file but rank 3 (en passant, it will look like "exd3")
                    if moves.len() == move_num {continue;}
                    if moves.clone()[move_num].replace("+", "").chars().count() == 4 {
                        if moves[move_num].replace("+", "").chars().skip(1).collect::<String>() == format!("x{}3", n.chars().next().unwrap()) {
                            // En passant taken by white, lets check if move_num is odd or even
                            if move_num % 2 == 0 {
                                // White took en passant
                                if game.white.username.to_lowercase() == username.to_lowercase() {
                                    en_passant_played += 1;
                                }
                            } else {
                                // Black took en passant
                                if game.black.username.to_lowercase() == username.to_lowercase() {
                                    en_passant_played += 1;
                                }
                            }
                            // println!("In theory, en passant was taken in game {} by {}", game.url, if move_num % 2 == 0 {&game.white.username} else {&game.black.username});
                            
                            // Add " e.p." to the move so that it doesn't crash
                            moves[move_num] = format!("{} e.p.", moves[move_num]);
                        }
                    }
                }
            }
        }
        
        // Play the moves
        for m in moves {
            // Make a move
            // println!("Move: {}", m);
            let mv = chess::ChessMove::from_san(&board, &m).expect("Invalid move");
            board = board.make_move_new(mv);

            // Check if en passant is possible
            if board.en_passant() == Some(mv.get_dest()){
                // println!("En passant possible");

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
