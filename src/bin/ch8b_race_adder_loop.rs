#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use core::ptr::addr_of_mut;
use user_lib::{exit, get_time, thread_create, waittid, yield_};

static mut A: usize = 0;
static mut OCCUPIED: bool = false;
const PER_THREAD: usize = 1000;
const THREAD_COUNT: usize = 16;

/// 线程入口：使用普通布尔变量进行忙等“伪锁”保护共享变量（存在竞态风险）
unsafe fn f() -> ! {
    let mut t = 2usize;
    // 每次循环尝试获取非原子标志并执行一次 RMW 序列
    for _ in 0..PER_THREAD {
        // 忙等等待标志变为 false；读取与后续写入之间存在竞态窗口
        while OCCUPIED {
            yield_();
        }
        // 设置占用标志（非原子写，无内存序保证）
        OCCUPIED = true;
        // 临界区开始：下面对 A 的访问并没有真正的原子互斥保证
        let a = addr_of_mut!(A);
        let cur = a.read_volatile();
        // 纯计算：放大潜在的交错机会
        for _ in 0..500 {
            t = t * t % 10007;
        }
        a.write_volatile(cur + 1);
        // 释放标志（其他线程可能立即观察到，也可能乱序）
        OCCUPIED = false;
    }

    exit(t as i32)
}

#[no_mangle]
/// 程序入口：启动多个使用布尔忙等伪锁的线程并校验结果
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
    println!("race adder using loop test passed!");
    0
}
