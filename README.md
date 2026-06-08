# mock_instant

## Modules
**_NOTE_** As of verison 0.6, the timer resolution is nanosecond-based, which may be much more accurate than the real thing

**_NOTE_** As of version 0.5. MockClock/Instant/SystemTime have been moved to specific modules

**_NOTE_** The modules, `global` and `thread_local` change the behavior across threads. If `global` is used, the clock keeps its state across threads, otherwise if `thread_local` is used, a new _source_ is made for each thread. In most cases, especially when unit testing, `thread_local` will deliver the most expected behavior.

To ensure unsurprising behavior, **reset** the clock _before_ each test (if that behavior is applicable.)

## Basics

This crate allows you to test `Instant`/`Duration`/`SystemTime` code, deterministically.

It provides a replacement `std::time::Instant` that uses a deterministic 'clock'

You can swap out the `std::time::Instant` with this one by doing something similar to:

```rust
#[cfg(test)]
use mock_instant::thread_local::Instant;

#[cfg(not(test))]
use std::time::Instant;
```

or for a `std::time::SystemTime`

```rust
#[cfg(test)]
use mock_instant::thread_local::{SystemTime, SystemTimeError};

#[cfg(not(test))]
use std::time::{SystemTime, SystemTimeError};
```

```rust
use mock_instant::thread_local::{MockClock, Instant};
use std::time::Duration;

let now = Instant::now();
MockClock::advance(Duration::from_secs(15));
MockClock::advance(Duration::from_secs(2));

// its been '17' seconds
assert_eq!(now.elapsed(), Duration::from_secs(17));
```

## API

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
MockClock::advance_system_time(time: Duration)

// Get the current `SystemTime`
MockClock::system_time() -> Duration

// Determine if this MockClock was thread-local: (useful for assertions to ensure the right mode is being used)
MockClock::is_thread_local() -> bool
Instant::now().is_thread_local() -> bool
SystemTime::now().is_thread_local() -> bool
```

## Usage

**_NOTE_** The clock starts at `Duration::ZERO`

In your tests, you can use `MockClock::set_time(Duration::ZERO)` to reset the clock back to 0. Or, you can set it to some sentinel time value.

Then, before you check your time-based logic, you can advance the clock by some `Duration` (it'll freeze the time to that duration)

You can also get the current frozen time with `MockClock::time`

`SystemTime` is also mockable with a similar API.

## Thread-safety

Two modes are provided via modules. The APIs are identical but the `MockClock` source has different behavior in different threads.

### `mock_instant::thread_local`

- `MockClock` will have an independent state per thread
- `Instant`will have an independent state per thread
- `SystemTime` will have an independent state per thread

Using `thread_local` creates a shared clock in each thread of operation within the test application. It is recommended for unit tests and integration tests that test single-threaded behavior.

`thread_local` will be the module used in most cases and will create the least surprise.

### `mock_instant::global`

- `MockClock` will have shared state per thread
- `Instant`will have shared state per thread
- `SystemTime` will have shared state per thread

Using `global` creates a shared clock across all thread of operation within the test application. It is recommended for integration tests that are specifically testing multi-threaded behavior. Using `global` when not testing multi-threaded behavior is likely to cause inconsistent and confusing errors.

Rust's unit test test-runner executes unit tests in multiple threads (unless directed to run single-threaded). If multiple unit tests are running simultaneouly within the test execution, changes to the `global` clock (resetting or advancing the clock) in one test can created unexpected results in other tests that are executing simultaneously, by changing the clock when the tests assume the clock is not being changed.

## License

License: 0BSD
