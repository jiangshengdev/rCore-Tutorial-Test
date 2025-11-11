#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use core::ptr::addr_of_mut;
use core::sync::atomic::{AtomicBool, Ordering};
use user_lib::{exit, get_time, thread_create, waittid, yield_};

// 共享计数器：受原子自旋锁保护
static mut A: usize = 0;
// 自旋锁标志：CAS(比较并交换)获取/释放，Relaxed(宽松)内存序仅为演示
static OCCUPIED: AtomicBool = AtomicBool::new(false);
// 每线程自增次数
const PER_THREAD: usize = 1000;
// 并发线程数量
const THREAD_COUNT: usize = 16;

/// 线程入口：使用 AtomicBool（原子布尔）构造的自旋锁保护共享变量自增
unsafe fn f() -> ! {
    let mut t = 2usize;
    // 通过 CAS(比较并交换)抢占锁，失败则让出 CPU 再试
    for _ in 0..PER_THREAD {
        while OCCUPIED
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // 抢锁失败：让出 CPU，降低忙等开销
            yield_();
        }
        // 持有“锁”：进入临界区
        let a = addr_of_mut!(A);
        let cur = a.read_volatile();
        // 纯计算：放大临界区时间
        for _ in 0..500 {
            t = t * t % 10007;
        }
        a.write_volatile(cur + 1);
        // 释放“锁”
        OCCUPIED.store(false, Ordering::Relaxed);
    }
    exit(t as i32)
}

#[no_mangle]
/// 程序入口：并发创建多个原子自旋锁版本的线程并等待结束
pub fn main() -> i32 {
    let start = get_time();
    let mut v = Vec::new();
    for _ in 0..THREAD_COUNT {
        v.push(thread_create(f as usize, 0) as usize);
    }
    let mut time_cost = Vec::new();
    for tid in v.iter() {
        time_cost.push(waittid(*tid));
    }
    println!("time cost is {}ms", get_time() - start);
    assert_eq!(unsafe { A }, PER_THREAD * THREAD_COUNT);
    println!("race adder using atomic test passed!");
    0
}
