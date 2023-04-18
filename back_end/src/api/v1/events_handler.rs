use crate::game::{Game, Result};
use crate::game_service::GameService;
use common::api::v1::models::{
    ClientMessage, EventRequest, GameView, Group, Player, ServerMessage, Tile, Clue, Guess,
};
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use super::{game_id_query_param, player_id_cookie};

pub fn route(game_service: Arc<GameService>) -> BoxedFilter<(impl Reply,)> {
    Filter::boxed(
        warp::path!("events")
            .and(warp::ws())
            .and(player_id_cookie())
            .and(game_id_query_param())
            .map(move |ws, player_id, game_id| {
                handle_ws_request(game_service.clone(), ws, player_id, game_id)
            }),
    )
}

fn handle_ws_request(
    game_service: Arc<GameService>,
    ws: warp::ws::Ws,
    player_id: String,
    game_id: String,
) -> impl Reply {
    let player_id: String = player_id.clone();
    let game_id: String = game_id.clone();
    ws.on_upgrade(move |socket| {
        EventsHandler::new(game_service).handle_socket(socket, player_id, game_id)
    })
}

struct EventsHandler {
    connections: Connections,
    game_service: Arc<GameService>,
}

impl EventsHandler {
    pub fn new(game_service: Arc<GameService>) -> Self {
        Self {
            connections: Default::default(),
            game_service,
        }
    }

    async fn handle_socket(self, socket: warp::ws::WebSocket, player_id: String, game_id: String) {
        let (sink, mut stream) = socket.split();

        let mut connections = self.connections.lock().await;
        if !connections.contains_key(game_id.as_str()) {
            connections.insert(game_id.clone(), Default::default());
        }
        let game_connections = connections.get_mut(game_id.as_str()).unwrap();
        game_connections.insert(player_id.clone(), sink);
        drop(connections);

        // Send current game view upon connection.
        match self.game_service.get_game(game_id.as_str()).await {
            Ok(game) => self.send_state_update(game_id.as_str(), game).await,
            Err(err) => eprintln!("Error retrieving game: {}", err),
        }

        while let Some(message_result) = stream.next().await {
            match message_result {
                Ok(message) => {
                    println!("Received message: {:?}", message);
                    self.handle_message(message, game_id.as_str(), player_id.as_str())
                        .await
                }
                Err(err) => eprintln!("Error receiving WebSocket message: {}", err),
            }
        }

        self.connections.lock().await.remove(game_id.as_str());
    }

    async fn handle_message(&self, message: warp::ws::Message, game_id: &str, player_id: &str) {
        if message.is_text() {
            self.handle_text_message(message.to_str().unwrap(), game_id, player_id)
                .await
        } else {
            eprintln!("Unrecognised message: {:?}", message)
        }
    }

    async fn handle_text_message(&self, text: &str, game_id: &str, player_id: &str) {
        match serde_json::from_str::<ClientMessage>(text) {
            Ok(message) => match message {
                ClientMessage::EventRequest(event_req) => {
                    match self.handle_request(event_req, game_id, player_id).await {
                        Ok(response) => self.send_state_update(game_id, response).await,
                        Err(err) => eprintln!("Error performing game event request: {err}"),
                    }
                }
                ClientMessage::Heartbeat => {}
            },
            Err(err) => eprintln!("Error parsing WebSocket message to game event request: {err}"),
        }
    }

    async fn handle_request(
        &self,
        request: EventRequest,
        game_id: &str,
        player_id: &str,
    ) -> Result<Game> {
        match request {
            EventRequest::StartGame => self.game_service.start_game(game_id, player_id).await,
            EventRequest::AddPlayer { name } => {
                self.game_service
                    .add_player(
                        game_id,
                        player_id,
                        Player {
                            name,
                            group: Group::Spectators,
                            is_host: false,
                        },
                    )
                    .await
            }
            EventRequest::MovePlayer { new_group } => {
                self.game_service
                    .move_player(game_id, player_id, new_group)
                    .await
            }
            EventRequest::RemovePlayer => self.game_service.remove_player(game_id, player_id).await,
            EventRequest::Clue { word, count } => {
                self.game_service
                    .provide_clue(game_id, player_id, Clue { word, count })
                    .await
            }
            EventRequest::Guess { tile_index } => {
                self.game_service
                    .guess(game_id, player_id, Guess { tile_index })
                    .await
            }
        }
    }

    async fn send_state_update(&self, game_id: &str, game: Game) {
        match self.connections.lock().await.get_mut(game_id) {
            Some(conns) => {
                for (player_id, sink) in conns {
                    let is_spymaster = game.teams.red.spy_masters.contains_key(player_id)
                        || game.teams.blue.spy_masters.contains_key(player_id);
                    let json = if is_spymaster {
                        serde_json::to_string(&ServerMessage::StateUpdate(GameView::from_game(
                            game.clone(),
                            true,
                            player_id.as_str(),
                        )))
                    } else {
                        serde_json::to_string(&ServerMessage::StateUpdate(GameView::from_game(
                            game.clone(),
                            false,
                            player_id.as_str(),
                        )))
                    }
                    .expect("Failed to serialize game state update");
                    let json = json.as_str();
                    println!("Sending {:?} to player {}", json, player_id);
                    match sink.send(warp::ws::Message::text(json)).await {
                        Ok(_) => println!(
                            "Successfully sent state update {} to player {}",
                            json, player_id
                        ),
                        Err(err) => {
                            eprintln!("Failed to send state update {}: {}", json, err)
                        }
                    }
                }
            }
            None => {
                panic!("No connections found for game {game_id}")
            }
        }
    }
}

trait FromGame {
    fn from_game(game: Game, is_spymaster: bool, player_id: &str) -> Self;
}

impl FromGame for GameView {
    fn from_game(game: Game, is_spymaster: bool, player_id: &str) -> Self {
        let tiles = game.tiles.map(|tile| Tile {
            word: tile.word,
            colour: if is_spymaster {
                Some(tile.colour)
            } else {
                None
            },
        });

        let this_player = game
            .teams
            .spectators
            .get(player_id)
            .or_else(|| game.teams.blue.guessers.get(player_id))
            .or_else(|| game.teams.blue.spy_masters.get(player_id))
            .or_else(|| game.teams.red.guessers.get(player_id))
            .or_else(|| game.teams.red.spy_masters.get(player_id))
            .expect("Player not found")
            .clone();

        Self {
            is_started: game.is_started,
            tiles,
            teams: game.teams,
            this_player,
            team_turn: game.team_turn,
            next_action: game.next_action,
            history: game.history,
        }
    }
}

type Connections = Arc<Mutex<HashMap<String, GameConnections>>>;

type GameConnections = HashMap<String, SplitSink<warp::ws::WebSocket, warp::ws::Message>>;