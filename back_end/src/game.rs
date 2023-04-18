use common::api::v1::models::{
    Action, GameEvent, Group, Player, Team, TeamColour, Teams, TileColour, Clue, Guess,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Game {
    pub is_started: bool,
    pub tiles: [Tile; 25],
    pub teams: Teams,
    pub host_id: String,
    pub team_turn: TeamColour,
    pub next_action: Action,
    pub history: Vec<GameEvent>,
}

impl Game {
    pub fn new(tiles: [Tile; 25], host_id: String, first_turn: TeamColour) -> Game {
        Game {
            is_started: false,
            tiles,
            teams: Teams {
                blue: Default::default(),
                red: Default::default(),
                spectators: Default::default(),
            },
            host_id,
            team_turn: first_turn,
            next_action: Action::Clue,
            history: vec![],
        }
    }

    pub fn start(&mut self, player_id: &str) -> Result<()> {
        self.validate_game_has_not_started()?;
        if self.host_id != player_id {
            return Err(CodeNamesError::NotHostError);
        }
        self.validate_team(&self.teams.blue)?;
        self.validate_team(&self.teams.red)?;
        self.is_started = true;
        Ok(())
    }

    pub fn player_exists(&self, player_id: &str) -> bool {
        self.teams.spectators.contains_key(player_id)
            || self.teams.blue.spy_masters.contains_key(player_id)
            || self.teams.blue.guessers.contains_key(player_id)
            || self.teams.red.spy_masters.contains_key(player_id)
            || self.teams.red.guessers.contains_key(player_id)
    }

    pub fn add_player(&mut self, player_id: &str, player: Player) -> Result<()> {
        if self.player_exists(player_id) {
            return Err(CodeNamesError::PlayerAlreadyInGameError);
        }
        self.teams.spectators.insert(player_id.to_string(), player);
        Ok(())
    }

    pub fn move_player(&mut self, player_id: &str, new_group: Group) -> Result<()> {
        let mut player = self.get_player_group(player_id)?.remove(player_id).unwrap();
        let group_to_move_to = match new_group {
            Group::Spectators => &mut self.teams.spectators,
            Group::BlueGuessers => &mut self.teams.blue.guessers,
            Group::BlueSpyMasters => &mut self.teams.blue.spy_masters,
            Group::RedGuessers => &mut self.teams.red.guessers,
            Group::RedSpyMasters => &mut self.teams.red.spy_masters,
        };
        player.group = new_group;
        group_to_move_to.insert(player_id.to_string(), player);
        Ok(())
    }

    pub fn remove_player(&mut self, player_id: &str) -> Result<()> {
        self.get_player_group(player_id)?.remove(player_id).unwrap();
        Ok(())
    }

    pub fn provide_clue(&mut self, player_id: &str, clue: Clue) -> Result<()> {
        self.validate_game_has_started()?;
        self.validate_action(Action::Clue)?;
        self.validate_player(player_id)?;
        self.history.push(GameEvent::Clue(clue));
        self.next_action = Action::Guess;
        Ok(())
    }

    pub fn guess(&mut self, player_id: &str, guess: Guess) -> Result<()> {
        self.validate_game_has_started()?;
        self.validate_action(Action::Guess)?;
        self.validate_player(player_id)?;
        self.history.push(GameEvent::Guess(guess));
        self.next_action = Action::Clue;
        Ok(())
    }

    fn validate_team<'b>(&self, team: &Team) -> Result<()> {
        if team.spy_masters.len() != 1 {
            Err(CodeNamesError::NotEnoughPlayersError)
        } else if team.guessers.len() == 0 {
            Err(CodeNamesError::NotEnoughPlayersError)
        } else {
            Ok(())
        }
    }

    fn validate_player(&self, player_id: &str) -> Result<&Player> {
        let team = match self.team_turn {
            TeamColour::Red => &self.teams.red,
            TeamColour::Blue => &self.teams.blue,
        };
        match self.next_action {
            Action::Clue => &team.spy_masters,
            Action::Guess => &team.guessers,
        }
        .get(player_id)
        .ok_or_else(|| {
            let exp_group: Group = match self.next_action {
                Action::Clue => match self.team_turn {
                    TeamColour::Red => Group::RedSpyMasters,
                    TeamColour::Blue => Group::RedGuessers,
                },
                Action::Guess => match self.team_turn {
                    TeamColour::Red => Group::BlueSpyMasters,
                    TeamColour::Blue => Group::BlueGuessers,
                },
            };
            let actual: Option<Group> = if self.teams.spectators.contains_key(player_id) {
                Some(Group::Spectators)
            } else if self.teams.blue.spy_masters.contains_key(player_id) {
                Some(Group::BlueSpyMasters)
            } else if self.teams.blue.guessers.contains_key(player_id) {
                Some(Group::BlueGuessers)
            } else if self.teams.red.spy_masters.contains_key(player_id) {
                Some(Group::RedSpyMasters)
            } else if self.teams.red.guessers.contains_key(player_id) {
                Some(Group::RedGuessers)
            } else {
                None
            };
            match actual {
                Some(act_group) => {
                    if exp_group != act_group {
                        CodeNamesError::IllegalPlayerGroupError {
                            exp_group,
                            act_group,
                        }
                    } else {
                        panic!("The player is in the correct group, but could not be found")
                    }
                }
                None => CodeNamesError::NoSuchPlayerError,
            }
        })
    }

    fn validate_game_has_not_started<'b>(&self) -> Result<()> {
        if self.is_started {
            Err(CodeNamesError::GameAlreadyStartedError)
        } else {
            Ok(())
        }
    }

    fn validate_game_has_started<'b>(&self) -> Result<()> {
        if !self.is_started {
            Err(CodeNamesError::GameNotStartedError)
        } else {
            Ok(())
        }
    }

    fn validate_action<'b>(&self, request_action: Action) -> Result<()> {
        if self.next_action != request_action {
            Err(CodeNamesError::InvalidActionError)
        } else {
            Ok(())
        }
    }

    fn get_player_group(&mut self, player_id: &str) -> Result<&mut HashMap<String, Player>> {
        fn contains_player<'a>(
            players: &'a mut HashMap<String, Player>,
            player_id: &str,
        ) -> Option<&'a mut HashMap<String, Player>> {
            if players.contains_key(player_id) {
                Some(players)
            } else {
                None
            }
        }
        contains_player(&mut self.teams.spectators, player_id)
            .or_else(|| contains_player(&mut self.teams.blue.guessers, player_id))
            .or_else(|| contains_player(&mut self.teams.blue.spy_masters, player_id))
            .or_else(|| contains_player(&mut self.teams.red.guessers, player_id))
            .or_else(|| contains_player(&mut self.teams.red.spy_masters, player_id))
            .ok_or(CodeNamesError::NoSuchPlayerError)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tile {
    pub word: String,
    pub colour: TileColour,
}

