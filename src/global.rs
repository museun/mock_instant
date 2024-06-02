use std::{sync::Mutex, time::Duration};

static TIME: Mutex<Duration> = Mutex::new(Duration::ZERO);
static SYSTEM_TIME: Mutex<Duration> = Mutex::new(Duration::ZERO);

fn with_time(d: impl Fn(&mut Duration)) {
    let mut t = TIME.lock().unwrap();
    d(&mut t);
}

fn get_time() -> Duration {
    *TIME.lock().unwrap()
}

fn with_system_time(d: impl Fn(&mut Duration)) {
    let mut t = SYSTEM_TIME.lock().unwrap();
    d(&mut t);
}

fn get_system_time() -> Duration {
    *SYSTEM_TIME.lock().unwrap()
}

crate::macros::define_mock_clock! {
    false;
    /// This uses a global mutex state for the deterministic clock
}

crate::macros::define_instant! {
    MockClock::time;
    false;
    /// This uses a global mutex for its time source
}

crate::macros::define_system_time! {
    MockClock::system_time;
    false;
    /// This uses a global mutex for its time source
}

crate::macros::define_instant_tests!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_thread_local() {
        assert!(!MockClock::is_thread_local());
        assert!(!Instant::now().is_thread_local());
        assert!(!SystemTime::now().is_thread_local());
    }

    #[test]
    fn thread_sharing() {
        let start = Instant::now();

        std::thread::spawn(move || {
            let start = Instant::now();
            MockClock::advance(Duration::from_secs(3));
            assert_eq!(start.elapsed(), Duration::from_secs(3));
        })
        .join()
        .unwrap();

        std::thread::spawn(move || {
            let start = Instant::now();
            MockClock::advance(Duration::from_secs(30));
            assert_eq!(start.elapsed(), Duration::from_secs(30));
        })
        .join()
        .unwrap();

        MockClock::advance(Duration::from_secs(10));
        assert_eq!(start.elapsed(), Duration::from_secs(43));
    }
}
