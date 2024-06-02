/*! # mock_instant
**_NOTE_** As of version 0.5. MockClock/Instant/SystemTime have been moved to specific modules
**_NOTE_** The modules, `global` and `thread_local` change the behavior across threads. If `global` is used, the clock keeps its state across threads, otherwise if `thread_local` is used, a new *source* is made for each thread

To ensure unsurprising behavior, **reset** the clock _before_ each test (if that behavior is applicable.)

---

This crate allows you to test `Instant`/`Duration`/`SystemTime` code, deterministically.

It provides a replacement `std::time::Instant` that uses a deterministic 'clock'

You can swap out the `std::time::Instant` with this one by doing something similar to:

```rust
#[cfg(test)]
use mock_instant::global::Instant;

#[cfg(not(test))]
use std::time::Instant;
```

or for a `std::time::SystemTime`
```rust
#[cfg(test)]
use mock_instant::global::{SystemTime, SystemTimeError};

#[cfg(not(test))]
use std::time::{SystemTime, SystemTimeError};
```

```rust
use mock_instant::global::{MockClock, Instant};
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
MockClock::set_time(time: Duration)

// Advance the current time by this `Duration`
MockClock::advance(time: Duration)

// Get the current time
MockClock::time() -> Duration

// Overrides the current `SystemTime` to this duration
MockClock::set_system_time(time: Duration)

// Advance the current `SystemTime` by this duration
MockClock::sdvance_system_time(time: Duration)

// Get the current `SystemTime`
MockClock::system_time() -> Duration

// Determine if this MockClock was thread-local: (useful for assertions to ensure the right mode is being used)
MockClock::is_thread_local() -> bool
Instant::now().is_thread_local() -> bool
SystemTime::now().is_thread_local() -> bool
```

## Usage:

**_NOTE_** The clock starts at `Duration::ZERO`

In your tests, you can use `MockClock::set_time(Duration::ZERO)` to reset the clock back to 0. Or, you can set it to some sentinel time value.

Then, before you check your time-based logic, you can advance the clock by some `Duration` (it'll freeze the time to that duration)

You can also get the current frozen time with `MockClock::time`

`SystemTime` is also mockable with a similar API.


## Thread-safety:

Two modes are provided via modules. The APIs are identical but the `MockClock` source has different behavior in different threads.

- global

when using mock_instant::global `MockClock` `Instant` and `SystemTime` will share its state across threads

- thread_local

when using mock_instant::thread_local `MockClock` `Instant` and `SystemTime` will have a new state per thread

*/

use std::time::Duration;

mod macros;

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

/// Thread-local state.
///
/// This creates a new state when accessed from a new thread
pub mod thread_local;

/// Global state.
///
/// This shares its 'clock' across threads
pub mod global;
