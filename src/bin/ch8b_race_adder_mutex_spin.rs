#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use core::ptr::addr_of_mut;
use user_lib::{exit, get_time, thread_create, waittid};
use user_lib::{mutex_create, mutex_lock, mutex_unlock};

static mut A: usize = 0;
const PER_THREAD: usize = 1000;
const THREAD_COUNT: usize = 16;

/// 线程入口：使用互斥锁保护共享变量的自增，避免数据竞争
unsafe fn f() -> ! {
    let mut t = 2usize;
    // 每次循环持有锁执行一次原子化的“读-改-写”序列
    for _ in 0..PER_THREAD {
        // 加锁：保证后续访问互斥
        mutex_lock(0);
        // 临界区开始
        let a = addr_of_mut!(A);
        let cur = a.read_volatile();
        // 纯计算：拉长临界区时间以观察互斥开销
        for _ in 0..500 {
            t = t * t % 10007;
        }
        a.write_volatile(cur + 1);
        // 临界区结束
        mutex_unlock(0);
    }
    exit(t as i32)
}

#[no_mangle]
/// 程序入口：创建互斥锁，启动多个累加线程并做结果校验
pub fn main() -> i32 {
    // 记录起始时间
    let start = get_time();
    // 创建互斥锁 id=0
    assert_eq!(mutex_create(), 0);
    // 保存线程 id
    let mut v = Vec::new();
    // 并发启动多个加锁版本线程
    for _ in 0..THREAD_COUNT {
        v.push(thread_create(f as usize, 0) as usize);
    }
    // 等待全部结束
    let mut time_cost = Vec::new();
    for tid in v.iter() {
        time_cost.push(waittid(*tid));
    }
    // 打印耗时与结果校验
    println!("time cost is {}ms", get_time() - start);
    assert_eq!(unsafe { A }, PER_THREAD * THREAD_COUNT);
    println!("race adder using spin mutex test passed!");
    0
}
