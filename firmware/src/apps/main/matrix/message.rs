use embassy_time::{Duration, Ticker};

use crate::util::text;

const MESSAGE: &[u8] = b"Die Fachschaft Elektro- und Informationstechnik an der \x80 Universit\xE4t Stuttgart w\xFCnscht Euch allen recht herzlich ein frohes Weihnachtsfest!";

pub async fn run() -> ! {
    let mut clock = Ticker::every(Duration::from_millis(50));

    loop {
        text::clear_scroll(&mut clock).await;
        text::scroll(MESSAGE, 64, &mut clock).await;
        text::clear_scroll(&mut clock).await;
    }
}
