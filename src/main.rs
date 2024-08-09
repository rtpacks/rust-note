fn main() {
    /*
     *
     * ## 实战：mini-redis - 深入 async & tokio 异步原理
     *
     * -《unit 71-async 异步编程：概念介绍》
     * -《unit 72-async 异步编程：Future 特征与任务调度》
     * -《unit 73-async 异步编程：Pin 和 Unpin》
     * -《unit 74-async 异步编程：Stream 流处理》
     *
     * 在之前的章节中，已经实现过一版简单的 Future 执行器（运行时），但是没有深入 Future 的原理，而在 rust 中 Future 与 async/.await 是非常重要的技能。
     * 现在深入的了解 Future 和 async/await，以更好的掌握 rust 开发。
     *
     * ### Future
     * Future 是一个实现了 std::future::Future 特征的值，该值包含了一系列异步计算过程。
     * 这个过程是惰性的，当执行到 .await 调用或者 block_on pull 时才会被执行。
     *
     * > Future 是一个能**产出值的异步计算**(值可能为空，例如 `()`)。它是异步函数的返回值和被执行的关键，异步函数则是异步编程的核心，所以 Future 特征是 Rust 异步编程的核心。
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
     * rust 中 Future 设计的非常巧妙，采用是事件通知的方式以便提高效率，这与 JavaScript 的 DOM 事件触发非常相似：
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
     * Future 是惰性的，需要在 poll 函数调用后才会真正执行，同时 poll 只会获取异步任务执行的状态，对异步任务执行流程和结果没有任何影响。
     *
     * 当前 poll 函数执行时获取的状态有两种：
     * - Future 可以被完成，则会返回 Poll::Ready(result)
     * - Future 仍在执行，则返回 `Poll::Pending`，并且安排一个 wake 回调函数：当未来 Future 准备好进一步执行时，该回调函数会被调用，接着管理该 Future 的执行器(例如 block_on 函数)收到信息会再次调用 poll 方法，此时 Future 就可以继续执行了。
     *
     * 这种 “事件通知 -\> 执行” 的方式可以精确的执行该 Future，要比定时轮询所有 Future 来的高效。
     *
     *
     */
}
