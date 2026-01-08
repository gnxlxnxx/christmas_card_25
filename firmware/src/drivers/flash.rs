use core::{iter, mem::MaybeUninit};

use ch32_hal::pac::{FLASH, FLASH_BASE, flash::regs::{Addr, Keyr, Modekeyr}};

use crate::util::sync::poll_while;

#[unsafe(link_section = ".highscore")]
static mut HIGH_SCORES: [u32; 2] = [0; 2];
const HIGH_SCORES_PTR: *mut u32 = (&raw mut HIGH_SCORES as *mut u32).wrapping_byte_add(0x0800_0000);

pub async fn new_high_score(id: usize, score: u32) -> u32 {
    let mut scores = unsafe { (&raw const HIGH_SCORES).read_volatile() };

    if scores[id] == u32::MAX {
        scores[id] = 0;
    }

    if score > scores[id] {
        scores[id] = score;

        while FLASH.statr().read().bsy() {}
        FLASH.keyr().write_value(Keyr(0x45670123));
        FLASH.keyr().write_value(Keyr(0xCDEF89AB));
        FLASH.modekeyr().write_value(Modekeyr(0x45670123));
        FLASH.modekeyr().write_value(Modekeyr(0xCDEF89AB));

        FLASH.ctlr().write(|w| w.set_page_er(true));
        FLASH.addr().write_value(Addr(HIGH_SCORES_PTR as u32));
        FLASH.ctlr().write(|w| {
            w.set_page_er(true);
            w.set_strt(true);
        });
        poll_while(|| FLASH.statr().read().bsy()).await;

        FLASH.ctlr().write(|w| {
            w.set_page_pg(true);
            w.set_bufrst(true);
        });
        FLASH.addr().write_value(Addr(HIGH_SCORES_PTR as u32));
        while FLASH.statr().read().bsy() {}
        for (i, score) in (0..16).zip(scores.into_iter().chain(iter::repeat(0))) {
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

        FLASH.ctlr().write(|w| {
            w.set_flock(true);
            w.set_lock(true);
        });
    }

    scores[id]
}
