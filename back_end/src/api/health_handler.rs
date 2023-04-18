use warp::Filter;

pub fn route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    warp::path!("health")
        .map(|| "ok")
        .boxed()
}
