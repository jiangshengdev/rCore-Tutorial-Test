// =============================================================
// 参考资料：
//   https://blog.aloni.org/posts/a-stack-less-rust-coroutine-100-loc/
//   https://github.com/chyyuu/example-coroutine-and-thread/tree/stackless-coroutine-x86
// -------------------------------------------------------------
// 本示例演示“栈less（无独立栈）”协程的一种极简实现思路：
// 1. 利用 Rust async/await 与 Future::poll 的驱动模型。
// 2. 不为每个协程分配单独的执行栈，而是通过状态机在多次 poll 之间推进逻辑。
// 3. Executor 只是维护一个 Future 队列，轮询执行，若返回 Poll::Pending 再放回队列。
// 4. Task 内部通过一个简单的二值状态(Halted / Running)来模拟“挂起”与“继续”。
// 5. Waker 在这里实现为“空操作”，因为我们使用轮询调度，不依赖外部事件唤醒。
// -------------------------------------------------------------
// 运行流程概述：
//   main -> 创建 Executor -> 注册多个异步任务(Future) -> exec.run() 循环 poll
//   每个任务内部调用 waiter().await，第一次 poll 时切换状态并返回 Ready；
//   下一次 poll 时再进入 waiter，返回 Pending，Executor 便把该 Future 重新排队。
//   如此交替即可观察到任务的多阶段输出，模拟协程“让出”与“继续”。
// =============================================================
// 注意：为了演示原理，未实现真实的事件驱动唤醒；Waker 所有回调均为空，实现最小可行示例。
// =============================================================
#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;
use core::future::Future; // Future 特征，定义异步任务的 poll 接口
use core::pin::Pin; // Pin 保证 Future 在内存地址上不再移动，符合 async 状态机生成的要求
use core::task::{Context, Poll}; // Context 携带 Waker，Poll 表示任务当前是否完成或仍需继续
use core::task::{RawWaker, RawWakerVTable, Waker}; // 手工构造一个最简 Waker（空唤醒）

use alloc::collections::VecDeque; // 任务队列（FIFO），Pending 时重新排入尾部

use alloc::boxed::Box;

/// Task 的内部状态机枚举
/// Halted  : 表示当前调用 waiter().await 时需要“挂起”（返回 Pending）
/// Running : 表示当前调用 waiter().await 时可以继续往下执行（返回 Ready）
enum State {
    Halted,
    Running,
}

/// 一个最小的“协程控制块”结构：仅保存一个状态位
struct Task {
    state: State,
}

impl Task {
    /// 返回一个 Waiter Future，用于在 async/await 中模拟协程的“让出”与“继续”。
    /// 第一次 poll：如果 state 为 Halted -> 切 Running 并返回 Ready（表示这一步完成，继续执行后续 async 语句）。
    /// 第二次 poll：如果 state 为 Running -> 切 Halted 并返回 Pending（表示需要再次调度）。
    fn waiter(&mut self) -> Waiter<'_> {
        Waiter { task: self }
    }
}

/// Waiter 是一个 Future，通过对 Task.state 的切换实现“半步执行”。
/// 每一次 await 会在 Ready 与 Pending 间交替，从而把控制流交回 Executor。
struct Waiter<'a> {
    task: &'a mut Task,
}

impl<'a> Future for Waiter<'a> {
    type Output = (); // 不返回数据，仅表示“阶段完成”

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        // 这里不使用 _cx 中的 Waker，因为示例是主动轮询
        match self.task.state {
            State::Halted => {
                // 状态为 Halted 时，切换为 Running，并立刻返回 Ready，表示本阶段完成
                self.task.state = State::Running;
                Poll::Ready(())
            }
            State::Running => {
                // 状态为 Running 时，切换为 Halted，并返回 Pending，Executor 会再次调度它
                self.task.state = State::Halted;
                Poll::Pending
            }
        }
    }
}

/// 简易执行器：维护一个 Future 队列，轮询调度。
/// - poll 返回 Ready：任务完成，丢弃。
/// - poll 返回 Pending：任务尚未完成，重新排入队列尾部等待后续轮询。
struct Executor {
    tasks: VecDeque<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    /// 创建一个空的执行器
    fn new() -> Self {
        Executor {
            tasks: VecDeque::new(),
        }
    }

    /// 推入一个通过闭包生成的 Future。
    /// 传入闭包 C：由一个初始 Task (state=Running) 构造出对应异步逻辑 F。
    /// 这里使用 FnOnce(Task) -> F 的模式，方便在内部捕获实例编号等 move 变量。
    fn push<C, F>(&mut self, closure: C)
    where
        F: Future<Output = ()> + 'static,
        C: FnOnce(Task) -> F,
    {
        let task = Task {
            state: State::Running,
        };
        // Box::pin 将 Future 堆分配并固定地址，满足 async 状态机的 Pin 要求
        self.tasks.push_back(Box::pin(closure(task)));
    }

    /// 主轮询函数：按 FIFO 顺序依次 poll 队列中的 Future。
    /// - 如果 Pending：放回队列尾部。
    /// - 如果 Ready：丢弃（表示完成）。
    /// 由于 Waker 是空实现，不会有外部唤醒，所以这里是纯“忙轮询”示例。
    fn run(&mut self) {
        let waker = create_waker(); // 创建一个空操作 Waker
        let mut context = Context::from_waker(&waker);

        while let Some(mut task) = self.tasks.pop_front() {
            match task.as_mut().poll(&mut context) {
                Poll::Pending => {
                    // 未完成，重新排队
                    self.tasks.push_back(task);
                }
                Poll::Ready(()) => {
                    // 已完成：不再入队
                }
            }
        }
    }
}

/// 构造一个“空唤醒” Waker：所有回调函数都什么也不做。
/// 在本示例中我们主动循环 poll，不依赖唤醒机制，因此这是安全且足够的。
pub fn create_waker() -> Waker {
    // Safety: 指向静态 vtable，所有函数为空操作，不会触及非法内存。
    unsafe { Waker::from_raw(RAW_WAKER) }
}

// 静态 RawWaker 与其 vtable。指针数据部分使用 null，表示不需要携带额外状态。
const RAW_WAKER: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);
const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

// 以下四个函数对应 Waker 的底层行为；全部为空操作：
// clone       : 返回同一个静态 RAW_WAKER
// wake        : 不进行实际唤醒
// wake_by_ref : 同上
// drop        : 无资源需要释放
unsafe fn clone(_: *const ()) -> RawWaker {
    RAW_WAKER
}
unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}

#[no_mangle]
pub fn main() -> i32 {
    println!("stackless coroutine Begin..");
    let mut exec = Executor::new();
    println!(" Create futures");
    // 创建三个任务，每个任务内部使用两次 waiter().await 来演示状态切换
    for instance in 1..=3 {
        exec.push(move |mut task| async move {
            println!("   Task {}: begin state", instance);
            task.waiter().await; // 第一次 await：切 Running -> Ready；继续执行下一行
            println!("   Task {}: next state", instance);
            task.waiter().await; // 第二次 await：切 Halted -> Pending；稍后再次被调度
            println!("   Task {}: end state", instance);
        });
    }

    println!(" Running");
    exec.run(); // 开始轮询执行所有 Future
    println!(" Done");
    println!("stackless coroutine PASSED");
    0
}
