use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    Heartbeat,
    EventRequest(EventRequest),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage {
    StateUpdate(GameView),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EventRequest {
    StartGame,
    AddPlayer { name: String },
    MovePlayer { new_group: Group },
    RemovePlayer,
    Clue { word: String, count: u8 },
    Guess { tile_index: u8 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameView {
    pub is_started: bool,
    pub tiles: [Tile; 25],
    pub teams: Teams,
    pub this_player: Player,
    pub team_turn: TeamColour,
    pub next_action: Action,
    pub history: Vec<GameEvent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Clue {
    pub word: String,
    pub count: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Guess {
    pub tile_index: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GameEvent {
    Clue(Clue),
    Guess(Guess),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum TileColour {
    Red,
    Blue,
    Grey,
    Black,
}

impl Display for TileColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileColour::Red => write!(f, "Red"),
            TileColour::Blue => write!(f, "Blue"),
            TileColour::Grey => write!(f, "Grey"),
            TileColour::Black => write!(f, "Black"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum TeamColour {
    Red,
    Blue,
}

impl TeamColour {
    pub fn other(&self) -> TeamColour {
        match self {
            TeamColour::Red => TeamColour::Blue,
            TeamColour::Blue => TeamColour::Red,
        }
    }
}

impl rand::distributions::Distribution<TeamColour> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> TeamColour {
        if rng.gen() {
            TeamColour::Blue
        } else {
            TeamColour::Red
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Action {
    Clue,
    Guess,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Teams {
    pub blue: Team,
    pub red: Team,
    pub spectators: HashMap<String, Player>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Team {
    pub spy_masters: HashMap<String, Player>,
    pub guessers: HashMap<String, Player>,
}

impl Default for Team {
    fn default() -> Self {
        Self {
            guessers: Default::default(),
            spy_masters: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Player {
    pub name: String,
    pub group: Group,
    pub is_host: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Role {
    SpyMaster,
    Guesser,
    Spectator,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Group {
    Spectators,
    BlueGuessers,
    BlueSpyMasters,
    RedGuessers,
    RedSpyMasters,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tile {
    pub word: String,
    pub colour: Option<TileColour>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewGameRequest {
    pub player_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewGameResponse {
    pub game_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JoinGameRequest {
    pub game_id: String,
    pub player_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerJoinedResponse {
    pub already_joined: bool,
}
