use core::{sync::atomic::{AtomicU8, Ordering}, task::Poll};

use embassy_time::{Duration, Instant, Ticker};

use crate::{drivers::{buttons::{Button, Buttons, Event}, matrix::{Framebuffer, Matrix}}};

const P1_BRIGHTNESS: u8 = 100;
const P2_BRIGHTNESS: u8 = P1_BRIGHTNESS / 3;
const LIMIT_MAX_BRIGHTNESS: u8 = 100;
const WIN_BRIGHTNESS: u8 = 150;

const WIDTH: u8 = 7;
const HEIGHT: u8 = 6;
const SELECTED_DELTA: u8 = 2;
const SPAWN_X: u8 = WIDTH / 2;
const BASE_Y: usize = Framebuffer::HEIGHT - HEIGHT as usize - 1;
const SELECTED_Y: usize = BASE_Y - SELECTED_DELTA as usize;

// from math import sqrt
// STEPS = 8
// MAX = 255 / (sqrt(2) - 1) # = sqrt(2 * DELTA_S / A)
// print('    ' + ' '.join((f'{round(MAX * (sqrt(i+1) - sqrt(i)))},' for i in range(STEPS))))
const DROP_DURATIONS: [u8; (HEIGHT + SELECTED_DELTA - 1) as usize] = [
    255, 196, 165, 145, 131, 121, 112,
];

const _: () = {
    assert!((WIDTH + 1) as usize <= Framebuffer::WIDTH);
    assert!((HEIGHT + 1 + SELECTED_DELTA) as usize <= Framebuffer::HEIGHT);
};

struct LimitAnimation {
    ticker: Ticker,
    inc: bool,
}

impl LimitAnimation {
    pub fn new() -> Self {
        Self {
            ticker: Ticker::every(Duration::from_millis(20)),
            inc: false,
        }
    }

    pub fn poll(&mut self) {
        if self.ticker.consume_expired() {
            let mut brightness = Matrix::fb().0[BASE_Y][WIDTH as usize].load(Ordering::Relaxed);

            if brightness == 0 {
                self.inc = true;
            } else if brightness >= LIMIT_MAX_BRIGHTNESS {
                self.inc = false;
            }

            if self.inc {
                brightness += 1;
            } else {
                brightness -= 1;
            }

            for row in Matrix::fb().0[BASE_Y..].iter().take(HEIGHT as usize) {
                row[WIDTH as usize].store(brightness, Ordering::Relaxed);
            }

            for field in &Matrix::fb().0[BASE_Y + HEIGHT as usize][..(WIDTH as usize + 1)] {
                field.store(brightness, Ordering::Relaxed);
            }
        }
    }
}

struct WinAnimation {
    ticker: Ticker,
    x: u8,
    y: u8,
    dx: i8,
    dy: i8,
    original_brightness: u8,
}

impl WinAnimation {
    pub fn new(x: u8, y: u8, dx: i8, dy: i8, original_brightness: u8) -> Self {
        Self {
            ticker: Ticker::every(Duration::from_millis(500)),
            x,
            y,
            dx,
            dy,
            original_brightness,
        }
    }
    pub fn poll(&mut self) {
        if self.ticker.consume_expired() {
            let brightness = if Matrix::fb().load(self.x as usize, self.y as usize) == self.original_brightness {
                WIN_BRIGHTNESS
            } else {
                self.original_brightness
            };

            for i in 0..4 {
                Matrix::fb().store(
                    (self.x as usize).wrapping_sub_signed(i * self.dx as isize) as usize,
                    (self.y as usize).wrapping_sub_signed(i * self.dy as isize) as usize,
                    brightness
                );
            }
        }
    }
}

enum GameResult {
    Won(WinAnimation),
    Remis,
}

struct DropAnimation {
    next_instant: Instant,
    delta_y: u8,
}

impl DropAnimation {
    pub fn new() -> Self {
        Self {
            next_instant: Instant::now() + Duration::from_ticks(616),
            delta_y: 0,
        }
    }

    pub fn poll(&mut self, game: &Game) -> bool {
        if self.next_instant <= Instant::now() {
            self.matrix_el(game.selected).store(0, Ordering::Relaxed);
            self.delta_y += 1;
            self.matrix_el(game.selected)
                .store(game.current_player_brightness(), Ordering::Relaxed);

            if self.delta_y >= game.top_y[game.selected as usize] + SELECTED_DELTA {
                return true;
            }

            self.next_instant += Duration::from_ticks(DROP_DURATIONS[self.delta_y as usize] as u32);
        }

        false
    }

    #[inline(always)]
    fn matrix_el(&self, x: u8) -> &'static AtomicU8 {
        &Matrix::fb().0[SELECTED_Y + self.delta_y as usize][x as usize]
    }
}

struct Game {
    top_y: [u8; WIDTH as usize],
    selected: u8,
    p1_turn: bool,
}

impl Game {
    pub fn new() -> Self {
        let game = Self {
            top_y: [HEIGHT; WIDTH as usize],
            selected: WIDTH / 2,
            p1_turn: true,
        };

        Matrix::fb().clear_all();
        game.draw_selected();

        game
    }

