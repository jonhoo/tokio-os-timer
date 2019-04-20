use super::TimeSpec;
use mio::{unix::EventedFd, Poll, PollOpt, Ready, Token};
use nix::sys::event::*;
use std::io;
use std::os::unix::io::RawFd;

pub(crate) struct Timer(RawFd);

impl Timer {
    pub(crate) fn new() -> io::Result<Self> {
        let kq = kqueue().map_err(|e| e.as_errno().unwrap())?;
        Ok(Timer(kq))
    }

    pub(crate) fn set(&mut self, timer: TimeSpec) -> io::Result<()> {
        let mut flags = EventFlag::EV_ADD | EventFlag::EV_ENABLE;
        if let TimeSpec::Timeout(..) = timer {
            flags |= EventFlag::EV_ONESHOT;
        }

        let time = match timer {
            TimeSpec::Timeout(d) | TimeSpec::Interval(d) => d,
        };

        // We need to decide what time unit we want...
        // We want the smallest unit that we can use without overflow, so:
        let mut unit = FilterFlag::NOTE_NSECONDS;
        let mut time = time.as_nanos();
        if time > isize::max_value() as u128 {
            unit = FilterFlag::NOTE_USECONDS;
            time /= 1_000;
        }
        if time > isize::max_value() as u128 {
            unit = FilterFlag::empty(); // default is milliseconds
            time /= 1_000;
        }
        if time > isize::max_value() as u128 {
            unit = FilterFlag::NOTE_SECONDS;
            time /= 1_000;
        }
        let time = time as isize;

        kevent(
            self.0,
            &[KEvent::new(
                1,
                EventFilter::EVFILT_TIMER,
                flags,
                unit,
                time,
                0,
            )],
            &mut [],
            0,
        )
        .map_err(|e| e.as_errno().unwrap())?;

        Ok(())
    }

    pub(crate) fn check(&mut self) -> io::Result<()> {
        let mut ev = [KEvent::new(
            0,
            EventFilter::EVFILT_TIMER,
            EventFlag::empty(),
            FilterFlag::empty(),
            0,
            0,
        )];
        match kevent(self.0, &[], &mut ev[..], 0).map_err(|e| e.as_errno().unwrap())? {
            1 => {
                // timer fired!
                assert_eq!(ev[0].ident(), 1);
                Ok(())
            }
            0 => {
                // timer has not fired?
                Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "no timer kevents",
                ))
            }
            n => unreachable!("somehow got {} events when waiting for at most 1", n),
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
        let _ = nix::unistd::close(self.0);
    }
}
