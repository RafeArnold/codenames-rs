use warp::Filter;

pub fn route(static_dir_path: String) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    warp::fs::dir(static_dir_path).boxed()
}
