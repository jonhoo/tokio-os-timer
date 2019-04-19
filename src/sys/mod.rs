use mio::{unix::EventedFd, Poll, PollOpt, Ready, Token};
use std::io;
use std::os::unix::io::RawFd;

pub(crate) struct Timer(RawFd);

impl Timer {
    pub(crate) fn new() -> io::Result<Self> {
        let tfd = unsafe { libc::timerfd_create(libc::CLOCK_MONOTONIC, libc::TFD_NONBLOCK) };
        if tfd == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(Timer(tfd))
        }
    }

    pub(crate) fn set(&mut self, timer: libc::itimerspec) -> io::Result<()> {
        let ret = unsafe { libc::timerfd_settime(self.0, 0, &timer, std::ptr::null_mut()) };
        if ret == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub(crate) fn check(&mut self) -> io::Result<()> {
        let mut buf = [0; 8];
        let ret = unsafe { libc::read(self.0, buf.as_mut().as_mut_ptr() as *mut _, 8) };
        if ret == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl mio::Evented for Timer {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.0).deregister(poll)
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };
    }
}
