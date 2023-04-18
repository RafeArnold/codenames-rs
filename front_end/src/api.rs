use common::api::v1::models::{
    ClientMessage, JoinGameRequest, NewGameRequest, NewGameResponse, PlayerJoinedResponse,
    ServerMessage,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::html::Scope;

use crate::game::{Game, GameMsg};

pub async fn player_joined(game_id: String) -> PlayerJoinedResponse {
    let window = web_sys::window().unwrap();
    let protocol = window.location().protocol().unwrap();
    let host = window.location().host().unwrap();
    let player_joined_request_url = format!(
        "{}//{}/play/v1/player-joined?game-id={}",
        protocol, host, game_id
    );
    let response =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&player_joined_request_url))
            .await
            .unwrap();
    let json = wasm_bindgen_futures::JsFuture::from(
        response
            .unchecked_into::<web_sys::Response>()
            .json()
            .unwrap(),
    )
    .await
    .unwrap();
    serde_wasm_bindgen::from_value::<PlayerJoinedResponse>(json).unwrap()
}

pub fn events(game_id: String, link: &Scope<Game>) -> web_sys::WebSocket {
    let window = web_sys::window().unwrap();
    let protocol = window.location().protocol().unwrap();
    let ws_protocol = match protocol.as_str() {
        "http:" => "ws:",
        _ => "wss:",
    };
    let host = window.location().host().unwrap();

    let ws_request_url = format!(
        "{}//{}/play/v1/events?game-id={}",
        ws_protocol, host, game_id
    );
    web_sys::console::log_1(&format!("connecting to {}", ws_request_url).into());
    let websocket = web_sys::WebSocket::new(&ws_request_url).unwrap();

    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        web_sys::console::log_1(&"connection opened".into());
    });
    websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let link_clone = link.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MessageEvent| {
        web_sys::console::log_1(&format!("message received: {:?}", event.data()).into());
        let message: ServerMessage =
            serde_json::from_str(event.data().as_string().unwrap().as_str()).unwrap();
        link_clone.send_message(GameMsg::ReceiveMessage(message));
    });
    websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let link_clone = link.clone();
    let interval_callback = Closure::<dyn Fn()>::new(move || {
        link_clone.send_message(GameMsg::SendMessage(ClientMessage::Heartbeat))
    });
    let interval_id = window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            30000,
        )
        .unwrap();
    interval_callback.forget();

    let onclose_callback = Closure::<dyn FnMut()>::new(move || {
        web_sys::console::log_1(&"connection closed".into());
        window.clear_interval_with_handle(interval_id);
    });
    websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();

    websocket
}

pub async fn new_game(player_name: String) -> NewGameResponse {
    web_sys::console::log_1(
        &format!("creating new game with player name {:?}", player_name).into(),
    );
    let window = web_sys::window().unwrap();
    let protocol = window.location().protocol().unwrap();
    let host = window.location().host().unwrap();
    let request_url: String = format!("{}//{}/play/v1/new-game", protocol, host);
    let request_headers: web_sys::Headers = web_sys::Headers::new().unwrap();
    request_headers
        .append("Content-Type", "application/json")
        .unwrap();
    let mut request_opts = web_sys::RequestInit::new();
    request_opts
        .method("POST")
        .body(Some(
            &serde_json::to_string(&NewGameRequest { player_name })
                .unwrap()
                .into(),
        ))
        .headers(&request_headers);
    let response: web_sys::Response = wasm_bindgen_futures::JsFuture::from(
        window.fetch_with_str_and_init(&request_url, &request_opts),
    )
    .await
    .unwrap()
    .unchecked_into();
    let response_json = wasm_bindgen_futures::JsFuture::from(response.json().unwrap())
        .await
        .unwrap();
    serde_wasm_bindgen::from_value(response_json).unwrap()
}

pub async fn join_game(game_id: String, player_name: String) {
    web_sys::console::log_1(&format!("joining game with player name {:?}", player_name).into());
    let window = web_sys::window().unwrap();
    let protocol = window.location().protocol().unwrap();
    let host = window.location().host().unwrap();
    let request_url: String = format!("{}//{}/play/v1/join-game", protocol, host);
    let request_headers: web_sys::Headers = web_sys::Headers::new().unwrap();
    request_headers
        .append("Content-Type", "application/json")
        .unwrap();
    let mut request_opts = web_sys::RequestInit::new();
    request_opts
        .method("POST")
        .body(Some(
            &serde_json::to_string(&JoinGameRequest {
                game_id,
                player_name,
            })
            .unwrap()
            .into(),
        ))
        .headers(&request_headers);
    let response: web_sys::Response = wasm_bindgen_futures::JsFuture::from(
        window.fetch_with_str_and_init(&request_url, &request_opts),
    )
    .await
    .unwrap()
    .unchecked_into();
    assert!(response.status() == 200);
}
