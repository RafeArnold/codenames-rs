use redis::{Client, Commands, Connection, IntoConnectionInfo, RedisResult};

use crate::game::Game;

pub struct GameRepository {
    connection: Connection,
}

impl GameRepository {
    pub fn new<T: IntoConnectionInfo>(url: T) -> RedisResult<Self> {
        let client = Client::open(url)?;
        let connection = client.get_connection()?;
        Ok(Self { connection })
    }

    pub fn get(&mut self, game_id: &str) -> Option<Game> {
        let value: Option<String> = self.connection.get(game_id).unwrap();
        value.map(|value| serde_json::from_str(value.as_str()).unwrap())
    }

    pub fn set(&mut self, game_id: &str, game: &Game) {
        self.connection
            .set(game_id, serde_json::to_string(game).unwrap())
            .unwrap()
    }

    pub fn del(&mut self, game_id: &str) {
        self.connection.del(game_id).unwrap()
    }
}
