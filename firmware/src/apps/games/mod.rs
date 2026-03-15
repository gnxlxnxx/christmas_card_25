use core::task::Poll;

use embassy_time::{Duration, Ticker};

use crate::{
    drivers::{
        buttons::{Button, Buttons, Event},
        flash::ScoreFlasher,
        ws2812::Color,
    },
    util::{
        itoa::utoa10,
        rand::Rng,
        text::{TEXT_BRIGHTNESS, TEXT_DURATION, TextScroller},
        ws2812::{FilteredWs2812, HUETABLE},
    },
};

const HIGH_SCORE_TEXT: &[u8] = b" High Score: ";

pub mod four_in_a_row;
pub mod pong;
pub mod snake;
pub mod tetris;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ShowScoreTaskState {
    Greeter,
    ScoreText,
    Score,
    HighScoreText,
    HighScore,
}

struct ShowScoreTask {
    ticker: Option<Ticker>,
    scroller: TextScroller,
    flasher: ScoreFlasher,
    state: ShowScoreTaskState,
    has_won: bool,
    score: u32,
    high_score: u32,
}

impl ShowScoreTask {
    pub fn new(game: GameSelection, has_won: bool, score: u32) -> Self {
        let (high_score, flasher) = ScoreFlasher::new(game.high_score_index(), score);

        Self {
            ticker: Some(Ticker::every(TEXT_DURATION)),
            scroller: TextScroller::new(),
            flasher,
            state: ShowScoreTaskState::Greeter,
            has_won,
            score,
            high_score,
        }
    }

    pub fn poll(&mut self) -> bool {
        if matches!(
            Buttons::event(),
            Poll::Ready(Event {
                button: Button::Start | Button::Select,
                pressed: true
            })
        ) {
            if self.state == ShowScoreTaskState::Greeter {
                self.next_state();
            } else {
                self.ticker = None;
            }
        }

        if self.ticker.as_mut().is_some_and(|t| t.consume_expired()) {
            let mut itoa_buf = [0u8; 10];

            if self.scroller.advance_brightness(self.text(&mut itoa_buf), self.brightness()) {
                self.next_state();
            }
        }

        self.flasher.poll() && self.ticker.is_none()
    }

    fn next_state(&mut self) {
        self.scroller = TextScroller::new();
        self.state = match self.state {
            ShowScoreTaskState::Greeter => ShowScoreTaskState::ScoreText,
            ShowScoreTaskState::ScoreText => ShowScoreTaskState::Score,
            ShowScoreTaskState::Score => ShowScoreTaskState::HighScoreText,
            ShowScoreTaskState::HighScoreText => ShowScoreTaskState::HighScore,
            ShowScoreTaskState::HighScore => ShowScoreTaskState::ScoreText,
        };
    }

    fn text<'a>(&self, itoa_buf: &'a mut [u8; 10]) -> &'a [u8] {
        match self.state {
            ShowScoreTaskState::Greeter => {
                if self.has_won {
                    b"Herzlichen Gl\x82ckwunsch!".as_slice()
                } else {
                    b"Game Over!".as_slice()
                }
            }
            ShowScoreTaskState::ScoreText => &HIGH_SCORE_TEXT[5..],
            ShowScoreTaskState::Score => utoa10(self.score, itoa_buf),
            ShowScoreTaskState::HighScoreText => HIGH_SCORE_TEXT,
            ShowScoreTaskState::HighScore => utoa10(self.high_score, itoa_buf),
        }
    }

    fn brightness(&self) -> u8 {
        if self.state == ShowScoreTaskState::Greeter && self.has_won {
            2 * TEXT_BRIGHTNESS
        } else {
            TEXT_BRIGHTNESS
        }
    }
}