#[derive(Debug)]
pub enum CodeNamesError {
    GameAlreadyStartedError,
    GameNotStartedError,
    IllegalPlayerGroupError { exp_group: Group, act_group: Group },
    InvalidActionError,
    NoSuchGameError,
    NoSuchPlayerError,
    NotEnoughPlayersError,
    NotHostError,
    PlayerAlreadyInGameError,
}

impl Display for CodeNamesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeNamesError::GameAlreadyStartedError => write!(f, "Game has already started"),
            CodeNamesError::GameNotStartedError => write!(f, "Game has not yet started"),
            CodeNamesError::IllegalPlayerGroupError {
                exp_group,
                act_group,
            } => {
                let fmt_group = |group: &Group| match group {
                    Group::Spectators => "Spectator",
                    Group::BlueGuessers => "Blue Guesser",
                    Group::BlueSpyMasters => "Blue Spy Master",
                    Group::RedGuessers => "Red Guesser",
                    Group::RedSpyMasters => "Red Spy Master",
                };
                write!(
                    f,
                    "Player must be a {} to perform this action, but is a {}",
                    fmt_group(exp_group),
                    fmt_group(act_group),
                )
            }
            CodeNamesError::InvalidActionError => write!(f, "Cannot perform this action"),
            CodeNamesError::NoSuchGameError => write!(f, "Game does not exist"),
            CodeNamesError::NoSuchPlayerError => write!(f, "Player is not in this game"),
            CodeNamesError::NotEnoughPlayersError => {
                write!(f, "Not enough players to perform this action")
            }
            CodeNamesError::NotHostError => {
                write!(f, "Player must be the host to perform this action")
            }
            CodeNamesError::PlayerAlreadyInGameError => write!(f, "Player is already in this game"),
        }
    }
}

impl std::error::Error for CodeNamesError {}

pub type Result<T> = core::result::Result<T, CodeNamesError>;

#[cfg(test)]
mod tests {
    use common::api::v1::models::Player;

    use crate::game::*;
    use std::array::from_fn;
    use std::collections::HashMap;

    #[test]
    fn when_enough_players_then_game_can_be_started() -> Result<()> {
        let tiles: [Tile; 25] = random_tiles();
        let player1_id = "player_1";
        let mut game: Game = Game::new(tiles, player1_id.to_string(), TeamColour::Red);
        let player1 = Player {
            name: "player_1_name".to_string(),
            group: Group::Spectators,
            is_host: false,
        };
        let player2_id = "player_2";
        let player2 = Player {
            name: "player_2_name".to_string(),
            group: Group::Spectators,
            is_host: false,
        };
        let player3_id = "player_3";
        let player3 = Player {
            name: "player_3_name".to_string(),
            group: Group::Spectators,
            is_host: false,
        };
        let player4_id = "player_4";
        let player4 = Player {
            name: "player_4_name".to_string(),
            group: Group::Spectators,
            is_host: false,
        };
        game.teams.blue.spy_masters = HashMap::from([(player1_id.to_string(), player1)]);
        game.teams.blue.guessers = HashMap::from([(player2_id.to_string(), player2)]);
        game.teams.red.spy_masters = HashMap::from([(player3_id.to_string(), player3)]);
        game.teams.red.guessers = HashMap::from([(player4_id.to_string(), player4)]);
        assert!(!game.is_started);
        game.start(player1_id)?;
        assert!(game.is_started);
        Ok(())
    }

    fn random_tiles() -> [Tile; 25] {
        from_fn(|_| Tile {
            word: "s".to_string(),
            colour: TileColour::Blue,
        })
    }
}
