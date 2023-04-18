use crate::game_service::GameService;
use game_repo::GameRepository;
use std::sync::Arc;
use warp::Filter;

mod api;
mod front_end_handler;
mod game;
mod game_repo;
mod game_service;

#[tokio::main]
async fn main() {
    let repo_url = std::env::var("REPO_URL").expect("No REPO_URL env variable");
    let game_repo: GameRepository = GameRepository::new(repo_url.as_str())
        .expect(format!("Failed to create repository from URL {}", repo_url).as_str());
    let game_service: Arc<GameService> = Arc::new(GameService::new(game_repo));
    let front_end_static_dir =
        std::env::var("FRONT_END_DIR").unwrap_or("./front_end/dist".to_string());
    let routes = api::health_handler::route()
        .or(front_end_handler::route(front_end_static_dir)
            .with(warp::wrap_fn(api::v1::player_id_cookie_wrap))
            .boxed())
        .or(api::v1::routes(game_service.clone()));
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
