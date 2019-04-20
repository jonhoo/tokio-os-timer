use std::time::Duration;

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
pub(crate) use kqueue::Timer;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "solaris"))]
mod timerfd;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "solaris"))]
pub(crate) use timerfd::Timer;

pub(crate) enum TimeSpec {
    Timeout(Duration),
    Interval(Duration),
}
