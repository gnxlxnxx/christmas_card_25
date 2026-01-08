use embassy_futures::select::{Either, select};
use embassy_time::Ticker;

use crate::{drivers::{buttons::{Button, Buttons, Event}, ws2812::Ws2812}, util::{itoa::utoa10, text::{self, TEXT_BRIGHTNESS, TEXT_DURATION}}};

pub mod snake;
pub mod tetris;

async fn show_score(has_won: bool, score: u32) {
    let mut clock = Ticker::every(TEXT_DURATION);

    select(
        wait_for_start_or_select(),
        async {
            if has_won {
                text::scroll(b"Herzlichen Gl\xFCckwunsch!", 2 * TEXT_BRIGHTNESS, &mut clock).await;
            } else {
                text::scroll(b"Game Over!", TEXT_BRIGHTNESS, &mut clock).await;
            }
        }
    ).await;

    select(
        wait_for_start_or_select(),
        async {
            let mut itoa = [0u8; 10];
            let score_str = utoa10(score as u32, &mut itoa);

            loop {
                text::scroll(b" Score: ", TEXT_BRIGHTNESS, &mut clock).await;
                text::scroll(score_str, TEXT_BRIGHTNESS, &mut clock).await;
            }
        }
    ).await;
}

async fn wait_for_start_or_select() {
    loop {
        match Buttons::event().await {
            Event {
                button: Button::Start,
                pressed: true,
            } |
            Event {
                button: Button::Select,
                pressed: true,
            } => {
                break
            },
            _ => (),
        }
    }
}

enum Game {
    Tetris,
    Snake,
}

impl Game {
    pub fn new() -> Self {
        Self::Tetris
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Tetris => Self::Snake,
            Self::Snake => Self::Tetris,
        }
    }

    pub fn to_str(&self) -> &'static [u8] {
        match self {
            Self::Tetris => b"Tetris",
            Self::Snake => b"Snake",
        }
    }

    pub async fn run(&self) {
        match self {
            Self::Tetris => tetris::run().await,
            Self::Snake => snake::run().await,
        }
    }
}

pub async fn run(ws2812: &mut Ws2812) {
    let mut game = Game::new();

    loop {
        let mut clock = Ticker::every(TEXT_DURATION);

        match select(
            Buttons::event(),
            async {
                text::clear_scroll(&mut clock).await;
                text::scroll(game.to_str(), TEXT_BRIGHTNESS, &mut clock).await;
            }
        ).await {
            Either::First(ev) => match ev {
                Event {
                    button: Button::Start,
                    pressed: true,
                } => break,
                Event {
                    button: _,
                    pressed: true,
                } => game.next(),
                _ => (),
            }
            Either::Second(()) => (),
        }
    }

    game.run().await;
}
