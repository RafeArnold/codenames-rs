use std::collections::HashMap;

use common::api::v1::models::{
    Action, ClientMessage, Clue, EventRequest, GameEvent, GameView, Group, Player, ServerMessage,
    TeamColour,
};
use futures::FutureExt;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::api::{events, player_joined};

pub struct Game {
    websocket: Option<web_sys::WebSocket>,
    view: Option<GameView>,
    clue_input: ClueInput,
}

#[derive(Clone, Debug)]
pub struct ClueInput {
    word: String,
    count: Option<u8>,
}

impl Game {
    fn render_game_view(&self, view: &GameView, ctx: &Context<Self>) -> Html {
        fn concat_player_names(players: &HashMap<String, Player>) -> String {
            players
                .values()
                .map(|player| player.name.to_owned())
                .reduce(|acc, player_name| acc + ", " + player_name.as_str())
                .unwrap_or_default()
        }
        let spectator_names = concat_player_names(&view.teams.spectators);
        let blue_guessers_names = concat_player_names(&view.teams.blue.guessers);
        let blue_spy_masters_names = concat_player_names(&view.teams.blue.spy_masters);
        let red_guessers_names = concat_player_names(&view.teams.red.guessers);
        let red_spy_masters_names = concat_player_names(&view.teams.red.spy_masters);

        let move_player = ctx.link().callback(|new_group: Group| {
            GameMsg::SendMessage(ClientMessage::EventRequest(EventRequest::MovePlayer {
                new_group,
            }))
        });
        let move_player_clone = move_player.clone();
        let join_spectators = move |_| move_player_clone.emit(Group::Spectators);
        let move_player_clone = move_player.clone();
        let join_blue_guessers = move |_| move_player_clone.emit(Group::BlueGuessers);
        let move_player_clone = move_player.clone();
        let join_blue_spy_masters = move |_| move_player_clone.emit(Group::BlueSpyMasters);
        let move_player_clone = move_player.clone();
        let join_red_guessers = move |_| move_player_clone.emit(Group::RedGuessers);
        let move_player_clone = move_player.clone();
        let join_red_spy_masters = move |_| move_player_clone.emit(Group::RedSpyMasters);

        let enough_players_to_start = view.teams.blue.spy_masters.len() > 0
            && view.teams.blue.guessers.len() > 0
            && view.teams.red.spy_masters.len() > 0
            && view.teams.red.guessers.len() > 0;

        let start_game = ctx.link().callback(|()| {
            GameMsg::SendMessage(ClientMessage::EventRequest(EventRequest::StartGame))
        });

        let guess = ctx.link().callback(|tile_index| {
            GameMsg::SendMessage(ClientMessage::EventRequest(EventRequest::Guess {
                tile_index,
            }))
        });

        let last_clue: Option<&Clue> = view
            .history
            .iter()
            .filter_map(|event| match event {
                GameEvent::Clue(clue) => Some(clue),
                GameEvent::Guess(_) => None,
            })
            .last();

        let clue_input = self.clue_input.clone();
        let provide_clue = ctx.link().batch_callback(move |_| {
            clue_input.count.map(|count| {
                GameMsg::SendMessage(ClientMessage::EventRequest(EventRequest::Clue {
                    word: clue_input.word.clone(),
                    count: count,
                }))
            })
        });
        let clue_count = self.clue_input.count.clone();
        let set_clue_word = ctx.link().callback(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            GameMsg::SetClueInput(ClueInput {
                word: value,
                count: clue_count,
            })
        });
        let clue_word = self.clue_input.word.clone();
        let set_clue_count = ctx.link().callback(move |event: InputEvent| {
            let value = event
                .target_unchecked_into::<HtmlInputElement>()
                .value()
                .parse()
                .ok();
            GameMsg::SetClueInput(ClueInput {
                word: clue_word.clone(),
                count: value,
            })
        });

