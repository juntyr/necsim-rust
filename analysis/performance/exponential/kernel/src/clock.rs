/// A predefined, read-only 64-bit unsigned cycle counter.
#[inline]
#[must_use]
pub fn counter() -> u64 {
    let counter: u64;
    unsafe { core::arch::asm!("mov.u64  {}, %clock64;", out(reg64) counter, options(nostack)) };
    counter
}

/// A predefined, 64-bit global nanosecond timer.
#[inline]
#[must_use]
pub fn timer_ns() -> u64 {
    let timer: u64;
    unsafe { core::arch::asm!("mov.u64  {}, %globaltimer;", out(reg64) timer, options(nostack)) };
    timer
}
