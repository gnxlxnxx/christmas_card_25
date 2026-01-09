use embassy_futures::select::{Either, select, select3};
use embassy_time::{Duration, Ticker};

use crate::{drivers::{
    buttons::{Button, Buttons, Event},
    ws2812::Ws2812,
}, util::sync::{self}};

pub mod matrix;
pub mod ws2812;

const AUTO_DURATION: Duration = Duration::from_secs(45);

pub async fn run(ws2812: &mut Ws2812) {
    let ws2812_next_event = sync::Event::new();
    let matrix_next_event = sync::Event::new();

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
                        } => break,
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
                            ws2812_next_event.trigger();
                        }
                        Event {
                            button: Button::R,
                            pressed: true,
                        } => {
                            auto = false;
                            matrix_next_event.trigger();
                        }
                        _ => (),
                    },
                    Either::Second(()) => {
                        if auto {
                            ws2812_next_event.trigger();
                            matrix_next_event.trigger();
                        }
                    }
                }
            }
        },
        matrix::run(&matrix_next_event),
        ws2812::run(ws2812, &ws2812_next_event),
    ).await;
}
