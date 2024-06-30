use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use futures::{task, Future, FutureExt};
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
     * 注意，当前 Future 是有一个数据可以表达任务状态的，如 `socket.has_data_to_read`，也就是 poll Future 后获取的状态信息来源于当前 Future 某个数据表达的状态。
     * 简单来说：**Future 一定要有一个能表达任务状态的数据**，这样执行器在 poll Future 时才知道对 Future 的操作是等待 `Poll::Pedning` 还是结束 `Poll::Ready`。
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
     * ### Waker 唤醒任务
     * 正常情况下，Future 是一个耗时任务，在第一次被 poll 时通常还未完成。此时 Future 就需要在自身再次准备好被执行时，借助 Waker 通知执行器，让执行器再次调度执行 Future 自身。
     *
     * 为了简化实现以及降低理解成本，可以新开一个线程，利用线程休眠模拟 Future 的异步任务（例如网络请求），线程休眠中代表 Future 的异步任务正在执行，线程休眠结束代表 Future 的异步任务运行结束，可以通知执行器调用 poll，以便 Future 被执行。
     *
     * 在之前的 SimpleFuture 中有一个注意点：
     * **Future 一定要有一个能表达任务状态的数据**，这样执行器在 poll Future 时才知道对 Future 的操作是等待 `Poll::Pedning` 还是结束 `Poll::Ready`。
     *
     * 根据这个注意点，TimeFuture 需要一个状态用来标识 TimeFuture 的异步任务是否完成，这个状态又是由新线程的休眠状态决定，所以 TimeFuture 要与新线程共享这份数据。
     * 同时由于是在不同线程间共享状态，需要考虑所有权和并发状态，即需要使用 Arc 和 Mutex 两个工具。
     *
     *
     * **整体流程如下：**
     *
     * 新线程与 TimeFuture 共享一个状态，TimeFuture 根据这个状态标识 TimeFuture 的异步任务是否完成。
     * 在生成 TimeFuture 时开始执行异步任务，当前案例中，异步任务就是新线程休眠。
     * 线程休眠结束后修改与 TimeFuture 共享的状态数据并调用 wake，表示 Future 的异步任务执行结束，可以再次被 poll。
     *
     * ```rust
     * struct SharedState {
     *     // 异步任务是否已经结束（线程休眠是否已经结束）
     *     completed: bool,
     * }
     *
     * // TimeFuture 需要一个状态标识是否完成，这个状态是由休眠线程传递的，涉及到多线程需要使用 Arc，并且状态应该是带锁的，避免多线程数据访问问题
     * struct TimeFuture {
     *     shared_state: Arc<Mutex<SharedState>>,
     * }
     * impl Future for TimeFuture {
     *     type Output = ();
     *
     *     fn poll(
     *         self: Pin<&mut Self>,
     *         cx: &mut std::task::Context<'_>,
     *     ) -> std::task::Poll<Self::Output> {
     *         // poll 时检查任务状态，来确定是否可以结束当前 Future
     *         let mut shared_state = self.shared_state.lock().unwrap();
     *
     *         if shared_state.completed {
     *             std::task::Poll::Ready(())
     *         } else {
     *             std::task::Poll::Pending
     *         }
     *     }
     * }
     * ```
     *
     * 以上代码可以描述一个 Future 的运行逻辑，但还缺少三个步骤：
     * 1. Future 的创建
     * 2. Future 执行异步任务（新线程休眠）
     * 3. Future 执行完异步任务（线程休眠）后让执行器再次 poll 当前 Future
     *
     * 创建 Future 和 Future 执行异步任务
     * ```rust
     * // 实现 Future 运行异步任务的逻辑
     * impl TimeFuture {
     *     fn new(duration: Duration) -> Self {
     *         let shared_state = Arc::new(Mutex::new(SharedState { completed: false }));
     *
     *         let _shared_state = Arc::clone(&shared_state);
     *         // 用线程休眠模拟异步任务
     *         thread::spawn(move || {
     *             thread::sleep(duration);
     *             let mut mutex = _shared_state.lock().unwrap();
     *             // 修改异步任务状态，模拟网络结束连接或IO关闭等场景。
     *             // Future 一定要有一个表示执行异步任务状态的数据，这样才能让执行器在 Poll 当前 Future 时知道该结束 `Poll::Ready` 还是等待 `Poll::Pending`
     *             mutex.completed = true;
     *         });
     *
     *         Self { shared_state }
     *     }
     * }
     * ```
     *
     *
     * 第三步，异步任务结束后需要调用 wake 让当前 Future 被再次 poll 执行，wake 应该来自哪？在什么时候、以及怎么注册 wake？
     * 其实很简单，在 Future 特征定义中，poll 函数 `fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;` 的 `cx: &mut Context<'_>` 就是注册、和外部调用的 wake 的来源。
     *
     * wake 注册就是将 wake 传递给外部，用 wake 关联当前 Future 的过程，而让外部调用 wake 函数就是在让执行器再次 poll wake 关联的 Future 的过程。
     * `cx: &mut Context<'_>` 中的 wake 指向的作用域就包含当前 Future 对应的信息，供外部调用时就可以让执行器正确识别并调度当前特定的 Future。
     *
     * > 每个 Future 在注册其 wake 函数时，将自身的信息存储在 Waker 中。**当 wake 被调用时 Future 的自身信息会被传递给执行器**，从而使执行器能够正确识别并调度特定的 Future。
     *
     * 因此，第三步骤其实是当前 Future 被 poll 执行时将 wake 存储起来，然后外部在异步任务结束后，调用 wake 函数让执行器正确识别 Future 并再次 poll 当前 Future 的过程。
     *
     * SharedState 增加存储 waker：
     * ```rust
     * // 利用线程休眠模拟异步任务，如网络请求
     * struct SharedState {
     *     // 异步任务是否已经结束（线程休眠是否已经结束）
     *     completed: bool,
     *     // 当前 Future 被 poll 执行时将 wake 存储起来，然后外部在异步任务结束后，调用 wake 函数让执行器正确识别 Future 并再次 poll 当前 Future
     *     waker: Option<std::task::Waker>,
     * }
     * ```
     *
     * poll Future 时存储 waker
     * ```diff
     * // 实现 Future poll 的逻辑
     * impl Future for TimeFuture {
     *     type Output = ();
     *
     *     fn poll(
     *         self: Pin<&mut Self>,
     *         cx: &mut std::task::Context<'_>,
     *     ) -> std::task::Poll<Self::Output> {
     *         // poll 时检查任务状态，来确定是否可以结束当前 Future
     *         let mut shared_state = self.shared_state.lock().unwrap();
     *
     *         if shared_state.completed {
     *             std::task::Poll::Ready(())
     *         } else {
     * +           // 选择每次都`clone`的原因是： `TimerFuture`可以在执行器的不同任务间移动，如果只克隆一次，
     * +           // 那么获取到的`waker`可能已经被篡改并指向了其它任务，最终导致执行器运行了错误的任务
     * +           shared_state.waker = Some(cx.waker().clone());
     *             std::task::Poll::Pending
     *         }
     *     }
     * }
     * ```
     *
     * 在 Future 异步任务结束时，调用 poll Future 存储的 waker
     * ```diff
     * // 实现 Future 运行异步任务的逻辑
     * impl TimeFuture {
     *     fn new(duration: Duration) -> Self {
     *         let shared_state = Arc::new(Mutex::new(SharedState {
     *             completed: false,
     *             waker: None,
     *         }));
     *
     *         let _shared_state = Arc::clone(&shared_state);
     *         // 用线程休眠模拟异步任务
     *         thread::spawn(move || {
     *             thread::sleep(duration);
     *             let mut mutex = _shared_state.lock().unwrap();
     *             // 修改异步任务状态，模拟网络结束连接或IO关闭等场景。
     *             // Future 一定要有一个表示执行异步任务状态的数据，这样才能让执行器在 Poll 当前 Future 时知道该结束 `Poll::Ready` 还是等待 `Poll::Pending`
     *             mutex.completed = true;
     *
     * +           // 在异步任务结束后，调用 poll Future 的 waker
     * +           if let Some(waker) = mutex.waker.take() {
     * +               waker.wake()
     * +           }
     *         });
     *
     *         Self { shared_state }
     *     }
     * }
     * ```
     *
     * 至此，一个简单的 TimeFuture 就已创建成功，接下来需要让它跑起来。
     *
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

    // 利用线程休眠模拟异步任务，如网络请求
    struct SharedState {
        // 异步任务是否已经结束（线程休眠是否已经结束）
        completed: bool,
        // 当前 Future 被 poll 执行时将 wake 存储起来，然后外部在异步任务结束后，调用 wake 函数让执行器正确识别 Future 并再次 poll 当前 Future
        waker: Option<std::task::Waker>,
    }

    // TimeFuture 需要一个状态标识是否完成，这个状态是由休眠线程传递的，涉及到多线程需要使用 Arc，并且状态应该是带锁的，避免多线程数据访问问题
    struct TimeFuture {
        shared_state: Arc<Mutex<SharedState>>,
    }
    // 实现 Future poll 的逻辑
    impl Future for TimeFuture {
        type Output = ();

        fn poll(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            // poll 时检查任务状态，来确定是否可以结束当前 Future
            let mut shared_state = self.shared_state.lock().unwrap();

            if shared_state.completed {
                std::task::Poll::Ready(())
            } else {
                // 选择每次都`clone`的原因是： `TimerFuture`可以在执行器的不同任务间移动，如果只克隆一次，
                // 那么获取到的`waker`可能已经被篡改并指向了其它任务，最终导致执行器运行了错误的任务
                shared_state.waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }
    }
    // 实现 Future 运行异步任务的逻辑
    impl TimeFuture {
        fn new(duration: Duration) -> Self {
            let shared_state = Arc::new(Mutex::new(SharedState {
                completed: false,
                waker: None,
            }));

            let _shared_state = Arc::clone(&shared_state);
            // 用线程休眠模拟异步任务
            thread::spawn(move || {
                thread::sleep(duration);
                let mut mutex = _shared_state.lock().unwrap();
                // 修改异步任务状态，模拟网络结束连接或IO关闭等场景。
                // Future 一定要有一个表示执行异步任务状态的数据，这样才能让执行器在 Poll 当前 Future 时知道该结束 `Poll::Ready` 还是等待 `Poll::Pending`
                mutex.completed = true;

                // 在异步任务结束后，调用 poll Future 的 waker
                if let Some(waker) = mutex.waker.take() {
                    waker.wake()
                }
            });

            Self { shared_state }
        }
    }
}
