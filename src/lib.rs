/*! # mock_instant

**_NOTE_** As of version 0.4, the thread-local clock has been removed. The clock will always be thread-safe.

To ensure unsurprising behavior, **reset** the clock _before_ each test (if that behavior is applicable.)

---

This crate allows you to test `Instant`/`Duration`/`SystemTime` code, deterministically.

_This uses a static mutex to have a thread-aware clock._

It provides a replacement `std::time::Instant` that uses a deterministic 'clock'

You can swap out the `std::time::Instant` with this one by doing something similar to:

```rust
#[cfg(test)]
use mock_instant::Instant;

#[cfg(not(test))]
use std::time::Instant;
```

or for a `std::time::SystemTime`
```rust
#[cfg(test)]
use mock_instant::{SystemTime, SystemTimeError};

#[cfg(not(test))]
use std::time::{SystemTime, SystemTimeError};
```


Or a `std::time::SystemTime` with this one by doing something similar to:

```rust
#[cfg(test)]
use mock_instant::SystemTime;

#[cfg(not(test))]
use std::time::SystemTime;
```

## Example

```rust
# use mock_instant::MockClock;
# use mock_instant::Instant;
use std::time::Duration;

let now = Instant::now();
MockClock::advance(Duration::from_secs(15));
MockClock::advance(Duration::from_secs(2));

// its been '17' seconds
assert_eq!(now.elapsed(), Duration::from_secs(17));
```

## API:

```rust,compile_fail
// Overrides the current time to this `Duration`
MockClick::set_time(time: Duration)

// Advance the current time by this `Duration`
MockClick::advance(time: Duration)

// Get the current time
MockClick::time() -> Duration

// Overrides the current `SystemTime` to this duration
MockClick::set_system_time(time: Duration)

// Advance the current `SystemTime` by this duration
MockClick::sdvance_system_time(time: Duration)

// Get the current `SystemTime`
MockClick::system_time() -> Duration
```

## Usage:

**_NOTE_** The clock starts at `Duration::ZERO`

In your tests, you can use `MockClock::set_time(Duration::ZERO)` to reset the clock back to 0. Or, you can set it to some sentinel time value.

Then, before you check your time-based logic, you can advance the clock by some `Duration` (it'll freeze the time to that duration)

You can also get the current frozen time with `MockClock::time`

`SystemTime` is also mockable with a similar API.

*/

use std::{sync::Mutex, time::Duration};

pub(crate) static TIME: Mutex<Duration> = Mutex::new(Duration::ZERO);
pub(crate) static SYSTEM_TIME: Mutex<Duration> = Mutex::new(Duration::ZERO);

pub(crate) fn with_time(d: impl Fn(&mut Duration)) {
    let mut t = TIME.lock().unwrap();
    d(&mut t);
}

pub(crate) fn get_time() -> Duration {
    *TIME.lock().unwrap()
}

pub(crate) fn with_system_time(d: impl Fn(&mut Duration)) {
    let mut t = SYSTEM_TIME.lock().unwrap();
    d(&mut t);
}

pub(crate) fn get_system_time() -> Duration {
    *SYSTEM_TIME.lock().unwrap()
}

/// A Mock clock
///
/// This uses thread local state to have a deterministic clock.
#[derive(Copy, Clone)]
pub struct MockClock;

impl std::fmt::Debug for MockClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockClock")
            .field("time", &Self::time())
            .field("system_time", &Self::system_time())
            .finish()
    }
}

impl MockClock {
    /// Set the internal Instant clock to this 'Duration'
    pub fn set_time(time: Duration) {
        self::with_time(|t| *t = time);
    }

    /// Advance the internal Instant clock by this 'Duration'
    pub fn advance(time: Duration) {
        self::with_time(|t| *t += time);
    }

    /// Get the current Instant duration
    pub fn time() -> Duration {
        self::get_time()
    }

    /// Set the internal SystemTime clock to this 'Duration'
    pub fn set_system_time(time: Duration) {
        self::with_system_time(|t| *t = time);
    }

    /// Advance the internal SystemTime clock by this 'Duration'
    pub fn advance_system_time(time: Duration) {
        self::with_system_time(|t| *t += time);
    }

    /// Get the current SystemTime duration
    pub fn system_time() -> Duration {
        self::get_system_time()
    }
}

