use core::{arch::x86_64::__cpuid, time::Duration};

use uefi::boot;

pub fn read_tsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}
pub fn tsc_freq() -> Option<u64> {
    if __cpuid(0).eax < 0x15 {
        return None;
    }
    let cpuid_result = core::arch::x86_64::__cpuid(0x15);
    if cpuid_result.eax == 0 || cpuid_result.ebx == 0 {
        return None;
    }
    let freq: u64 = (cpuid_result.ecx as u64 * cpuid_result.ebx as u64) / cpuid_result.eax as u64;
    if freq == 0 {
        return None;
    }

    Some(freq)
}

pub fn calibrate() -> u64 {
    let tsc1 = read_tsc();
    boot::stall(Duration::from_millis(1));
    let tsc2 = read_tsc();
    (tsc2 - tsc1) * 1000
}

pub fn frequency() -> Option<u64> {
    if let Some(freq) = tsc_freq() {
        return Some(freq);
    }
    let calib_freq = calibrate();
    match calib_freq {
        0 => None,
        _ => Some(calib_freq),
    }
}

pub fn ticks_to_us(ticks: u64, freq: u64) -> u64 {
    ((ticks as u128 * 1_000_000) / freq as u128) as u64
}
