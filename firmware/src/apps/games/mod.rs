use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

use crate::{
    drivers::{
        buttons::{Button, Buttons, Event},
        ws2812::Color,
    },
    util::{
        itoa::utoa10,
        rand::WhiteNoiseGenerator,
        text::{self, TEXT_BRIGHTNESS, TEXT_DURATION},
        ws2812::{FilteredWs2812, HUETABLE},
    },
};

const HIGH_SCORE_TEXT: &[u8] = b" High Score: ";

pub mod pong;
pub mod snake;
pub mod tetris;

async fn show_score(has_won: bool, score: u32, high_score: u32) {
    let mut clock = Ticker::every(TEXT_DURATION);
    let scroll = if has_won {
        text::scroll(b"Herzlichen Gl\x82ckwunsch!", 2 * TEXT_BRIGHTNESS, &mut clock)
    } else {
        text::scroll(b"Game Over!", TEXT_BRIGHTNESS, &mut clock)
    };

    select(wait_for_start_or_select(), scroll).await;

    select(
        wait_for_start_or_select(),
        async {
            let mut itoa_buf = [0u8; 10];

            loop {
                text::scroll(&HIGH_SCORE_TEXT[5..], TEXT_BRIGHTNESS, &mut clock).await;
                text::scroll(utoa10(score, &mut itoa_buf), TEXT_BRIGHTNESS, &mut clock).await;
                text::scroll(HIGH_SCORE_TEXT, TEXT_BRIGHTNESS, &mut clock).await;
                text::scroll(utoa10(high_score, &mut itoa_buf), TEXT_BRIGHTNESS, &mut clock).await;
            }
        }
    ).await;
}

async fn wait_for_start_or_select() {
    while !matches!(
        Buttons::event().await,
        Event {
            button: Button::Start | Button::Select,
            pressed: true
        }
    ) {}
}

fn set_ws2812_led_random(led: &mut Color, enabled: bool, noisegen: &mut WhiteNoiseGenerator) {
    if enabled {
        if *led == Color::new(0, 0, 0) {
            let ang = noisegen.rand8() as usize;
            led.set_r(HUETABLE[(ang + 85) % HUETABLE.len()]);
            led.set_g(HUETABLE[ang % HUETABLE.len()]);
            led.set_b(HUETABLE[(ang + 170) % HUETABLE.len()]);
        }
    } else {
        *led = Color::new(0, 0, 0);
    }
}

enum Game {
    Tetris,
    Snake,
    Pong,
}

impl Game {
    pub const fn new() -> Self {
        Self::Tetris
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Tetris => Self::Snake,
            Self::Snake => Self::Pong,
            Self::Pong => Self::Tetris,
        }
    }

    pub fn prev(&mut self) {
        *self = match self {
            Self::Snake => Self::Tetris,
            Self::Pong => Self::Snake,
            Self::Tetris => Self::Pong,
        }
    }

    pub const fn to_str(&self) -> &'static [u8] {
        match self {
            Self::Tetris => b" Tetris",
            Self::Snake => b" Snake",
            Self::Pong => b" Pong",
        }
    }

    pub const fn high_score_index(&self) -> usize {
        match self {
            Self::Tetris => 0,
            Self::Snake => 1,
            _ => panic!("No high score for this game"),
        }
    }

    pub async fn run(&self) {
        match self {
            Self::Tetris => tetris::run().await,
            Self::Snake => snake::run().await,
            Self::Pong => pong::run().await,
        }
    }
}

pub async fn run(filt_ws2812: &mut FilteredWs2812) {
    let leds = filt_ws2812.target_mut();
    leds[1] = Color::new(0, 0, 0);
    leds[4] = Color::new(0, 0, 0);

    select(
        async {
            let mut game = Game::new();

            loop {
                let mut clock = Ticker::every(TEXT_DURATION);

                if let Either::First(Event { pressed: true, button }) = select(
                    Buttons::event(),
                    text::scroll(game.to_str(), TEXT_BRIGHTNESS, &mut clock),
                ).await {
                    match button {
                        Button::Start => break,
                        Button::Select => game.next(),
                        Button::L => return,
                        Button::R => game.prev(),
                    }
                }
            }

            game.run().await;
        },
        async {
            let mut clock = Ticker::every(Duration::from_millis(10));
            let mut noisegen = WhiteNoiseGenerator::new();

            loop {
                let leds = filt_ws2812.target_mut();

                set_ws2812_led_random(&mut leds[0], Buttons::get(Button::L), &mut noisegen);
                set_ws2812_led_random(&mut leds[2], Buttons::get(Button::Start), &mut noisegen);
                set_ws2812_led_random(&mut leds[3], Buttons::get(Button::Select), &mut noisegen);
                set_ws2812_led_random(&mut leds[5], Buttons::get(Button::R), &mut noisegen);

                filt_ws2812.update().await;
                clock.next().await;
            }
        },
    ).await;
}
