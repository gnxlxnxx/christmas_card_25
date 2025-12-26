use embassy_futures::select::select3;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal, watch::Watch};
use embassy_time::Duration;

use crate::drivers::{buttons::{self, Button, Buttons, Event}, ws2812::Ws2812};

pub mod matrix;
pub mod ws2812;

const AUTO_DURATION: Duration = Duration::from_secs(15);

pub async fn run(ws2812: &mut Ws2812<'_>) {
    let ws2812_next_signal: Signal<NoopRawMutex, ()> = Signal::new();
    let matrix_next_signal: Signal<NoopRawMutex, ()> = Signal::new();
    let auto_watch: Watch<NoopRawMutex, bool, 2> = Watch::new_with(true);
    let auto_sender = auto_watch.sender();

    select3(
        async {
            loop {
                match Buttons::event().await {
                    Event { button: Button::Start, pressed: true } => return,
                    Event { button: Button::Select, pressed: true } => auto_sender.send(true),
                    Event { button: Button::L, pressed: true } => {
                        auto_sender.send(false);
                        ws2812_next_signal.signal(());
                    },
                    Event { button: Button::R, pressed: true } => {
                        auto_sender.send(false);
                        matrix_next_signal.signal(());
                    },
                    _ => (),
                }
            }
        },
        matrix::run(&matrix_next_signal, auto_watch.receiver().unwrap()),
        ws2812::run(ws2812, &ws2812_next_signal, auto_watch.receiver().unwrap()),
    ).await;
}
