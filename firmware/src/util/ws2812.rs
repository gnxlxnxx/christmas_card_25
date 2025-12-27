use crate::drivers::ws2812::{self, Color, Ws2812};

pub struct FilteredWs2812<'a, 'b> {
    ws2812: &'a mut Ws2812<'b>,
    output: [Color; ws2812::LEDS],
    target: [Color; ws2812::LEDS],
}

impl<'a, 'b> FilteredWs2812<'a, 'b> {
    pub fn new(ws2812: &'a mut Ws2812<'b>) -> Self {
        Self {
            ws2812,
            output: [Color::new(0, 0, 0); ws2812::LEDS],
            target: [Color::new(0, 0, 0); ws2812::LEDS],
        }
    }

    pub fn target_mut(&mut self) -> &'_ mut [Color; ws2812::LEDS] {
        &mut self.target
    }

    pub fn update(&mut self) -> impl Future {
        for (current, desired) in self.output.iter_mut().zip(self.target.iter()) {
            current.transition(desired);
        }

        self.ws2812.write(&self.output)
    }
}
