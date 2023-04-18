use crate::api::v1::player_id_cookie;
use crate::game::CodeNamesError;
use crate::game_service::GameService;
use common::api::v1::models::PlayerJoinedResponse;
use std::sync::Arc;
use warp::reply::Reply;
use warp::Filter;

use super::game_id_query_param;

pub fn route(game_service: Arc<GameService>) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    Filter::boxed(
        warp::get()
            .and(warp::path!("player-joined"))
            .and(player_id_cookie())
            .and(game_id_query_param())
            .then(move |player_id, game_id| {
                handle_request(game_service.clone(), player_id, game_id)
            }),
    )
}

async fn handle_request(
    game_service: Arc<GameService>,
    player_id: String,
    game_id: String,
) -> warp::reply::Response {
    match game_service
        .player_exists(game_id.as_str(), player_id.as_str())
        .await
    {
        Ok(already_joined) => warp::reply::json(&PlayerJoinedResponse { already_joined }).into_response(),
        Err(err) => match err {
            CodeNamesError::NoSuchGameError => warp::http::Response::builder()
                .status(warp::http::StatusCode::NOT_FOUND)
                .body(format!("No game with ID {} found", game_id).into())
                .unwrap(),
            _ => {
                panic!("These errors should not appear on a simple player existence check")
            }
        },
    }
}
