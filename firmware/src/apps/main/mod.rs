use embassy_futures::select::{Either, select, select3};
use embassy_time::{Duration, Ticker};

use crate::{
    drivers::buttons::{Button, Buttons, Event},
    util::{
        sync::{self},
        ws2812::FilteredWs2812,
    },
};

pub mod matrix;
pub mod ws2812;

const AUTO_DURATION: Duration = Duration::from_secs(45);

pub async fn run(filt_ws2812: &mut FilteredWs2812) {
    let ws2812_next_event = sync::Event::new();
    let matrix_next_event = sync::Event::new();

    select3(
        async {
            let mut clock = Ticker::every(AUTO_DURATION);
            let mut auto = true;

            loop {
                match select(Buttons::event(), clock.next()).await {
                    Either::First(Event { pressed: true, button }) => match button {
                        Button::Start => break,
                        Button::Select => {
                            clock.reset();
                            auto = true;
                        }
                        Button::L => {
                            ws2812_next_event.trigger();
                            auto = false;
                        }
                        Button::R => {
                            matrix_next_event.trigger();
                            auto = false;
                        }
                    }
                    Either::First(Event { pressed: false, button: _ }) => (),
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
        ws2812::run(filt_ws2812, &ws2812_next_event),
    ).await;
}