/// An error returned from the duration_since and elapsed methods on SystemTime, used to learn how far in the opposite direction a system time lies.
#[derive(Clone, Debug)]
pub struct SystemTimeError(Duration);

impl SystemTimeError {
    pub fn duration(&self) -> Duration {
        self.0
    }
}

impl std::fmt::Display for SystemTimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "second time provided was later than self")
    }
}

impl std::error::Error for SystemTimeError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "other time was not earlier than self"
    }
}

/// A simple deterministic `SystemTime` wrapped around a modifiable `Duration`
///
/// This is used the state for the 'wall clock' that is configurable via
/// the `MockClock`
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct SystemTime(Duration);

/// A mocked UNIX_EPOCH. This starts at `0` rather than the traditional unix epoch date.
pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::ZERO);

impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = UNIX_EPOCH;

    pub fn now() -> Self {
        Self(MockClock::system_time())
    }

    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        self.0
            .checked_sub(earlier.0)
            .ok_or_else(|| SystemTimeError(earlier.0 - self.0))
    }

    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        Self::now().duration_since(*self)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
        duration
            .as_millis()
            .checked_add(self.0.as_millis())
            .map(|c| Duration::from_millis(c as _))
            .map(Self)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
        self.0
            .as_millis()
            .checked_sub(duration.as_millis())
            .map(|c| Duration::from_millis(c as _))
            .map(Self)
    }
}

impl std::ops::Add<Duration> for SystemTime {
    type Output = SystemTime;

    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding duration to instant")
    }
}

impl std::ops::AddAssign<Duration> for SystemTime {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs
    }
}

impl std::ops::Sub<Duration> for SystemTime {
    type Output = SystemTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        self.checked_sub(rhs)
            .expect("overflow when subtracting duration from instant")
    }
}

impl std::ops::SubAssign<Duration> for SystemTime {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs
    }
}

impl From<std::time::SystemTime> for SystemTime {
    fn from(value: std::time::SystemTime) -> Self {
        Self(
            value
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .expect("std::time::SystemTime is before UNIX_EPOCH"),
        )
    }
}

impl From<SystemTime> for std::time::SystemTime {
    fn from(value: SystemTime) -> Self {
        Self::UNIX_EPOCH + value.0
    }
}

/// A simple deterministic Instant wrapped around a modifiable Duration
///
/// This used a thread-local state as the 'wall clock' that is configurable via
/// the `MockClock`
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Instant(Duration);

impl Instant {
    pub fn now() -> Self {
        Self(MockClock::time())
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.0 - earlier.0
    }

    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        self.0.checked_sub(earlier.0)
    }

    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    pub fn elapsed(&self) -> Duration {
        MockClock::time() - self.0
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        duration
            .as_millis()
            .checked_add(self.0.as_millis())
            .map(|c| Duration::from_millis(c as _))
            .map(Self)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.0
            .as_millis()
            .checked_sub(duration.as_millis())
            .map(|c| Duration::from_millis(c as _))
            .map(Self)
    }
}

impl std::ops::Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs
    }
}

impl std::ops::Sub for Instant {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Self::Output {
        self.duration_since(rhs)
    }
}

impl std::ops::Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, rhs: Duration) -> Self::Output {
        self.checked_sub(rhs)
            .expect("overflow when substracting duration from instant")
    }
}

