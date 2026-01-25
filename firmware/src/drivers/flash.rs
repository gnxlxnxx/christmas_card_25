use ch32_metapac::{
    FLASH,
    flash::regs::{Addr, Keyr, Modekeyr},
};

type HighScoreStore = [u32; 2];

const KEY1: u32 = 0x45670123;
const KEY2: u32 = 0xCDEF89AB;

#[unsafe(link_section = ".highscore")]
static mut HIGH_SCORES: HighScoreStore = [0; 2];
const HIGH_SCORES_PTR: *mut HighScoreStore = (&raw mut HIGH_SCORES).wrapping_byte_add(0x08000000);

pub enum ScoreFlasher {
    Erasing(HighScoreStore),
    Programming,
    Done,
}

impl ScoreFlasher {
    pub fn new(id:usize, score: u32) -> (u32, Self) {
        let mut scores = unsafe { HIGH_SCORES_PTR.read_volatile() };

        // Restore erased flash to sane value
        if scores[id] == u32::MAX {
            scores[id] = 0;
        }

        let high_score = scores[id];
        let flasher = if score > high_score {
            scores[id] = score;

            while FLASH.statr().read().bsy() {}

            // Unlock flash
            critical_section::with(|_| {
                FLASH.keyr().write_value(Keyr(KEY1));
                FLASH.keyr().write_value(Keyr(KEY2));
                FLASH.modekeyr().write_value(Modekeyr(KEY1));
                FLASH.modekeyr().write_value(Modekeyr(KEY2));
            });

            // Erase score page
            FLASH.ctlr().write(|w| w.set_page_er(true));
            FLASH.addr().write_value(Addr(HIGH_SCORES_PTR as u32));
            FLASH.ctlr().write(|w| {
                w.set_page_er(true);
                w.set_strt(true);
            });

            Self::Erasing(scores)
        } else {
            Self::Done
        };

        (high_score, flasher)
    }

    pub fn poll(&mut self) -> bool {
        match self {
            Self::Erasing(scores) => {
                if !FLASH.statr().read().bsy() {
                    // Write score page
                    FLASH.ctlr().write(|w| w.set_page_pg(true));
                    FLASH.ctlr().write(|w| {
                        w.set_page_pg(true);
                        w.set_bufrst(true);
                    });
                    while FLASH.statr().read().bsy() {}
                    for i in 0..16 {
                        unsafe {
                            (HIGH_SCORES_PTR as *mut u32)
                                .add(i)
                                .write_volatile(scores.get(i).copied().unwrap_or(0));
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

                    *self = Self::Programming;
                }
            }
            Self::Programming => {
                if !FLASH.statr().read().bsy() {
                    // Lock flash
                    FLASH.ctlr().write(|w| {
                        w.set_flock(true);
                        w.set_lock(true);
                    });

                    *self = Self::Done;
                }
            }
            Self::Done => (),
        }

        matches!(self, Self::Done)
    }
}
