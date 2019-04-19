//! This crate provides timers for use with tokio that rely on OS mechanisms for timer management
//! rather than a separate timing mechanism like [`tokio-timer`]. This comes at somewhat increased
//! overhead if you have many timers, but allows the timers to have any granularity supported by
//! your operating system where `tokio-timer` can only support timers with a granularity of 1ms.
//!
//! The current implementation relies on [`timerfd_create(2)`], and will thus only work on
//! platforms whose `libc` contains that call (probably just Linux at the moment).
//!
//!   [`tokio-timer`]: https://docs.rs/tokio-timer/
//!   [`timerfd_create(2)`]: https://linux.die.net/man/2/timerfd_settime

#![deny(missing_docs)]

mod delay;
pub use delay::Delay;

mod interval;
pub use interval::Interval;