impl std::ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        self.0 -= rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_time() {
        MockClock::set_time(Duration::ZERO)
    }

    fn reset_system_time() {
        MockClock::set_system_time(Duration::ZERO)
    }

    #[test]
    fn set_system_time() {
        reset_system_time();

        MockClock::set_system_time(Duration::from_secs(42));
        assert_eq!(MockClock::system_time(), Duration::from_secs(42));

        reset_system_time();
        assert_eq!(MockClock::system_time(), Duration::ZERO);
    }

    #[test]
    fn advance_system_time() {
        reset_system_time();

        for i in 0..3 {
            MockClock::advance_system_time(Duration::from_millis(100));
            let time = Duration::from_millis(100 * (i + 1));
            assert_eq!(MockClock::system_time(), time);
        }
    }

    #[test]
    fn system_time() {
        reset_system_time();

        let now = SystemTime::now();
        for i in 0..3 {
            MockClock::advance_system_time(Duration::from_millis(100));
            assert_eq!(now.elapsed().unwrap(), Duration::from_millis(100 * (i + 1)));
        }
        MockClock::advance_system_time(Duration::from_millis(100));

        let next = SystemTime::now();
        assert_eq!(
            next.duration_since(now).unwrap(),
            Duration::from_millis(400)
        );
    }

    #[test]
    fn system_time_methods() {
        reset_system_time();

        let system_time = SystemTime::now();
        let interval = Duration::from_millis(42);
        MockClock::advance_system_time(interval);

        // zero + 1 = 1
        assert_eq!(
            system_time.checked_add(Duration::from_millis(1)).unwrap(),
            SystemTime(Duration::from_millis(1))
        );

        // now + 1 = diff + 1
        assert_eq!(
            SystemTime::now()
                .checked_add(Duration::from_millis(1))
                .unwrap(),
            SystemTime(Duration::from_millis(43))
        );

        // zero - 1 = None
        assert!(system_time.checked_sub(Duration::from_millis(1)).is_none());

        // now - 1 = diff -1
        assert_eq!(
            SystemTime::now()
                .checked_sub(Duration::from_millis(1))
                .unwrap(),
            SystemTime(Duration::from_millis(41))
        );

        // now - 1 = diff - 1
        assert_eq!(
            SystemTime::now() - Duration::from_millis(1),
            SystemTime(Duration::from_millis(41))
        );

        // now - diff + 1 = none
        assert!(SystemTime::now()
            .checked_sub(Duration::from_millis(43))
            .is_none());
    }

    #[test]
    fn system_time_from_std_roundtrip() {
        let std_now = std::time::SystemTime::now();
        let mock_now: SystemTime = std_now.into();
        assert!(mock_now.0 > Duration::from_secs(1708041600)); // Friday 16 February 2024 00:00:00 GMT
        let roundtrip_now: std::time::SystemTime = mock_now.into();
        assert_eq!(std_now, roundtrip_now)
    }

    #[test]
    fn set_time() {
        reset_time();

        MockClock::set_time(Duration::from_secs(42));
        assert_eq!(MockClock::time(), Duration::from_secs(42));

        reset_time();
        assert_eq!(MockClock::time(), Duration::ZERO);
    }

    #[test]
    fn advance() {
        reset_time();

        for i in 0..3 {
            MockClock::advance(Duration::from_millis(100));
            let time = Duration::from_millis(100 * (i + 1));
            assert_eq!(MockClock::time(), time);
        }
    }

    #[test]
    fn instant() {
        reset_time();

        let now = Instant::now();
        for i in 0..3 {
            MockClock::advance(Duration::from_millis(100));
            assert_eq!(now.elapsed(), Duration::from_millis(100 * (i + 1)));
        }
        MockClock::advance(Duration::from_millis(100));

        let next = Instant::now();
        assert_eq!(next.duration_since(now), Duration::from_millis(400));
    }

    #[test]
    fn methods() {
        reset_time();

        let instant = Instant::now();
        let interval = Duration::from_millis(42);
        MockClock::advance(interval);

        // 0 - now = None
        assert!(instant.checked_duration_since(Instant::now()).is_none());

        // now - 0 = diff
        assert_eq!(
            Instant::now().checked_duration_since(instant).unwrap(),
            interval
        );

        // 0 since now = none
        assert!(instant.checked_duration_since(Instant::now()).is_none());

        // now since 0 = diff
        assert_eq!(
            Instant::now().checked_duration_since(instant).unwrap(),
            interval
        );

        // zero + 1 = 1
        assert_eq!(
            instant.checked_add(Duration::from_millis(1)).unwrap(),
            Instant(Duration::from_millis(1))
        );

        // now + 1 = diff + 1
        assert_eq!(
            Instant::now()
                .checked_add(Duration::from_millis(1))
                .unwrap(),
            Instant(Duration::from_millis(43))
        );

        // zero - 1 = None
        assert!(instant.checked_sub(Duration::from_millis(1)).is_none());

        // now - 1 = diff -1
        assert_eq!(
            Instant::now()
                .checked_sub(Duration::from_millis(1))
                .unwrap(),
            Instant(Duration::from_millis(41))
        );

        // now - 1 = diff - 1
        assert_eq!(
            Instant::now() - Duration::from_millis(1),
            Instant(Duration::from_millis(41))
        );

        // now - diff + 1 = none
        assert!(Instant::now()
            .checked_sub(Duration::from_millis(43))
            .is_none());
    }
}
