use ch32_hal::pac::{FLASH, flash::regs::{Addr, Keyr, Modekeyr}};

use crate::util::sync::poll_while;

#[unsafe(link_section = ".highscore")]
static mut HIGH_SCORES: [u32; 2] = [0; 2];
const HIGH_SCORES_PTR: *mut u32 = (&raw mut HIGH_SCORES as *mut u32).wrapping_byte_add(0x08000000);

pub async fn new_high_score(id: usize, score: u32) -> u32 {
    let mut scores = unsafe { (&raw const HIGH_SCORES).read_volatile() };

    // Restore erased flash to sane value
    if scores[id] == u32::MAX {
        scores[id] = 0;
    }

    if score > scores[id] {
        scores[id] = score;

        // Unlock flash
        while FLASH.statr().read().bsy() {}
        FLASH.keyr().write_value(Keyr(0x45670123));
        FLASH.keyr().write_value(Keyr(0xCDEF89AB));
        FLASH.modekeyr().write_value(Modekeyr(0x45670123));
        FLASH.modekeyr().write_value(Modekeyr(0xCDEF89AB));

        FLASH.addr().write_value(Addr(HIGH_SCORES_PTR as u32));

        // Erase score page
        FLASH.ctlr().write(|w| w.set_page_er(true));
        FLASH.ctlr().write(|w| {
            w.set_page_er(true);
            w.set_strt(true);
        });
        poll_while(|| FLASH.statr().read().bsy()).await;

        // Write score page
        FLASH.ctlr().write(|w| {
            w.set_page_pg(true);
            w.set_bufrst(true);
        });
        while FLASH.statr().read().bsy() {}
        for (i, score) in scores.into_iter().enumerate() {
            unsafe {
                HIGH_SCORES_PTR.add(i).write_volatile(score);
            }
            FLASH.ctlr().write(|w| {
                w.set_page_pg(true);
                w.set_bufload(true);
            });
            while FLASH.statr().read().bsy() {}

        }
        FLASH.ctlr().write(|w| {
            w.set_page_pg(true);
            w.set_strt(true);
        });
        poll_while(|| FLASH.statr().read().bsy()).await;

        // Lock flash
        FLASH.ctlr().write(|w| {
            w.set_flock(true);
            w.set_lock(true);
        });
    }

    scores[id]
}
