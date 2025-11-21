mod core;
mod state;
mod rendering;
mod input;
mod app;

#[macroquad::main("tiny-neo-space")]
async fn main() {
    app::run().await;
}
