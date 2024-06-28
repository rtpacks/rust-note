use std::{thread, time::Duration};

use futures::Future;
use tokio::net::TcpSocket;

fn main() {
    /*
     *
     * ## async 异步编程：Future 特征与任务调度
     * Future 是一个能**产出值的异步计算**(值可能为空，例如 `()`)。它是异步函数的返回值和被执行的关键，异步函数则是异步编程的核心，所以 Future 特征是 Rust 异步编程的核心。
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
     * 以一个从 socket 读取数据的场景为例：
     * - 如果有数据，可以直接读取数据并返回 Poll::Ready(data)
     * - 如果没有数据，Future 会被阻塞且不会再继续执行，此时它会注册一个 wake 函数，当 socket 数据准备好时，该函数将被调用以通知执行器 Future 已经准备好，可以继续执行，然后执行器再次调用 poll 获取状态。
     *
     * 伪代码流程：
     * ```rust
     * pub struct SocketRead<'a> {
     *     socket: &'a Socket,
     * }
     *
     * impl SimpleFuture for SocketRead<'_> {
     *     type Output = Vec<u8>;
     *
     *     fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
     *         if self.socket.has_data_to_read() {
     *             // socket有数据，写入buffer中并返回
     *             Poll::Ready(self.socket.read_buf())
     *         } else {
     *             // socket中还没数据，注册一个`wake`函数，当数据可用时，该函数会被调用，
     *             // 然后当前Future的执行器会再次调用`poll`方法，此时就可以读取到数据
     *             self.socket.set_readable_callback(wake);
     *             Poll::Pending
     *         }
     *     }
     * }
     * ```
     *
     * 通过 Future，无需开发者手动管理轮询逻辑，在数据未准备好前注册 wake 函数并返回 `Poll::Pedning` 状态。
     * 执行器暂停当前 Future 的执行，等数据准备好时，socket 内部调用注册的 wake 回调函数通知执行器可以运行当前的 Future，运行后获取 `Poll::Ready` 状态，表示可以结束。
     *
     *
     * 这种由执行器调度执行，回调函数作为通信触发的方式，能为 IO 密集型带来极高的并发量，并且可以做到无内存分配的状态机即无需手动管理执行状态和维护相关的同步信息。
     *
     * 一个 Future 可以在内部管理多个子 Future，并发执行但统一结束：
     * ```rust
     * // 一个 Future 可以管理多个子 Future，使其并发执行。之所以可以并发，是因为两个子 Future 的轮询可以交替进行，一个阻塞另一个就可以立刻执行，反之亦然
     * pub struct Join<FutureA, FutureB> {
     *     // 结构体的每个字段都包含一个 Future，可以运行直到完成，等到当前 Future 完成后，字段会被设置为 `None`. 这样 Future 完成后就不会再被轮询
     *     a: Option<FutureA>,
     *     b: Option<FutureB>,
     * }
     *
     * impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
     * where
     *     FutureA: SimpleFuture<Output = ()>,
     *     FutureB: SimpleFuture<Output = ()>,
     * {
     *     type Output = ();
     *     fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
     *         // 尝试去完成一个 Future `a`，等到当前 Future 完成后，字段会被设置为 `None`. 这样 Future 完成后就不会再被轮询
     *         if let Some(a) = &mut self.a {
     *             if let Poll::Ready(()) = a.poll(wake) {
     *                 self.a.take();
     *             }
     *         }
     *
     *         // 尝试去完成一个 Future `b`，等到当前 Future 完成后，字段会被设置为 `None`. 这样 Future 完成后就不会再被轮询
     *         if let Some(b) = &mut self.b {
     *             if let Poll::Ready(()) = b.poll(wake) {
     *                 self.b.take();
     *             }
     *         }
     *
     *         if self.a.is_none() && self.b.is_none() {
     *             // 两个 Future都已完成 - 可以成功地统一返回
     *             Poll::Ready(())
     *         } else {
     *             // 至少还有一个 Future 没有完成任务，因此返回 `Poll::Pending`。当该 Future 再次准备好时，通过调用`wake()`函数来继续执行
     *             Poll::Pending
     *         }
     *     }
     * }
     * ```
     *
     * Future 管理子 Future 需要注意一点：避免 Future 完成后被再次执行的情况，这里通过 Option 实现，将已完成的 Future 从 Some 变为 None。
     *
     * 除了并发执行外，使用 Future 管理子 Future 也可以实现串行执行，并发执行和串行执行是最基础的使用 Future 特征表达异步控制流。
     *
     * 在实际场景中，通知外部的方式不会像 wake 函数这么简单，想象一下在一个成百上千的 Tcp 连接（Future）中，
     * **所有的 Future 共享一个 waker**，如果 wake 不携带数据，执行器就不能确定是哪个 Future 应该被唤醒并 poll。
     * 为了能区分由不同 Future wake 触发的信息，需要一个能携带数据的通信方式。
     *
     * rust 通过 Context 和 Waker 的组合，每个 Future 在注册其 wake 函数时，可以将自身的信息存储在 Waker 中。当 wake 被调用时，Future 的自身信息会被传递给执行器，从而使执行器能够正确识别并调度特定的 Future。
     *
     * futures库中 Future 特征的定义：
     * ```rust
     * trait Future {
     *     type Output;
     *     // 相比较 SimpleFuture，futures 中的 Future 特征主要由两点不同
     *     // 1. `self` 是 `Pin<&mut Self>`，而不是 `&mut self`
     *     // 2. `wake: fn()` 修改为 `cx: &mut Context<'_>`:
     *     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
     * }
     * ```
     *
     */

    enum Poll<T> {
        Ready(T),
        Pending,
    }
    trait SimpleFuture {
        type Output;
        fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
    }

    // Socket 伪代码流程
    /*
     * pub struct SocketRead<'a> {
     *     socket: &'a Socket,
     * }
     *
     * impl SimpleFuture for SocketRead<'_> {
     *     type Output = Vec<u8>;
     *
     *     fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
     *         if self.socket.has_data_to_read() {
     *             // socket有数据，写入buffer中并返回
     *             Poll::Ready(self.socket.read_buf())
     *         } else {
     *             // socket中还没数据
     *             // 注册一个`wake`函数，当数据可用时，该函数会被调用，
     *             // 然后当前Future的执行器会再次调用`poll`方法，此时就可以读取到数据
     *             self.socket.set_readable_callback(wake);
     *             Poll::Pending
     *         }
     *     }
     * }
     */

    // 一个 Future 可以管理多个子 Future，使其并发执行。之所以可以并发，是因为两个子 Future 的轮询可以交替进行，一个阻塞另一个就可以立刻执行，反之亦然
    pub struct Join<FutureA, FutureB> {
        // 结构体的每个字段都包含一个 Future，可以运行直到完成，等到当前 Future 完成后，字段会被设置为 `None`. 这样 Future 完成后就不会再被轮询
        a: Option<FutureA>,
        b: Option<FutureB>,
    }

    impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
    where
        FutureA: SimpleFuture<Output = ()>,
        FutureB: SimpleFuture<Output = ()>,
    {
        type Output = ();
        fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
            // 尝试去完成一个 Future `a`，等到当前 Future 完成后，字段会被设置为 `None`. 这样 Future 完成后就不会再被轮询
            if let Some(a) = &mut self.a {
                if let Poll::Ready(()) = a.poll(wake) {
                    self.a.take();
                }
            }

            // 尝试去完成一个 Future `b`，等到当前 Future 完成后，字段会被设置为 `None`. 这样 Future 完成后就不会再被轮询
            if let Some(b) = &mut self.b {
                if let Poll::Ready(()) = b.poll(wake) {
                    self.b.take();
                }
            }

            if self.a.is_none() && self.b.is_none() {
                // 两个 Future都已完成 - 可以成功地统一返回
                Poll::Ready(())
            } else {
                // 至少还有一个 Future 没有完成任务，因此返回 `Poll::Pending`。当该 Future 再次准备好时，通过调用`wake()`函数来继续执行
                Poll::Pending
            }
        }
    }
}
