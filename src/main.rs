
// By InvalidSE
// https://github.com/invalidse

use std::io;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct Games {
    games: Vec<Game>
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

    let mut bongclouds = 0;

    for game in all_games {

        if game.initial_setup != "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" {
            continue;
        }

        // Get the PGN
        if game.pgn.is_none(){continue;}
        let pgn = game.pgn.unwrap();

        // Check if the bongcloud was played
        if pgn.contains(" 2. Ke2") {
            println!("{} played the bongcloud in game {}", game.white.username, game.url);
            if game.white.username.to_lowercase() == username.to_lowercase() {
                bongclouds += 1;
            }
        }
        if pgn.contains(" 2... Ke7") {
            println!("{} played the bongcloud in game {}", game.black.username, game.url);
            if game.black.username.to_lowercase() == username.to_lowercase() {
                bongclouds += 1;
            }
        }
    }

    println!("Bongclouds: {}", bongclouds);

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


