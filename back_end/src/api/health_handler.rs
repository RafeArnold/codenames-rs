use warp::Filter;

pub fn route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    warp::path!("health")
        .map(|| warp::http::StatusCode::OK)
        .boxed()
}
