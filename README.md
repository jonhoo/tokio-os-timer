[![Crates.io](https://img.shields.io/crates/v/tokio-os-timer.svg)](https://crates.io/crates/tokio-os-timer)
[![Documentation](https://docs.rs/tokio-os-timer/badge.svg)](https://docs.rs/tokio-os-timer/)
[![Build Status](https://travis-ci.com/jonhoo/tokio-os-timer.svg?branch=master)](https://travis-ci.com/jonhoo/tokio-os-timer)

This crate provides timers for use with tokio that rely on OS mechanisms
for timer management rather than a separate timing mechanism like
[`tokio-timer`]. This comes at somewhat increased overhead if you have
many timers, but allows the timers to have any granularity supported by
your operating system where `tokio-timer` can only support timers with a
granularity of 1ms. In particular, the system timers usually support
whatever granularity the underlying hardware supports (see
"High-resolution timers" in [`time(7)`]), which on my laptop is 1ns!

## Platform support

The current implementation relies on [`timerfd_create(2)`], and will
thus only work on platforms whose `libc` contains that call (probably
just Linux at the moment). A partial macOS/BSD implementation based on
[kqueue] exists in
[#6](https://github.com/jonhoo/tokio-os-timer/pull/6), but needs
debugging from someone using one of those platforms. Windows support is
sadly unlikely to appear
([#9](https://github.com/jonhoo/tokio-os-timer/issues/9)).

  [`tokio-timer`]: https://docs.rs/tokio-timer/
  [`timerfd_create(2)`]: https://linux.die.net/man/2/timerfd_settime
  [kqueue]: https://man.openbsd.org/kqueue.2
  [`time(7)`]: https://linux.die.net/man/7/time
