# mock_instant

**_NOTE_** As of version 0.3, the thread-local clock has been removed. The clock will always be thread-safe.

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

```
#[cfg(test)]
use mock_instant::{SystemTime, SystemTimeError};

#[cfg(not(test))]
use std::time::{SystemTime, SystemTimeError};
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

```rust
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

---

License: 0BSD
