use futures::FutureExt;
use yew::prelude::*;

use crate::{
    api::{join_game, new_game},
    simple_input::SimpleInput,
};

pub struct Menu {
    player_name: String,
}

#[derive(PartialEq, Properties)]
pub struct MenuProps {
    pub game_id: Option<String>,
    pub set_joined: Callback<()>,
}

#[derive(Debug)]
pub enum MenuMsg {
    SetJoined,
    SetPlayerName(String),
}

impl Component for Menu {
    type Message = MenuMsg;

    type Properties = MenuProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            player_name: "".to_owned(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let player_name = self.player_name.clone();
        let set_player_name = ctx
            .link()
            .callback(|player_name: String| MenuMsg::SetPlayerName(player_name));
        if let Some(game_id) = ctx.props().game_id.clone() {
            let onclick = ctx.link().callback_future(move |_| {
                join_game(game_id.clone(), player_name.clone()).map(|_| MenuMsg::SetJoined)
            });
            html! {
                <>
                    <SimpleInput label_name={"name:"} value={self.player_name.clone()} set_value={set_player_name}/>
                    if self.player_name.trim().len() > 0 {
                        <button {onclick}>{"join game"}</button>
                    }
                </>
            }
        } else {
            let onclick = ctx.link().callback_future(move |_| {
                new_game(player_name.clone()).map(|response| {
                    let window = web_sys::window().unwrap();
                    let protocol = window.location().protocol().unwrap();
                    let host = window.location().host().unwrap();
                    let new_url = format!(
                        "{}//{}{}?game-id={}",
                        protocol,
                        host,
                        window.location().pathname().unwrap(),
                        response.game_id
                    );
                    window
                        .history()
                        .unwrap()
                        .push_state_with_url(
                            &wasm_bindgen::JsValue::null(),
                            "",
                            Some(&new_url.as_str()),
                        )
                        .unwrap();
                    MenuMsg::SetJoined
                })
            });
            html! {
                <>
                    <SimpleInput label_name={"name:"} value={self.player_name.clone()} set_value={set_player_name}/>
                    if self.player_name.trim().len() > 0 {
                        <button {onclick}>{"new game"}</button>
                    }
                </>
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MenuMsg::SetJoined => ctx.props().set_joined.emit(()),
            MenuMsg::SetPlayerName(player_name) => self.player_name = player_name,
        };
        true
    }
}
