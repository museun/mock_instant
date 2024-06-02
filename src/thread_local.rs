use std::{cell::RefCell, time::Duration};

std::thread_local! {
    static TIME: RefCell<Duration> = const { RefCell::new(Duration::ZERO) };
    static SYSTEM_TIME: RefCell<Duration> = const { RefCell::new(Duration::ZERO) };
}

fn with_time(d: impl Fn(&mut Duration)) {
    TIME.with(|t| d(&mut t.borrow_mut()));
}

fn get_time() -> Duration {
    TIME.with(|t| *t.borrow())
}

fn with_system_time(d: impl Fn(&mut Duration)) {
    SYSTEM_TIME.with(|t| d(&mut t.borrow_mut()));
}

fn get_system_time() -> Duration {
    SYSTEM_TIME.with(|t| *t.borrow())
}

crate::macros::define_mock_clock! {
    true;
    /// This uses thread-local state for the deterministic clock
}

crate::macros::define_instant! {
    MockClock::time;
    true;
    /// This uses a thread-local cell for its time source
}

crate::macros::define_system_time! {
    MockClock::system_time;
    true;
    /// This uses a global mutex for its time source
}

crate::macros::define_instant_tests!();

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn is_thread_local() {
        assert!(MockClock::is_thread_local());
        assert!(Instant::now().is_thread_local());
        assert!(SystemTime::now().is_thread_local());
    }

    // this checks that threads get their own time source
    #[test]
    fn thread_locality() {
        MockClock::set_time(Duration::ZERO);

        let start = Instant::now();
        let handles = [
            std::thread::spawn(move || {
                let start = Instant::now();
                MockClock::advance(Duration::from_secs(3));
                assert_eq!(start.elapsed(), Duration::from_secs(3));
            }),
            std::thread::spawn(move || {
                let start = Instant::now();
                MockClock::advance(Duration::from_secs(30));
                assert_eq!(start.elapsed(), Duration::from_secs(30));
            }),
        ];

        MockClock::advance(Duration::from_secs(10));
        assert_eq!(start.elapsed(), Duration::from_secs(10));

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
