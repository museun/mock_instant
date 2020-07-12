# mock_instant

## mock_instant

This crate allows you to test Instant/Duration code, deterministically per thread.const

It provides a replacement `std::time::Instant` that uses a deterministic thread-local 'clock'

You can swap out the `std::time::Instant` with this one by doing something similar to:
```rust
#[cfg(test)]
use mock_instant::Instant;

#[cfg(not(test))]
use std::time::Instant;
```

## Example
```rust
use std::time::Duration;

let now = Instant::now();
MockClock::advance(Duration::from_secs(15));
MockClock::advance(Duration::from_secs(2));

// its been '17' seconds
assert_eq!(now.elapsed(), Duration::from_secs(17));
```

License: 0BSD