fn set_ws2812_led_random(led: &mut Color, enabled: bool, noisegen: &mut Rng) {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum GameSelection {
    Tetris,
    Snake,
    Pong,
    FourInARow,
}

impl GameSelection {
    pub const fn new() -> Self {
        Self::Tetris
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Tetris => Self::Snake,
            Self::Snake => Self::Pong,
            Self::Pong => Self::FourInARow,
            Self::FourInARow => Self::Tetris,
        }
    }

    pub fn prev(&mut self) {
        *self = match self {
            Self::Snake => Self::Tetris,
            Self::Pong => Self::Snake,
            Self::FourInARow => Self::Pong,
            Self::Tetris => Self::FourInARow,
        }
    }

    pub const fn to_str(self) -> &'static [u8] {
        match self {
            Self::Tetris => b" Tetris",
            Self::Snake => b" Snake",
            Self::Pong => b" Pong",
            Self::FourInARow => b" Four In A Row",
        }
    }

    pub const fn high_score_index(self) -> usize {
        match self {
            Self::Tetris => 0,
            Self::Snake => 1,
            _ => panic!("No high score for this game"),
        }
    }
}

enum Game {
    Tetris(tetris::Task),
    Snake(snake::Task),
    Pong(pong::Task),
    FourInARow(four_in_a_row::Task),
}

impl Game {
    pub fn from_selection(selection: GameSelection) -> Self {
        match selection {
            GameSelection::Tetris => Self::Tetris(tetris::Task::new()),
            GameSelection::Snake => Self::Snake(snake::Task::new()),
            GameSelection::Pong => Self::Pong(pong::Task::new()),
            GameSelection::FourInARow => Self::FourInARow(four_in_a_row::Task::new()),
        }
    }

    pub fn poll(&mut self) -> bool {
        match self {
            Self::Tetris(task) => task.poll(),
            Self::Snake(task) => task.poll(),
            Self::Pong(task) => task.poll(),
            Self::FourInARow(task) => task.poll(),
        }
    }
}

enum GameTask {
    Selecting(GameSelection, Ticker, TextScroller),
    Started(Game),
}

impl GameTask {
    pub fn new() -> Self {
        Self::Selecting(
            GameSelection::new(),
            Ticker::every(TEXT_DURATION),
            TextScroller::new(),
        )
    }

    pub fn poll(&mut self) -> bool {
        match self {
            Self::Selecting(game, ticker, scroller) => {
                if ticker.consume_expired() && scroller.advance(game.to_str()) {
                    *scroller = TextScroller::new();
                }

                if let Poll::Ready(Event {
                    pressed: true,
                    button,
                }) = Buttons::event()
                {
                    *scroller = TextScroller::new();
                    match button {
                        Button::Start => *self = Self::Started(Game::from_selection(*game)),
                        Button::Select => game.next(),
                        Button::L => return true,
                        Button::R => game.prev(),
                    }
                }

                false
            }
            Self::Started(game) => game.poll(),
        }
    }
}

pub struct Task {
    game: GameTask,
    ws2812_ticker: Ticker,
    rng: Rng,
}

impl Task {
    pub fn new(ws2812: &mut FilteredWs2812) -> Self {
        let leds = ws2812.target_mut();
        leds[1] = Color::new(0, 0, 0);
        leds[4] = Color::new(0, 0, 0);

        Self {
            game: GameTask::new(),
            ws2812_ticker: Ticker::every(Duration::from_millis(10)),
            rng: Rng::new(),
        }
    }

    pub fn poll(&mut self, ws2812: &mut FilteredWs2812) -> bool {
        if self.ws2812_ticker.expired() {
            let leds = ws2812.target_mut();

            set_ws2812_led_random(&mut leds[0], Buttons::get(Button::L), &mut self.rng);
            set_ws2812_led_random(&mut leds[2], Buttons::get(Button::Start), &mut self.rng);
            set_ws2812_led_random(&mut leds[3], Buttons::get(Button::Select), &mut self.rng);
            set_ws2812_led_random(&mut leds[5], Buttons::get(Button::R), &mut self.rng);

            if ws2812.try_update() {
                self.ws2812_ticker.consume_next();
            }
        }

        self.game.poll()
    }
}
