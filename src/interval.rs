use crate::sys::{TimeSpec, Timer};
use futures::{try_ready, Async, Poll, Stream};
use std::io;
use std::time::Duration;

/// A stream that yields once every time a fixed amount of time elapses.
///
/// Instances of `Interval` perform no work.
pub struct Interval {
    e: Option<Timer>,
}

impl Interval {
    /// Create a new `Interval` instance that yields at now + `interval`, and every subsequent
    /// `interval`.
    pub fn new(interval: Duration) -> io::Result<Self> {
        if interval.as_secs() == 0 && interval.subsec_nanos() == 0 {
            // this would be interpreted as "inactive timer" by timerfd_settime
            return Ok(Self { e: None });
        }

        let mut timer = Timer::new()?;

        // arm the timer
        timer.set(TimeSpec::Interval(interval))?;

        Ok(Self { e: Some(timer) })
    }
}

impl Stream for Interval {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(ref mut e) = self.e {
            try_ready!(e.poll());
        }
        Ok(Async::Ready(Some(())))
    }
}
