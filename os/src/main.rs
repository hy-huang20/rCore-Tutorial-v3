#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

//use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE, INPUT_CONDVAR};
use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};
extern crate alloc;

#[macro_use]
extern crate bitflags;

use log::*;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod net;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;

core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

use lazy_static::*;
use sync::UPIntrFreeCell;

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

use embassy_executor::Executor;
use embassy_executor::Spawner;
use static_cell::StaticCell;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(kernel_start(spawner)).unwrap();
    });
}

async fn test(){
    let f1 = async {
        println!("========= async test f1 ==============");
    };
    let f2 = async {
        println!("========= async test f2 ==============");
    };
    let f3 = async {
        println!("========= async test f3 ==============");
    };
    f3.await;
    f2.await;
    f1.await;
}

#[embassy_executor::task]
async fn kernel_start(spawner: Spawner) {
    clear_bss();
    logging::init();
    mm::init();
    UART.init();
    info!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    info!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    info!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    info!("KERN: init trap");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    board::device_init();
    fs::list_apps();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    // 测试异步调用
    test().await;

    // 开始线程调度
    task::run_tasks().await;
    panic!("Unreachable in rust_main!");
}
