use crate::app::App;

mod api;
mod app;
mod game;
mod simple_input;
mod menu;

fn main() {
    yew::Renderer::<App>::new().render();
}
