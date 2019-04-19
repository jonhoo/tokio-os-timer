// heavily copied from https://github.com/tokio-rs/tokio/blob/master/tokio-timer/tests/delay.rs

use futures::Future;
use std::time::{Duration, Instant};
use tokio_os_timer::Delay;

macro_rules! assert_ready {
    ($f:expr) => {{
        use ::futures::Async::*;

        match $f.poll().unwrap() {
            Ready(v) => v,
            NotReady => panic!("NotReady"),
        }
    }};
    ($f:expr, $($msg:expr),+) => {{
        use ::futures::Async::*;

        match $f.poll().unwrap() {
            Ready(v) => v,
            NotReady => {
                let msg = format!($($msg),+);
                panic!("NotReady; {}", msg)
            }
        }
    }}
}

macro_rules! assert_not_ready {
    ($f:expr) => {{
        let res = $f.poll().unwrap();
        assert!(!res.is_ready(), "actual={:?}", res)
    }};
    ($f:expr, $($msg:expr),+) => {{
        let res = $f.poll().unwrap();
        if res.is_ready() {
            let msg = format!($($msg),+);
            panic!("actual={:?}; {}", res, msg);
        }
    }};
}
#[test]
fn immediate() {
    let mut mock = tokio_mock_task::MockTask::new();
    let mut delay = Delay::new(Duration::new(0, 0)).unwrap();
    mock.enter(|| assert_ready!(delay));
}

#[test]
fn delayed() {
    let mut mock = tokio_mock_task::MockTask::new();
    let delay = Duration::from_millis(100);
    let start = Instant::now();
    let mut t = Delay::new(delay).unwrap();
    mock.enter(|| assert_not_ready!(t));
    loop {
        if start.elapsed() >= delay {
            mock.enter(|| assert_ready!(t));
            break;
        } else {
            mock.enter(|| assert_not_ready!(t));
        }
        // make sure delay and this are co-prime
        std::thread::sleep(Duration::from_millis(7));
    }
}
