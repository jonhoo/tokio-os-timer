//! This crate provides timers for use with tokio that rely on OS mechanisms for timer management
//! rather than a separate timing mechanism like [`tokio-timer`]. This comes at somewhat increased
//! overhead if you have many timers, but allows the timers to have any granularity supported by
//! your operating system where `tokio-timer` can only support timers with a granularity of 1ms.
//! In particular, the system timers usually support whatever granularity the underlying hardware
//! supports (see "High-resolution timers" in [`time(7)`]), which on my laptop is 1ns!
//!
//! The current implementation uses [`timerfd_create(2)`] on Linux, and [`kqueue(2)` timers] on
//! macOS and BSDs.
//!
//!   [`tokio-timer`]: https://docs.rs/tokio-timer/
//!   [`timerfd_create(2)`]: https://linux.die.net/man/2/timerfd_settime
//!   [`kqueue(2)` timers]: https://man.openbsd.org/kqueue.2
//!   [`time(7)`]: https://linux.die.net/man/7/time

#![deny(missing_docs)]

mod delay;
pub use self::delay::Delay;

mod interval;
pub use self::interval::Interval;

mod sys;
