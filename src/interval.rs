use futures::{try_ready, Async, Poll, Stream};
use std::io;
use std::time::Duration;

/// A stream that yields once every time a fixed amount of time elapses.
///
/// Instances of `Interval` perform no work.
pub struct Interval {
    fd: Box<std::os::unix::io::RawFd>,
    e: Option<tokio_reactor::PollEvented<mio::unix::EventedFd<'static>>>,
}

impl Interval {
    /// Create a new `Interval` instance that yields at now + `interval`, and every subsequent
    /// `interval`.
    pub fn new(interval: Duration) -> io::Result<Self> {
        if interval.as_secs() == 0 && interval.subsec_nanos() == 0 {
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

impl Stream for Interval {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.e.is_none() {
            return Ok(Async::Ready(Some(())));
        }

        let ready = mio::Ready::readable();
        try_ready!(self.e.as_mut().unwrap().poll_read_ready(ready));

        // do a read to reset
        let mut buf = [0; 8];
        let ret = unsafe { libc::read(*self.fd, buf.as_mut().as_mut_ptr() as *mut _, 8) };
        if ret == -1 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::WouldBlock {
                self.e.as_mut().unwrap().clear_read_ready(ready)?;
                return Ok(Async::NotReady);
            }
            return Err(e);
        }
        Ok(Async::Ready(Some(())))
    }
}

impl Drop for Interval {
    fn drop(&mut self) {
        if let Some(e) = self.e.take() {
            drop(e);
            unsafe { libc::close(*self.fd) };
        }
    }
}
