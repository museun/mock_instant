/*! # mock_instant

This crate allows you to test Instant/Duration code, deterministically ***per thread***.

If cross-thread determinism is required, enable the `sync` feature:
```toml
mock_instant = { version = "0.2", features = ["sync"] }
```

It provides a replacement `std::time::Instant` and `std::time::SystemTime` that uses a deterministic thread-local 'clock'

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

# Example
```rust
# use mock_instant::{MockClock, Instant};
use std::time::Duration;

let now = Instant::now();
MockClock::advance(Duration::from_secs(15));
MockClock::advance(Duration::from_secs(2));

// its been '17' seconds
assert_eq!(now.elapsed(), Duration::from_secs(17));
```

# Mocking a SystemTime
```rust
# use mock_instant::{MockClock, SystemTime};
use std::time::Duration;

let now = SystemTime::now();
MockClock::advance_system_time(Duration::from_secs(15));
MockClock::advance_system_time(Duration::from_secs(2));

// its been '17' seconds
assert_eq!(now.elapsed().unwrap(), Duration::from_secs(17));
```
*/

use std::time::Duration;

#[cfg(feature = "sync")]
mod reference {
    use once_cell::sync::OnceCell;
    use std::{sync::Mutex, time::Duration};

    pub static TIME: OnceCell<Mutex<Duration>> = OnceCell::new();
    pub static SYSTEM_TIME: OnceCell<Mutex<Duration>> = OnceCell::new();

    pub fn with_time(d: impl Fn(&mut Duration)) {
        let t = TIME.get_or_init(Mutex::default);
        let mut t = t.lock().unwrap();
        d(&mut t);
    }

    pub fn get_time() -> Duration {
        *TIME.get_or_init(Mutex::default).lock().unwrap()
    }

    pub fn with_system_time(d: impl Fn(&mut Duration)) {
        let t = SYSTEM_TIME.get_or_init(Mutex::default);
        let mut t = t.lock().unwrap();
        d(&mut t);
    }

    pub fn get_system_time() -> Duration {
        *SYSTEM_TIME.get_or_init(Mutex::default).lock().unwrap()
    }
}

#[cfg(not(feature = "sync"))]
mod reference {
    use std::cell::RefCell;
    use std::time::Duration;

    thread_local! {
        pub static TIME: RefCell<Duration> = RefCell::new(Duration::default());
        pub static SYSTEM_TIME: RefCell<Duration> = RefCell::new(Duration::default());
    }

    pub fn with_time(d: impl Fn(&mut Duration)) {
        TIME.with(|t| d(&mut t.borrow_mut()))
    }

    pub fn with_system_time(d: impl Fn(&mut Duration)) {
        SYSTEM_TIME.with(|t| d(&mut t.borrow_mut()))
    }

    pub fn get_time() -> Duration {
        TIME.with(|t| *t.borrow())
    }

    pub fn get_system_time() -> Duration {
        SYSTEM_TIME.with(|t| *t.borrow())
    }
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
        reference::with_time(|t| *t = time);
    }

    /// Advance the internal Instant clock by this 'Duration'
    pub fn advance(time: Duration) {
        reference::with_time(|t| *t += time);
    }

    /// Get the current Instant duration
    pub fn time() -> Duration {
        reference::get_time()
    }

    /// Set the internal SystemTime clock to this 'Duration'
    pub fn set_system_time(time: Duration) {
        reference::with_system_time(|t| *t = time);
    }

    /// Advance the internal SystemTime clock by this 'Duration'
    pub fn advance_system_time(time: Duration) {
        reference::with_system_time(|t| *t += time);
    }

    /// Get the current SystemTime duration
    pub fn system_time() -> Duration {
        reference::get_system_time()
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

/// A simple deterministic SystemTime wrapped around a modifiable Duration
///
/// This used a thread-local state as the 'wall clock' that is configurable via
/// the `MockClock`
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct SystemTime(Duration);

pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::from_secs(0));

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
        MockClock::set_time(Duration::default())
    }

    fn reset_system_time() {
        MockClock::set_system_time(Duration::default())
    }

    #[test]
    fn set_system_time() {
        MockClock::set_system_time(Duration::from_secs(42));
        assert_eq!(MockClock::system_time(), Duration::from_secs(42));

        reset_system_time();
        assert_eq!(MockClock::system_time(), Duration::default());
    }

    #[test]
    fn advance_system_time() {
        for i in 0..3 {
            MockClock::advance_system_time(Duration::from_millis(100));
            let time = Duration::from_millis(100 * (i + 1));
            assert_eq!(MockClock::system_time(), time);
        }
    }

    #[test]
    fn system_time() {
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
    fn set_time() {
        MockClock::set_time(Duration::from_secs(42));
        assert_eq!(MockClock::time(), Duration::from_secs(42));

        reset_time();
        assert_eq!(MockClock::time(), Duration::default());
    }

    #[test]
    fn advance() {
        for i in 0..3 {
            MockClock::advance(Duration::from_millis(100));
            let time = Duration::from_millis(100 * (i + 1));
            assert_eq!(MockClock::time(), time);
        }
    }

    #[test]
    fn instant() {
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
