use embassy_futures::select::{Either, select, select3};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};

use crate::drivers::{
    buttons::{Button, Buttons, Event},
    ws2812::Ws2812,
};

pub mod matrix;
pub mod ws2812;

const AUTO_DURATION: Duration = Duration::from_secs(45);

pub async fn run(ws2812: &mut Ws2812<'_>) {
    let ws2812_next_signal: Signal<NoopRawMutex, ()> = Signal::new();
    let matrix_next_signal: Signal<NoopRawMutex, ()> = Signal::new();

    select3(
        async {
            let mut clock = Ticker::every(AUTO_DURATION);
            let mut auto = true;

            loop {
                match select(Buttons::event(), clock.next()).await {
                    Either::First(ev) => match ev {
                        Event {
                            button: Button::Start,
                            pressed: true,
                        } => return,
                        Event {
                            button: Button::Select,
                            pressed: true,
                        } => {
                            clock.reset();
                            auto = true;
                        }
                        Event {
                            button: Button::L,
                            pressed: true,
                        } => {
                            auto = false;
                            ws2812_next_signal.signal(());
                        }
                        Event {
                            button: Button::R,
                            pressed: true,
                        } => {
                            auto = false;
                            matrix_next_signal.signal(());
                        }
                        _ => (),
                    },
                    Either::Second(()) => {
                        if auto {
                            ws2812_next_signal.signal(());
                            matrix_next_signal.signal(());
                        }
                    }
                }
            }
        },
        matrix::run(&matrix_next_signal),
        ws2812::run(ws2812, &ws2812_next_signal),
    ).await;
}
