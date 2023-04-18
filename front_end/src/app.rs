use std::ops::Deref;

use web_sys::{window, UrlSearchParams};
use yew::prelude::*;

use crate::{game::Game, menu::Menu};

#[function_component]
pub fn App() -> Html {
    fn get_game_id() -> Option<String> {
        UrlSearchParams::new_with_str(window().unwrap().location().search().unwrap().as_str())
            .unwrap()
            .get("game-id")
    }
    let game_id_handle = use_state_eq(|| get_game_id());
    let joined_handle = use_state_eq(|| true);
    let game_id_handle_clone = game_id_handle.clone();
    let joined_handle_clone = joined_handle.clone();
    let set_joined = Callback::from(move |_| {
        joined_handle_clone.set(true);
        game_id_handle_clone.set(get_game_id());
    });
    let joined_handle_clone = joined_handle.clone();
    let force_join = Callback::from(move |_| joined_handle_clone.set(false));
    html! {
        if game_id_handle.is_some() && *joined_handle.deref() {
            <Game game_id={game_id_handle.deref().clone().unwrap()} {force_join}/>
        } else {
            <Menu game_id={game_id_handle.deref().clone()} {set_joined}/>
        }
    }
}
