use futures::{try_ready, Async, Poll};
use std::io;
use std::time::Duration;
use tokio_reactor::Registration;

#[cfg(any(
    target_os = "bitrig",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod kqueue;

#[cfg(any(
    target_os = "bitrig",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use kqueue::Timer as SysTimer;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "solaris"))]
mod timerfd;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "solaris"))]
use timerfd::Timer as SysTimer;

pub(crate) enum TimeSpec {
    Timeout(Duration),
    Interval(Duration),
}

pub(crate) struct Timer {
    r: Registration,
    t: SysTimer,
}

impl Timer {
    pub(crate) fn new() -> io::Result<Self> {
        let t = Timer {
            r: Registration::new(),
            t: SysTimer::new()?,
        };
        t.r.register(&t.t)?;
        Ok(t)
    }

    pub(crate) fn set(&mut self, timer: TimeSpec) -> io::Result<()> {
        self.t.set(timer)
    }

    pub(crate) fn poll(&mut self) -> Poll<(), io::Error> {
        let r = try_ready!(self.r.poll_read_ready());
        if !r.is_readable() {
            return Ok(Async::NotReady);
        }

        // make sure to also check the timer in case of spurious wakeups
        match self.t.check() {
            Ok(_) => Ok(Async::Ready(())),
            Err(err) => {
                if err.kind() == io::ErrorKind::WouldBlock {
                    return Ok(Async::NotReady);
                }
                Err(err)
            }
        }
    }
}
