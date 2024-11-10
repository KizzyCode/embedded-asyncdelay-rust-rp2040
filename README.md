[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)


# `embedded-asyncdelay-rp2040`
A hardware-based, asynchronous delay that can be used with async/await runtimes. The implementation uses one of the
underlying hardware alarms (`Alarm0` to `Alarm3`) to trigger an interrupt, which in turn wakes the future's associated
waker.

## Configure Hardware Alarm
To use this crate, you must specify which hardware alarm to use for the underlying interrupt scheduler. This is done by
setting the appropriate `alarm` feature. By default, `alarm0` is selected.

## Waker Slots and Initialization
Since we need to wake the async wakers from the interrupt callback, we need some stack-allocated shared storage. By
default, this crate automatically allocates enough space for 16 slots by default. To override this, you must disable the
`init16` feature, and call the `setup_waker_slots!` macro with your desired number of slots manually.

__IMPORTANT:__ If you disable the `init16` feature, but don't call `setup_waker_slots!`, this will yield a rather
cryptic linker error and compilation will fail.
