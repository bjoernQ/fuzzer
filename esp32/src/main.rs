#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::print;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc};

extern crate alloc;

mod tst;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;

    extern "C" {
        static mut _heap_start: u32;
    }

    unsafe {
        let heap_start = &_heap_start as *const _ as usize;
        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}

#[entry]
fn main() -> ! {
    init_heap();

    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    print!(">>>");
    tst::main();
    print!("<<<");

    loop {}
}