        html! {
            <div>
                <h2>{"players"}</h2>
                <table>
                    <thead>
                        <tr>
                            <th>{"spectators"}</th>
                            <th>{"blue"}</th>
                            <th>{"red"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>
                                <p>{spectator_names}</p>
                                if !view.is_started && view.this_player.group != Group::Spectators {
                                    <p>
                                        <button onclick={join_spectators}>{"join spectators"}</button>
                                    </p>
                                }
                            </td>
                            <td>
                                <p><b>{"guessers"}</b></p>
                                <p>{blue_guessers_names}</p>
                                if !view.is_started && view.this_player.group != Group::BlueGuessers {
                                    <p>
                                        <button onclick={join_blue_guessers}>{"join blue guessers"}</button>
                                    </p>
                                }
                                <p><b>{"spy masters"}</b></p>
                                <p>{blue_spy_masters_names}</p>
                                if !view.is_started && view.this_player.group != Group::BlueSpyMasters {
                                    <p>
                                        <button onclick={join_blue_spy_masters}>{"join blue spy masters"}</button>
                                    </p>
                                }
                            </td>
                            <td>
                                <p><b>{"guessers"}</b></p>
                                <p>{red_guessers_names}</p>
                                if !view.is_started && view.this_player.group != Group::RedGuessers {
                                    <p>
                                        <button onclick={join_red_guessers}>{"join red guessers"}</button>
                                    </p>
                                }
                                <p><b>{"spy masters"}</b></p>
                                <p>{red_spy_masters_names}</p>
                                if !view.is_started && view.this_player.group != Group::RedSpyMasters {
                                    <p>
                                        <button onclick={join_red_spy_masters}>{"join red spy masters"}</button>
                                    </p>
                                }
                            </td>
                        </tr>
                    </tbody>
                </table>
                if !view.is_started && view.this_player.is_host && enough_players_to_start {
                    <p>
                        <button onclick={move |_| start_game.emit(())}>{"start game"}</button>
                    </p>
                }
                if view.is_started {
                    <table>
                        <tbody>
                            {
                                for view.tiles.chunks(5).enumerate().map(|(row_index, row_tiles)| {
                                    html! {
                                        <tr key={row_index}>
                                            {
                                                for row_tiles.iter().enumerate().map(|(column_index, tile)| {
                                                    let index = row_index * 5 + column_index;
                                                    let guess = guess.clone();
                                                    html! {
                                                        <td key={column_index}>
                                                            if let Some(colour) = &tile.colour {
                                                                {tile.word.clone() + " " + colour.to_string().as_str()}
                                                            } else {
                                                                {&tile.word}
                                                                if view.next_action == Action::Guess &&
                                                                    match view.team_turn {
                                                                        TeamColour::Red => view.this_player.group == Group::RedGuessers,
                                                                        TeamColour::Blue => view.this_player.group == Group::BlueGuessers,
                                                                    }
                                                                {
                                                                    <button onclick={move |_| guess.clone().emit(index.try_into().unwrap())}>{"guess"}</button>
                                                                }
                                                            }
                                                        </td>
                                                    }
                                                })
                                            }
                                        </tr>
                                    }
                                })
                            }
                        </tbody>
                    </table>
                    if view.this_player.group == Group::BlueSpyMasters
                        || view.this_player.group == Group::RedSpyMasters
                    {
                        <table>
                            <tbody>
                                {
                                    for view.tiles.chunks(5).enumerate().map(|(row_index, row_tiles)| {
                                        html! {
                                            <tr key={row_index}>
                                                {
                                                    for row_tiles.iter().enumerate().map(|(column_index, tile)| {
                                                        html! {
                                                            <td key={column_index}>{tile.colour.as_ref().unwrap().to_string().as_str()}</td>
                                                        }
                                                    })
                                                }
                                            </tr>
                                        }
                                    })
                                }
                            </tbody>
                        </table>
                    }
                    if view.next_action == Action::Guess &&
                        match view.team_turn {
                            TeamColour::Red => view.this_player.group == Group::RedGuessers,
                            TeamColour::Blue => view.this_player.group == Group::BlueGuessers,
                        }
                    {
                        <label>{"word"}
                            <input type={"text"} oninput={set_clue_word} value={self.clue_input.word.clone()}/>
                        </label>
                        <label>{"count"}
                            <input type={"number"} oninput={set_clue_count} min={"1"} max={"9"} value={self.clue_input.count.map(|count| count.to_string())}/>
                        </label>
                        if let Some(clue_count) = self.clue_input.count {
                            if self.clue_input.word.trim().len() > 0 && clue_count > 0 {
                                <button onclick={provide_clue}>{"submit clue"}</button>
                            }
                        }
                    }
                    if let Some(last_clue) = last_clue {
                        if view.next_action == Action::Guess {
                            {format!("CLUE: {} - {}", last_clue.word, last_clue.count)}
                        }
                    }
                }
            </div>
        }
    }
}

#[derive(Debug)]
pub enum GameMsg {
    ReceiveMessage(ServerMessage),
    SendMessage(ClientMessage),
    PlayerJoined(bool),
    SetClueInput(ClueInput),
}

#[derive(PartialEq, Properties)]
pub struct GameProps {
    pub game_id: String,
    pub force_join: Callback<()>,
}

impl Component for Game {
    type Message = GameMsg;
    type Properties = GameProps;

    fn create(ctx: &Context<Self>) -> Self {
        let GameProps {
            game_id,
            force_join: _,
        } = ctx.props();

        ctx.link().send_future(
            player_joined(game_id.clone())
                .map(|response| GameMsg::PlayerJoined(response.already_joined)),
        );

        Self {
            websocket: None,
            view: None,
            clue_input: ClueInput { word: "".to_string(), count: None },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            if self.view.is_some() {
                {self.render_game_view(self.view.as_ref().unwrap(), ctx)}
            } else {
                <h1>{"loading..."}</h1>
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        web_sys::console::log_1(&format!("received message: {:?}", msg).into());
        match msg {
            GameMsg::ReceiveMessage(message) => match message {
                ServerMessage::StateUpdate(view) => {
                    self.view = Some(view);
                    true
                }
            },
            GameMsg::SendMessage(message) => {
                if let Some(websocket) = self.websocket.as_ref() {
                    websocket
                        .send_with_str(serde_json::to_string(&message).unwrap().as_str())
                        .unwrap();
                }
                false
            }
            GameMsg::PlayerJoined(player_joined) => {
                if player_joined {
                    self.websocket = Some(events(ctx.props().game_id.clone(), ctx.link()));
                } else {
                    ctx.props().force_join.emit(());
                }
                false
            }
            GameMsg::SetClueInput(clue_input) => {
                self.clue_input = clue_input;
                true
            }
        }
    }
}
