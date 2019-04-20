use crate::sys::Timer;
use futures::{try_ready, Async, Poll, Stream};
use std::io;
use std::time::Duration;

/// A stream that yields once every time a fixed amount of time elapses.
///
/// Instances of `Interval` perform no work.
pub struct Interval {
    e: Option<tokio_reactor::PollEvented<Timer>>,
}

impl Interval {
    /// Create a new `Interval` instance that yields at now + `interval`, and every subsequent
    /// `interval`.
    pub fn new(interval: Duration) -> io::Result<Self> {
        if interval.as_secs() == 0 && interval.subsec_nanos() == 0 {
            // this would be interpreted as "inactive timer" by timerfd_settime
            return Ok(Self { e: None });
        }

        let mut timer = tokio_reactor::PollEvented::new(Timer::new()?);

        // arm the timer
        timer.get_mut().set(libc::itimerspec {
            // first expiry
            it_value: libc::timespec {
                tv_sec: interval.as_secs() as i64,
                tv_nsec: i64::from(interval.subsec_nanos()),
            },
            // subsequent expiry intervals
            it_interval: libc::timespec {
                tv_sec: interval.as_secs() as i64,
                tv_nsec: i64::from(interval.subsec_nanos()),
            },
        })?;

        Ok(Self { e: Some(timer) })
    }
}

impl Stream for Interval {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(ref mut e) = self.e {
            let ready = mio::Ready::readable();
            try_ready!(e.poll_read_ready(ready));

            // do a read to reset
            match e.get_mut().check() {
                Ok(_) => Ok(Async::Ready(Some(()))),
                Err(err) => {
                    if err.kind() == io::ErrorKind::WouldBlock {
                        e.clear_read_ready(ready)?;
                        return Ok(Async::NotReady);
                    }
                    Err(err)
                }
            }
        } else {
            Ok(Async::Ready(Some(())))
        }
    }
}
