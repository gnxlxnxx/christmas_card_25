use embassy_futures::select::select;

use crate::{
    drivers::ws2812::{self, Color},
    util::{sync::Event, ws2812::FilteredWs2812},
};

pub mod fire;
pub mod huewheel;
pub mod snowball;

pub enum Mode {
    Fire,
    Snowball,
    Huewheel,
}

impl Mode {
    pub fn new() -> Self {
        Self::Fire
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Fire => Self::Snowball,
            Self::Snowball => Self::Huewheel,
            Self::Huewheel => Self::Fire,
        };
    }

    async fn animate(&mut self, filt_ws2812: &mut FilteredWs2812) -> [Color; ws2812::LEDS] {
        match self {
            Self::Fire => fire::run(filt_ws2812).await,
            Self::Snowball => snowball::run(filt_ws2812).await,
            Self::Huewheel => huewheel::run(filt_ws2812).await,
        }
    }
}

pub async fn run(filt_ws2812: &mut FilteredWs2812, next_event: &Event) -> ! {
    let mut mode = Mode::new();

    loop {
        if select(mode.animate(filt_ws2812), next_event.wait())
            .await
            .is_second()
        {
            mode.next();
        }
    }
}