    pub fn drop(&mut self) -> DropAnimation {
        self.top_y[self.selected as usize] -= 1;

        DropAnimation::new()
    }

    pub fn move_right(&mut self) {
        self.move_right_test();
    }

    pub fn move_left(&mut self) {
        self.clear_selected();

        loop {
            if self.selected <= 0 {
                self.selected = WIDTH - 1;
            } else {
                self.selected -= 1;
            }

            if self.top_y[self.selected as usize] > 0 {
                break;
            }
        }

        self.draw_selected();
    }

    fn move_right_test(&mut self)  -> bool {
        let selected_start = self.selected;

        self.clear_selected();

        loop {
            if self.selected >= WIDTH - 1 {
                self.selected = 0;
            } else {
                self.selected += 1;
            }


            if self.top_y[self.selected as usize] > 0 {
                break;
            } else if self.selected == selected_start {
                return true;
            }
        }

        self.draw_selected();

        false
    }

    pub fn next_player(&mut self) -> Option<GameResult> {
        if let Some(a) = self.check_win() {
            Some(GameResult::Won(a))
        } else {
            self.p1_turn ^= true;
            self.selected = SPAWN_X - 1;

            if self.move_right_test() {
                Some(GameResult::Remis)
            } else {
                None
            }
        }
    }

    fn check_win(&self) -> Option<WinAnimation> {
        let x = self.selected as usize;
        let y = self.top_y[x as usize] as usize;
        let p_brightness = self.current_player_brightness();

        for (dx, dy) in [(1i8, 0i8), (0, 1), (1, 1), (-1, 1)] {
            let dx = dx as isize;
            let dy = dy as isize;

            let back = match dx {
                -1 => WIDTH as usize - 1 - x,
                0 => usize::MAX,
                1 => x,
                _ => unreachable!(),
            };
            let back = back.min(match dy {
                0 => usize::MAX,
                1 => y,
                _ => unreachable!(),
            }) as isize;
            let start_x = x.wrapping_sub_signed(dx * back);
            let start_y = y.wrapping_sub_signed(dy * back);
            let len = match dx {
                -1 => start_x + 1,
                0 => usize::MAX,
                1 => WIDTH as usize - start_x,
                _ => unreachable!(),
            };
            let len = len.min(match dy {
                0 => usize::MAX,
                1 => HEIGHT as usize - start_y,
                _ => unreachable!(),
            }) as isize;

            let start_y = start_y + BASE_Y;

            let mut consecutive = 0;

            for i in 0..len {
                let x = start_x.wrapping_add_signed(dx * i);
                let y = start_y.wrapping_add_signed(dy * i);

                if Matrix::fb().0[y][x].load(Ordering::Relaxed) == p_brightness {
                    consecutive += 1;
                    if consecutive >= 4 {
                        return Some(WinAnimation::new(x as u8, y as u8, dx as i8, dy as i8, p_brightness));
                    }
                } else {
                    consecutive = 0;
                }
            }
        }

        None
    }

    #[inline(always)]
    fn clear_selected(&self) {
        Matrix::fb().0[SELECTED_Y][self.selected as usize].store(0, Ordering::Relaxed);
    }

    fn draw_selected(&self) {
        Matrix::fb().0[SELECTED_Y][self.selected as usize].store(
            self.current_player_brightness(),
            Ordering::Relaxed,
        );
    }

    fn current_player_brightness(&self) -> u8 {
        if self.p1_turn {
            P1_BRIGHTNESS
        } else {
            P2_BRIGHTNESS
        }
    }
}

enum TaskState {
    Game(Game, Option<DropAnimation>),
    Score(GameResult),
}

pub struct Task {
    state: TaskState,
    limit_animation: LimitAnimation,
}

impl Task {
    pub fn new() -> Self {
        Self {
            state: TaskState::Game(Game::new(), None),
            limit_animation: LimitAnimation::new(),
        }
    }

    pub fn poll(&mut self) -> bool {
        self.limit_animation.poll();

        match &mut self.state {
            TaskState::Game(game, drop_animation) => {
                if let Some(animation) = drop_animation {
                    Buttons::discard_events();

                    if animation.poll(game) {
                        *drop_animation = None;
                        if let Some(res) = game.next_player() {
                            self.state = TaskState::Score(res);
                        }
                    }
                } else {
                    if let Poll::Ready(Event { pressed: true, button }) = Buttons::event() {
                        match button {
                            Button::Start | Button::Select => *drop_animation = Some(game.drop()),
                            Button::L => game.move_left(),
                            Button::R => game.move_right(),
                        }
                    }
                }
            }
            TaskState::Score(game_result) => {
                if matches!(
                    Buttons::event(),
                    Poll::Ready(Event {
                        button: Button::Start | Button::Select,
                        pressed: true,
                    })
                ) {
                    return true;
                }

                if let GameResult::Won(win_animation) = game_result {
                    win_animation.poll();
                }
            }
        }

        false
    }
}
