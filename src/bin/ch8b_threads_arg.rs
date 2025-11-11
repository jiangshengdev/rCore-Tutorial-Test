#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::{exit, thread_create, waittid};

/// 线程参数结构：包含要打印的字符与退出码
struct Argument {
    pub ch: char,
    pub rc: i32,
}

/// 线程入口：读取只读参数，循环打印字符并以 rc 作为退出码
fn thread_print(arg: *const Argument) -> ! {
    // 通过裸指针获取只读引用（生命周期由调用方保证）
    let arg = unsafe { &*arg };
    // 重复打印以制造调度交错
    for _ in 0..1000 {
        print!("{}", arg.ch);
    }
    exit(arg.rc)
}

#[no_mangle]
/// 程序入口：创建带参数的多个线程并等待其结束
pub fn main() -> i32 {
    // 保存线程 id 用于后续等待
    let mut v = Vec::new();
    // 参数数组：栈上分配，生命周期覆盖整个 main
    let args = [
        Argument { ch: 'a', rc: 1 },
        Argument { ch: 'b', rc: 2 },
        Argument { ch: 'c', rc: 3 },
    ];
    // 为每个参数创建线程，传递裸指针（只读访问）
    for arg in args.iter() {
        v.push(thread_create(
            thread_print as usize,
            arg as *const _ as usize,
        ));
    }
    // 等待全部线程结束，读取退出码
    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    println!("main thread exited.");
    println!("threads with arg test passed!");
    0
}
