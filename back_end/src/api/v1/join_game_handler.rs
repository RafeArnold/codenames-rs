use crate::api::v1::player_id_cookie;
use crate::game_service::GameService;
use common::api::v1::models::{JoinGameRequest, Player, Group};
use std::sync::Arc;
use warp::Filter;

pub fn route(game_service: Arc<GameService>) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    Filter::boxed(
        warp::post()
            .and(warp::path!("join-game"))
            .and(player_id_cookie())
            .and(warp::body::json::<JoinGameRequest>())
            .and_then(move |player_id, request: JoinGameRequest| {
                handle_request(game_service.clone(), player_id, request)
            }),
    )
}

async fn handle_request(
    game_service: Arc<GameService>,
    player_id: String,
    request: JoinGameRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    game_service
        .add_player(
            request.game_id.as_str(),
            player_id.as_str(),
            Player {
                name: request.player_name,
                group: Group::Spectators,
                is_host: false,
            },
        )
        .await
        .map_err(|err| warp::reject::custom(err))?;
    Ok(warp::http::StatusCode::OK)
}
