use crate::api::v1::player_id_cookie;
use crate::game_service::GameService;
use common::api::v1::models::{NewGameRequest, NewGameResponse, Player, Group};
use std::sync::Arc;
use warp::reply::json;
use warp::Filter;

pub fn route(game_service: Arc<GameService>) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    Filter::boxed(
        warp::post()
            .and(warp::path!("new-game"))
            .and(player_id_cookie())
            .and(warp::body::json::<NewGameRequest>())
            .and_then(move |player_id, request| {
                handle_request(game_service.clone(), player_id, request)
            }),
    )
}

async fn handle_request(
    game_service: Arc<GameService>,
    player_id: String,
    request: NewGameRequest,
) -> Result<warp::reply::Json, warp::Rejection> {
    let game_id: String = game_service.new_game(player_id.clone()).await;
    game_service
        .add_player(
            game_id.as_str(),
            player_id.as_str(),
            Player {
                name: request.player_name,
                group: Group::Spectators,
                is_host: true,
            },
        )
        .await
        .map_err(|err| warp::reject::custom(err))?;
    Ok(json(&NewGameResponse { game_id }))
}
