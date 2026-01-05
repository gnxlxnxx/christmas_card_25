use crate::drivers::ws2812::Ws2812;

pub mod snake;
pub mod tetris;

pub async fn run(ws2812: &mut Ws2812) {
    tetris::run().await;
    snake::run().await;
}
