use crate::drivers::ws2812::Ws2812;

pub mod snake;

pub async fn run(ws2812: &mut Ws2812<'_>) {
    snake::run().await;
}
