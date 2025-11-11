#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::exit;
use user_lib::{semaphore_create, semaphore_down, semaphore_up};
use user_lib::{sleep_blocking, thread_create, waittid};

const SEM_SYNC: usize = 0;

/// 线程一：完成自身工作后通过 up 增加信号量计数，唤醒等待者
unsafe fn first() -> ! {
    // 工作模拟：延迟一段时间确保 second 先进入阻塞
    sleep_blocking(10);
    println!("First work and wakeup Second");
    // 增加计数：若 second 已在 down 阻塞则被唤醒
    semaphore_up(SEM_SYNC);
    exit(0)
}

/// 线程二：先执行 down 等待信号量，确认线程一已完成后继续
unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait first");
    // 初始计数为 0 → 在此阻塞直到 first 调用 up
    semaphore_down(SEM_SYNC);
    println!("Second can work now");
    exit(0)
}

#[no_mangle]
/// 程序入口：创建初始值为 0 的同步信号量并启动两个线程
pub fn main() -> i32 {
    // create semaphores
    assert_eq!(semaphore_create(0) as usize, SEM_SYNC);
    // create threads
    let threads = [
        thread_create(first as usize, 0),
        thread_create(second as usize, 0),
    ];
    // wait for all threads to complete
    for &thread in &threads {
        waittid(thread as usize);
    }
    println!("sync_sem passed!");
    0
}
