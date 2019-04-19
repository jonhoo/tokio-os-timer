//! This crate provides timers for use with tokio that rely on OS mechanisms for timer management
//! rather than a separate timing mechanism like [`tokio-timer`]. This comes at somewhat increased
//! overhead if you have many timers, but allows the timers to have any granularity supported by
//! your operating system where `tokio-timer` can only support timers with a granularity of 1ms.
//!
//! The current implementation relies on [`timerfd_create(2)`], and will thus only work on
//! platforms whose `libc` contains that call.
//!
//!   [`tokio-timer`]: https://docs.rs/tokio-timer/
//!   [`timerfd_create(2)`]: https://linux.die.net/man/2/timerfd_settime

#![deny(missing_docs)]

use futures::{try_ready, Async, Future, Poll};
use std::io;
use std::time::Duration;

/// A future that completes a specified amount of time from its creation.
///
/// Instances of `Delay` perform no work and complete with `()` once the specified duration has been passed.
pub struct Delay {
    fd: Box<std::os::unix::io::RawFd>,
    e: Option<tokio_reactor::PollEvented<mio::unix::EventedFd<'static>>>,
}

impl Delay {
    /// Create a new `Delay` instance that elapses at now + `delay`.
    pub fn new(delay: Duration) -> io::Result<Self> {
        if delay.as_secs() == 0 && delay.subsec_nanos() == 0 {
            // this would be interpreted as "inactive timer" by timerfd_settime
            return Ok(Self {
                fd: Box::new(0),
                e: None,
            });
        }

        let tfd = unsafe { libc::timerfd_create(libc::CLOCK_MONOTONIC, libc::TFD_NONBLOCK) };
        if tfd == -1 {
            return Err(io::Error::last_os_error());
        }
        let tfd = Box::new(tfd);
        let mtfd = mio::unix::EventedFd(&*tfd);
        let e = tokio_reactor::PollEvented::new(mtfd);
        let e: tokio_reactor::PollEvented<mio::unix::EventedFd<'static>> =
            unsafe { std::mem::transmute(e) };

        // arm the timer
        let timer = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: delay.as_secs() as i64,
                tv_nsec: delay.subsec_nanos() as i64,
            },
        };
        let ret = unsafe { libc::timerfd_settime(*tfd, 0, &timer, std::ptr::null_mut()) };
        if ret == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            fd: tfd,
            e: Some(e),
        })
    }
}

impl Future for Delay {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.e.is_none() {
            return Ok(Async::Ready(()));
        }

        let ready = mio::Ready::readable();
        let _ = try_ready!(self.e.as_mut().unwrap().poll_read_ready(ready));
        // we don't ever _actually_ need to read from a timerfd
        self.e.as_mut().unwrap().clear_read_ready(ready)?;
        Ok(Async::Ready(()))
    }
}

impl Drop for Delay {
    fn drop(&mut self) {
        if let Some(e) = self.e.take() {
            drop(e);
            unsafe { libc::close(*self.fd) };
        }
    }
}
