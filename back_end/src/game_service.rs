use common::api::v1::models::{Clue, Group, Guess, Player, TeamColour, TileColour};
use rand::{seq::SliceRandom, Rng};
use tokio::sync::Mutex;

use crate::{
    game::{CodeNamesError, Game, Result, Tile},
    game_repo::GameRepository,
};

const WORDS: &'static [u8; 3283] = include_bytes!("../wordlist-eng.json");

pub struct GameService {
    repo: Mutex<GameRepository>,
}

impl GameService {
    pub fn new(repo: GameRepository) -> Self {
        Self {
            repo: Mutex::new(repo),
        }
    }

    pub async fn new_game(&self, player_id: String) -> String {
        let mut rng = rand::rngs::OsRng::default();
        // TODO: add nicer game id generator.
        let game_id = uuid::Uuid::new_v4().simple().to_string();
        let first_turn = rng.gen();
        let game = Game::new(
            Self::generate_tiles(&mut rng, &first_turn),
            player_id,
            first_turn,
        );
        self.repo.lock().await.set(game_id.as_str(), &game);
        game_id
    }

    pub async fn start_game(&self, game_id: &str, player_id: &str) -> Result<Game> {
        self.perform_request(game_id, |game| game.start(player_id))
            .await
    }

    pub async fn player_exists(&self, game_id: &str, player_id: &str) -> Result<bool> {
        self.get_game(game_id)
            .await
            .map(|game| game.player_exists(player_id))
    }

    pub async fn add_player(&self, game_id: &str, player_id: &str, player: Player) -> Result<Game> {
        self.perform_request(game_id, |game| game.add_player(player_id, player))
            .await
    }

    pub async fn move_player(
        &self,
        game_id: &str,
        player_id: &str,
        new_group: Group,
    ) -> Result<Game> {
        self.perform_request(game_id, |game| game.move_player(player_id, new_group))
            .await
    }

    pub async fn remove_player(&self, game_id: &str, player_id: &str) -> Result<Game> {
        let game = self
            .perform_request(game_id, |game| game.remove_player(player_id))
            .await;
        if let Ok(game) = game.as_ref() {
            if game.teams.spectators.is_empty()
                && game.teams.blue.guessers.is_empty()
                && game.teams.blue.spy_masters.is_empty()
                && game.teams.red.guessers.is_empty()
                && game.teams.red.spy_masters.is_empty()
            {
                self.remove_game(game_id).await
            }
        }
        game
    }

    pub async fn provide_clue(&self, game_id: &str, player_id: &str, clue: Clue) -> Result<Game> {
        self.perform_request(game_id, |game| game.provide_clue(player_id, clue))
            .await
    }

    pub async fn guess(&self, game_id: &str, player_id: &str, guess: Guess) -> Result<Game> {
        self.perform_request(game_id, |game| game.guess(player_id, guess))
            .await
    }

    pub async fn get_game(&self, game_id: &str) -> Result<Game> {
        self.repo
            .lock()
            .await
            .get(game_id)
            .ok_or(CodeNamesError::NoSuchGameError)
    }

    fn generate_tiles<R: Rng>(rng: &mut R, first_turn: &TeamColour) -> [Tile; 25] {
        let mut tile_colours: [TileColour; 25] = [
            TileColour::Black,
            TileColour::Blue,
            TileColour::Blue,
            TileColour::Blue,
            TileColour::Blue,
            TileColour::Blue,
            TileColour::Blue,
            TileColour::Blue,
            TileColour::Blue,
            match first_turn {
                TeamColour::Red => TileColour::Red,
                TeamColour::Blue => TileColour::Blue,
            },
            TileColour::Red,
            TileColour::Red,
            TileColour::Red,
            TileColour::Red,
            TileColour::Red,
            TileColour::Red,
            TileColour::Red,
            TileColour::Red,
            TileColour::Grey,
            TileColour::Grey,
            TileColour::Grey,
            TileColour::Grey,
            TileColour::Grey,
            TileColour::Grey,
            TileColour::Grey,
        ];
        tile_colours.shuffle(rng);
        let mut words: Vec<&'static str> = serde_json::from_slice(WORDS).unwrap();
        words.shuffle(rng);
        std::array::from_fn(|index| Tile {
            word: words.get(index).unwrap().to_string(),
            colour: tile_colours.get(index).unwrap().clone(),
        })
    }

    async fn remove_game(&self, game_id: &str) {
        self.repo.lock().await.del(game_id);
    }

    async fn perform_request<F>(&self, game_id: &str, f: F) -> Result<Game>
    where
        F: FnOnce(&mut Game) -> Result<()>,
    {
        let mut repo = self.repo.lock().await;
        let mut game = repo.get(game_id).ok_or(CodeNamesError::NoSuchGameError)?;
        f(&mut game)?;
        repo.set(game_id, &game);
        Ok(game.clone())
    }
}
