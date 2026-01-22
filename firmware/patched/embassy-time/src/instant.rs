use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use core::cmp::{Ord, Ordering};

use super::Duration;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// An Instant in time, based on the MCU's clock ticks since startup.
pub struct Instant {
    ticks: i32,
}

impl Instant {
    /// The smallest (earliest) value that can be represented by the `Instant` type.
    pub const MIN: Instant = Instant { ticks: i32::MIN };
    /// The largest (latest) value that can be represented by the `Instant` type.
    pub const MAX: Instant = Instant { ticks: i32::MAX };

    /// Returns an Instant representing the current time.
    #[inline]
    pub fn now() -> Instant {
        Instant {
            ticks: embassy_time_driver::now(),
        }
    }

    /// Create an Instant from a tick count since system boot.
    pub const fn from_ticks(ticks: i32) -> Self {
        Self { ticks }
    }

    /// Tick count since system boot.
    pub const fn as_ticks(&self) -> i32 {
        self.ticks
    }

    /// Duration between this Instant and another Instant
    /// Panics on over/underflow.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        unwrap!(self.checked_duration_since(earlier))
    }

    /// Duration between this Instant and another Instant
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        Some(Duration {
            ticks: self.ticks.wrapping_sub(earlier.ticks).try_into().ok()?,
        })
    }

    /// Returns the duration since the "earlier" Instant.
    /// If the "earlier" instant is in the future, the duration is set to zero.
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        Duration {
            ticks: self.ticks.wrapping_sub(earlier.ticks).max(0) as u32,
        }
    }

    /// Duration elapsed since this Instant.
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        Instant { ticks: self.ticks.wrapping_add_unsigned(other.ticks) }
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        Instant { ticks: self.ticks.wrapping_sub_unsigned(other.ticks) }
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

// This is only transitive if the difference doesn't become too big.
// So don't hold onto Instants for too long!
impl Ord for Instant {
    fn cmp(&self, other: &Instant) -> Ordering {
        self.ticks.wrapping_sub(other.ticks).cmp(&0)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Instant {
    fn eq(&self, other: &Self) -> bool {
        self.ticks == other.ticks
    }
}

impl Eq for Instant {}

impl<'a> fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ticks", self.ticks)
    }
}
