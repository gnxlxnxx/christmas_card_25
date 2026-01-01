use embassy_time::{Duration, Ticker};

use crate::util::text::{self, TEXT_BRIGHTNESS, TEXT_DURATION};

// const MESSAGE: &[u8] = b"Die Fachschaft Elektro- und Informationstechnik an der \x80 Universit\xE4t Stuttgart w\xFCnscht Euch allen recht herzlich ein frohes Weihnachtsfest!";
const MESSAGE: &[u8] = b"Xmas";

pub async fn run() -> ! {
    let mut clock = Ticker::every(TEXT_DURATION);

    loop {
        text::clear_scroll(&mut clock).await;
        text::scroll(MESSAGE, TEXT_BRIGHTNESS, &mut clock).await;
        text::clear_scroll(&mut clock).await;
    }
}
