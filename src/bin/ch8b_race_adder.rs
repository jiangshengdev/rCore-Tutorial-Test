#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use core::ptr::addr_of_mut;
use user_lib::{exit, get_time, thread_create, waittid};

// 共享计数器：多个线程并发执行读-改-写（无同步），存在数据竞争
static mut A: usize = 0;
// 每个线程自增的次数（负载规模）
const PER_THREAD: usize = 1000;
// 并发线程数量
const THREAD_COUNT: usize = 16;

/// 线程入口：在无任何同步的情况下对共享变量自增（存在数据竞争）
unsafe fn f() -> ! {
    let mut t = 2usize;
    // 每次循环执行一次“读-改-写”，该序列不是原子操作
    for _ in 0..PER_THREAD {
        // 获取共享变量地址（未加锁）
        let a = addr_of_mut!(A);
        // 非原子读取：可能与其他线程写操作交错
        let cur = a.read_volatile();
        // 纯计算：拉长一次迭代时间，放大交错概率
        for _ in 0..500 {
            t = t * t % 10007;
        }
        // 非原子写回：典型读-改-写(RMW)竞态窗口
        a.write_volatile(cur + 1);
    }
    exit(t as i32)
}

#[no_mangle]
/// 程序入口：并发启动多个竞争线程并等待结束，最后断言结果
pub fn main() -> i32 {
    // 记录起始时间
    let start = get_time();
    // 保存线程标识（id）
    let mut v = Vec::new();
    // 创建多个线程并发执行函数 f
    for _ in 0..THREAD_COUNT {
        v.push(thread_create(f as usize, 0) as usize);
    }
    // 等待线程结束并收集退出码（仅用于耗时里的计算结果）
    let mut time_cost = Vec::new();
    for tid in v.iter() {
        time_cost.push(waittid(*tid));
    }
    // 打印用时并断言结果（在无同步下理论上不可靠，教学用）
    println!("time cost is {}ms", get_time() - start);
    assert_eq!(unsafe { A }, PER_THREAD * THREAD_COUNT);
    println!("race adder test passed!");
    0
}
