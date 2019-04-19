// heavily copied from https://github.com/tokio-rs/tokio/blob/master/tokio-timer/tests/delay.rs

use futures::Stream;
use std::time::Duration;
use tokio_os_timer::Interval;

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
    let mut interval = Interval::new(Duration::new(0, 0)).unwrap();
    assert!(mock.enter(|| assert_ready!(interval)).is_some());
    assert!(mock.enter(|| assert_ready!(interval)).is_some());
}

#[test]
fn delayed() {
    let mut mock = tokio_mock_task::MockTask::new();
    let interval = Duration::from_millis(10);
    let mut t = Interval::new(interval).unwrap();
    mock.enter(|| assert_not_ready!(t));
    // sleep until interval has passed
    std::thread::sleep(Duration::from_millis(15));
    assert!(mock.enter(|| assert_ready!(t)).is_some());
    mock.enter(|| assert_not_ready!(t));
    // sleep until interval has passed again
    std::thread::sleep(Duration::from_millis(10));
    assert!(mock.enter(|| assert_ready!(t)).is_some());
    mock.enter(|| assert_not_ready!(t));
}
