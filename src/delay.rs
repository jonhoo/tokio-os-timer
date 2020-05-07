use crate::sys::{TimeSpec, Timer};
use futures::{try_ready, Async, Future, Poll};
use std::io;
use std::time::Duration;

/// A future that completes a specified amount of time from its creation.
///
/// Instances of `Delay` perform no work and complete with `()` once the specified duration has been passed.
pub struct Delay {
    e: Option<Timer>,
}

impl Delay {
    /// Create a new `Delay` instance that elapses at now + `delay`.
    #[deprecated(since = "0.1.8", note = "Please use the async-timer crate")]
    pub fn new(delay: Duration) -> io::Result<Self> {
        if delay.as_secs() == 0 && delay.subsec_nanos() == 0 {
            // this would be interpreted as "inactive timer" by timerfd_settime
            return Ok(Self { e: None });
        }

        let mut timer = Timer::new()?;

        // arm the timer
        timer.set(TimeSpec::Timeout(delay))?;

        Ok(Self { e: Some(timer) })
    }
}

impl Future for Delay {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(ref mut e) = self.e {
            try_ready!(e.poll());
        }
        Ok(Async::Ready(()))
    }
}
