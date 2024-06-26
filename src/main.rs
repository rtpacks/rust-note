use std::{thread, time::Duration};

fn main() {
    /*
     *
     * ## async 异步编程：Future 特征与任务调度
     * Future 是一个能**产出值的异步计算**(值可能为空，例如 `()`)。
     * 它是异步函数的返回值和被执行的关键，异步函数则是异步编程的核心，所以 Future 特征是 Rust 异步编程的核心。
     *
     * Future 是一个能**产出值的异步计算**，并且，Future 是惰性的，需要在 poll 调用后才会真正执行。
     *
     * 这里 rust 设计的非常巧妙，采用事件通知的方式提高效率，与 JavaScript 的 DOM 事件非常相似。
     *
     * 通常获取一个状态有两种方式：定时轮询、事件触发。定时轮询非常简单，设置好循环及时间间隔即可：
     * ```rust
     * loop {
     *     // let status = fetchStatus();
     *     let status = true;
     *     if status {
     *         return;
     *     }
     *     thread::sleep(Duration::from_secs(1));
     * }
     * ```
     *
     * 事件通知往往与回调相关，更简单来说，**函数是可以当作参数传递的**，外部传入一个回调函数，当内部执行完成/错误时调用回调函数，此时由内部通知外部，外部就可以获取内部的状态。
     * ```rust
     * enum Poll<T> {
     *     Ready(T),
     *     Pending,
     * }
     * trait SimpleFuture {
     *     type Output;
     *     fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
     * }
     * ```
     * 
     * > `fn()` 是一个函数指针类型，表示一个不带参数且不返回值的函数，类似的形式有 `fn(i32) -> i32` `fn(&str) -> String ` 等。
     *
     * Future 是惰性的，需要在 poll 调用后才会真正执行，同时 poll 只会获取异步任务执行的状态，对异步任务执行流程和结果没有任何影响。
     * 当前 poll 获取的状态一般有两种：
     * - Future 可以被完成，则会返回 Poll::Ready(result) 
     * - Future 仍在执行，则返回 `Poll::Pending`，并且安排一个 wake 回调函数：当未来 Future 准备好进一步执行时，该回调函数会被调用，接着管理该 Future 的执行器(例如 block_on 函数)收到信息会再次调用 poll 方法，此时 Future 就可以继续执行了。
     * 
     * 这种 “事件通知 -\> 执行” 的方式可以精确的执行该 Future，要比定时轮询所有 Future 来的高效。
     * 
     *
     */
}
