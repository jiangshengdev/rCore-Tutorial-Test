#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, thread_create, waittid};

/// 线程 A：打印字符 'a' 并进行一些纯计算，最终以退出码 1 结束
pub fn thread_a() -> ! {
    let mut t = 2i32;
    // 重复多次以制造充足的调度交错机会
    for _ in 0..1000 {
        // 打印一个字符（I/O 可能触发调度）
        print!("a");
        // 纯计算：延长一次迭代的执行时间，放大交错效果
        for __ in 0..5000 {
            t = t * t % 10007;
        }
    }
    println!("{}", t);
    exit(1)
}

/// 线程 B：打印字符 'b' 并进行一些纯计算，最终以退出码 2 结束
pub fn thread_b() -> ! {
    let mut t = 2i32;
    // 与 A/C 对称，便于观察交错输出
    for _ in 0..1000 {
        // 打印一个字符
        print!("b");
        // 纯计算：延长一次迭代的执行时间
        for __ in 0..5000 {
            t = t * t % 10007;
        }
    }
    println!("{}", t);
    exit(2)
}

/// 线程 C：打印字符 'c' 并进行一些纯计算，最终以退出码 3 结束
pub fn thread_c() -> ! {
    let mut t = 2i32;
    // 结构对称，便于对比
    for _ in 0..1000 {
        // 打印一个字符
        print!("c");
        // 纯计算：延长执行时间
        for __ in 0..5000 {
            t = t * t % 10007;
        }
    }
    println!("{}", t);
    exit(3)
}

#[no_mangle]
/// 程序入口：创建三个线程并等待其结束，校验退出码与 tid 一致
pub fn main() -> i32 {
    // 创建三个线程，返回的 tid 作为后续校验依据
    let tids = [
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];
    // 逐个等待线程退出并读取退出码
    for &tid in &tids {
        let exit_code = waittid(tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
        // 约定：线程退出码等于其 tid，用于基本一致性校验
        assert_eq!(tid, exit_code);
    }
    println!("main thread exited.");
    println!("threads test passed!");
    0
}
