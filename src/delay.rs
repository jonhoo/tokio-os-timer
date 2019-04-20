use crate::sys::{TimeSpec, Timer};
use futures::{try_ready, Async, Future, Poll};
use std::io;
use std::time::Duration;

/// A future that completes a specified amount of time from its creation.
///
/// Instances of `Delay` perform no work and complete with `()` once the specified duration has been passed.
pub struct Delay {
    e: Option<tokio_reactor::PollEvented<Timer>>,
}

impl Delay {
    /// Create a new `Delay` instance that elapses at now + `delay`.
    pub fn new(delay: Duration) -> io::Result<Self> {
        if delay.as_secs() == 0 && delay.subsec_nanos() == 0 {
            // this would be interpreted as "inactive timer" by timerfd_settime
            return Ok(Self { e: None });
        }

        let mut timer = tokio_reactor::PollEvented::new(Timer::new()?);

        // arm the timer
        timer.get_mut().set(TimeSpec::Timeout(delay))?;

        Ok(Self { e: Some(timer) })
    }
}

impl Future for Delay {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(ref mut e) = self.e {
            let ready = mio::Ready::readable();
            let _ = try_ready!(e.poll_read_ready(ready));
            // we don't ever _actually_ need to check the timerfd
            e.clear_read_ready(ready)?;
        }
        Ok(Async::Ready(()))
    }
}
