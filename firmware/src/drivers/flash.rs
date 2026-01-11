use ch32_hal::pac::{
    FLASH,
    flash::regs::{Addr, Keyr, Modekeyr},
};

use crate::util::sync::poll_while;

#[unsafe(link_section = ".highscore")]
static mut HIGH_SCORES: [u32; 2] = [0; 2];
const HIGH_SCORES_PTR: *mut [u32; 2] = (&raw mut HIGH_SCORES).wrapping_byte_add(0x08000000);

pub fn new_high_score(id: usize, score: u32) -> u32 {
    let mut scores = unsafe { HIGH_SCORES_PTR.read_volatile() };

    // Restore erased flash to sane value
    if scores[id] == u32::MAX {
        scores[id] = 0;
    }

    if score > scores[id] {
        scores[id] = score;

        // Very long critical section (multiple ms)
        // Far from ideal, but seems to improve reliability
        critical_section::with(|_| {
            // Unlock flash
            while FLASH.statr().read().bsy() {}
            FLASH.keyr().write_value(Keyr(0x45670123));
            FLASH.keyr().write_value(Keyr(0xCDEF89AB));
            FLASH.modekeyr().write_value(Modekeyr(0x45670123));
            FLASH.modekeyr().write_value(Modekeyr(0xCDEF89AB));

            // Erase score page
            FLASH.ctlr().write(|w| w.set_page_er(true));
            FLASH.addr().write_value(Addr(HIGH_SCORES_PTR as u32));
            FLASH.ctlr().write(|w| {
                w.set_page_er(true);
                w.set_strt(true);
            });
            while FLASH.statr().read().bsy() {}

            // Write score page
            FLASH.ctlr().write(|w| {
                w.set_page_pg(true);
                w.set_bufrst(true);
            });
            while FLASH.statr().read().bsy() {}
            for i in 0..16 {
                unsafe {
                    (HIGH_SCORES_PTR as *mut u32)
                        .add(i)
                        .write_volatile(scores.get(i as usize).copied().unwrap_or(0));
                }
                FLASH.ctlr().write(|w| {
                    w.set_page_pg(true);
                    w.set_bufload(true);
                });
                while FLASH.statr().read().bsy() {}
            }
            FLASH.addr().write_value(Addr(HIGH_SCORES_PTR as u32));
            FLASH.ctlr().write(|w| {
                w.set_page_pg(true);
                w.set_strt(true);
            });
            while FLASH.statr().read().bsy() {}

            // Lock flash
            FLASH.ctlr().write(|w| {
                w.set_flock(true);
                w.set_lock(true);
            });
        });
    }

    scores[id]
}
