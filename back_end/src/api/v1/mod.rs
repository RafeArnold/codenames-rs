use crate::game::CodeNamesError;
use crate::game_service::GameService;
use std::collections::HashMap;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use self::events_handler::EventsRouter;

pub mod events_handler;
pub mod join_game_handler;
pub mod new_game_handler;
pub mod player_joined_handler;

pub fn routes(game_service: Arc<GameService>) -> BoxedFilter<(impl Reply,)> {
    warp::path!("play" / "v1" / ..)
        .and(
            new_game_handler::route(game_service.clone())
                .or(join_game_handler::route(game_service.clone()))
                .or(player_joined_handler::route(game_service.clone()))
                .or(EventsRouter::new(game_service.clone()).route())
                .recover(handle_missing_query_param_rejection)
                .boxed()
                .with(warp::wrap_fn(player_id_cookie_wrap)),
        )
        .boxed()
}

const GAME_ID_QUERY_PARAM_NAME: &str = "game-id";

pub fn game_id_query_param() -> impl Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::query::<HashMap<String, String>>().and_then(
        |query_params: HashMap<String, String>| async move {
            query_params
                .get(GAME_ID_QUERY_PARAM_NAME)
                .map(|game_id| game_id.to_string())
                .ok_or_else(|| {
                    warp::reject::custom(MissingQueryParameter {
                        parameter_key: GAME_ID_QUERY_PARAM_NAME,
                    })
                })
        },
    )
}

#[derive(Debug)]
struct MissingQueryParameter {
    parameter_key: &'static str,
}

impl warp::reject::Reject for MissingQueryParameter {}

impl warp::reject::Reject for CodeNamesError {}

async fn handle_missing_query_param_rejection(
    err: warp::Rejection,
) -> Result<impl Reply, warp::Rejection> {
    if let Some(err) = err.find::<MissingQueryParameter>() {
        Ok(warp::http::Response::builder()
            .status(warp::http::StatusCode::BAD_REQUEST)
            .body(format!("Missing {} query parameter", err.parameter_key))
            .unwrap())
    } else {
        Err(err)
    }
}

const PLAYER_ID_COOKIE_NAME: &str = "codenames.player-id";

pub fn player_id_cookie() -> impl Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::cookie::<String>(PLAYER_ID_COOKIE_NAME)
}

pub fn player_id_cookie_wrap<R>(filter: BoxedFilter<(R,)>) -> BoxedFilter<(Response,)>
where
    R: Reply + 'static,
{
    warp::any()
        .and(warp::cookie::optional::<String>(PLAYER_ID_COOKIE_NAME))
        .and(filter)
        .map(|player_id: Option<String>, reply: R| match player_id {
            Some(_) => reply.into_response(),
            None => {
                let player_id: String = uuid::Uuid::new_v4().simple().to_string();
                let mut res = reply.into_response();
                res.headers_mut().insert(
                    warp::http::header::SET_COOKIE,
                    warp::http::HeaderValue::from_str(
                        format!(
                            "{}={}; HttpOnly; SameSite=Strict{}",
                            PLAYER_ID_COOKIE_NAME,
                            player_id,
                            if std::env::var("INSECURE")
                                .ok()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(false)
                            {
                                ""
                            } else {
                                "; Secure"
                            }
                        )
                        .as_str(),
                    )
                    .unwrap(),
                );
                res
            }
        })
        .boxed()
}
