use std::{
    pin::Pin,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::Context,
    thread,
    time::Duration,
};

use futures::{
    future::BoxFuture,
    task::{self, ArcWake},
    Future, FutureExt,
};

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
     * **注意：这里的外部是指当前线程外。**
     *
     * SharedState 增加存储 waker：
     * ```rust
     * // 利用线程休眠模拟异步任务，如网络请求
     * enum FutureStatus {
     *     init,
     *     pending,
     *     completed,
     * }
     * struct SharedState {
     *     // 异步任务的状态
     *     status: FutureStatus,
     *     // 当前 Future 被 poll 执行时将 wake 存储起来，然后外部在异步任务结束后，调用 wake 函数让执行器正确识别 Future 并再次 poll 当前 Future
     *     waker: Option<std::task::Waker>,
     * }
     * ```
     *
     * poll Future 时存储 waker，此时才开始执行异步任务，这也是为什么 future 被称为是惰性的，因为只有在第一次 poll 后才会开始执行。在编写 Future 时也需要注意应将异步任务放在第一次 poll 中执行。
     * ```rust
     * // TimeFuture 需要一个状态标识是否完成，这个状态是由休眠线程传递的，涉及到多线程需要使用 Arc，并且状态应该是带锁的，避免多线程数据访问问题
     * struct TimeFuture {
     *     shared_state: Arc<Mutex<SharedState>>,
     * }
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
     *         return match shared_state.status {
     *             FutureStatus::init => {
     *                 // 选择每次都`clone`的原因是： `TimerFuture`可以在执行器的不同任务间移动，如果只克隆一次，
     *                 // 那么获取到的`waker`可能已经被篡改并指向了其它任务，最终导致执行器运行了错误的任务
     *                 shared_state.waker = Some(cx.waker().clone());
     *                 shared_state.status = FutureStatus::pending;
     *
     *                 let _shared_state = Arc::clone(&self.shared_state);
     *                 // 用线程休眠模拟异步任务
     *                 thread::spawn(move || {
     *                     thread::sleep(Duration::from_secs(6));
     *                     let mut mutex = _shared_state.lock().unwrap();
     *                     // 修改异步任务状态，模拟网络结束连接或IO关闭等场景。
     *                     // Future 一定要有一个表示执行异步任务状态的数据，这样才能让执行器在 Poll 当前 Future 时知道该结束 `Poll::Ready` 还是等待 `Poll::Pending`
     *                     mutex.status = FutureStatus::completed;
     *
     *                     // 在异步任务结束后，调用 poll Future 的 waker
     *                     if let Some(waker) = mutex.waker.take() {
     *                         waker.wake()
     *                     }
     *                 });
     *
     *                 std::task::Poll::Pending
     *             }
     *             FutureStatus::pending => std::task::Poll::Pending,
     *             FutureStatus::completed => {
     *                 println!("completed");
     *                 std::task::Poll::Ready(())
     *             }
     *         };
     *     }
     * }
     * // Future 生成
     * impl TimeFuture {
     *     fn new() -> Self {
     *         let shared_state = Arc::new(Mutex::new(SharedState {
     *             status: FutureStatus::init,
     *             waker: None,
     *         }));
     *
     *         Self { shared_state }
     *     }
     * }
     * ```
     *
     * 至此，一个简单的 TimeFuture 就已创建成功，测试代码：
     * ```rust
     * futures::executor::block_on(TimeFuture::new());
     * ```
     *
     * ### 执行器 Executor
     * Rust 的 Future 是惰性的，只有被 poll 后才会开始执行，rust poll Future 一般有两种方式：
     * - 在 async 函数中使用 .await 来调用另一个 async 函数，这个方式只能解决 async 内部的问题，因为 `.await` 只允许用在 async 函数中。因此这种方式不能在非 async 函数中**阻塞等待 async 函数的完成**，也就可能导致 async 函数的 Future 还未开始执行，当前的非 async 函数就已经退出函数栈。
     * - 执行器 executor 会管理一批 Future (最外层的 async 函数)，然后通过不停地 poll 推动它们直到完成。最开始，执行器会先 poll 一次 Future ，然后不会再主动去 poll，而是等待 Future 通过调用 wake 函数来通知它可以继续，它才会继续去 poll。这种 wake 通知然后 poll 的方式会不断重复，直到 Future 完成。
     *
     * 在 `TimeFuture` 的实现测试中，使用的就是执行器 poll Future。执行器会不断 poll Future 直至结束。下面来构建一个自己的执行器，用来运行自定义的 `TimeFuture`。
     *
     * #### 构建执行器
     * > 这里将每个步骤描述的比较详细，如果只需要了解，可以看：https://course.rs/advance/async/future-excuting.html#执行器-executor
     *
     * 在 rust 中，执行器是**不停地 poll 推动 Future 获取状态，直到 Future 完成**。需要注意的是，执行器会先 poll 一次 Future，然后不会再主动去 poll，而是等待 Future 通过调用 wake 函数来通知执行器可以继续，执行器才会继续去 poll。
     *
     * 观察原有的 TimeFuture 实现，会发现 TimeFuture 不会自动触发，并且在被动触发后只会在异步任务结束时触发一次 wake。
     * 这与执行器会先 poll 一次 Future，然后等待 Future 调用 wake 来通知执行器可以继续，形成执行器不停的 poll Future 的场景还少了两点：
     * 1. 执行器需要主动触发一次 Future
     * 2. Future 需要不断地触发 wake，达到执行器不停的 poll Future 的目的
     *
     * rust 通过维护一个消息通道（channel）来实现执行器 Executor 的调度执行，这其中的逻辑与 JavaScript 的事件循环队列非常类似。
     * 这里通过同步消息通道，简单实现一个 Executor，具体划分为：
     * - 执行器 `Executor` 作为通道的接收者 Receiver，如果有 Future 进入消息通道，Executor 就开始执行 Future，如果消息通道为空，则阻塞当前函数。
     * - 创建者 `Spawner` 作为发送者 Transmitter，将创建 Future 并将其发送到消息通道中，触发执行器 `Executor` 去 poll Future。
     *
     * **对于当前 Future 缺少的两个点**
     *
     * 第一点：执行器 Executor 主动触发 poll 一次 Future。
     * 这一点很容易实现，模拟流程：创建者（发送者）创建 Future，然后将 Future 发送到消息通道，接收者（执行器）接收，然后主动 poll 一次 Future。
     *
     * 第二点：Future 需要不断地触发 wake，达到执行器不停的 poll Future 的目的。
     * 这一点的实现并不简单，有两种方式可以不断地触发 wake：
     * 1. 没有任务调度系统，任务状态由 Future 自身管理。Future 在被第一次 poll 后，主动调用 wake，触发 poll。
     * 2. 有任务调度系统，任务状态由 Executor 管理，可操控性大，Future 也不会引入无关的逻辑。
     *
     * 第一种方式，没有任务调度系统，任务状态由 Future 自身管理，很明显可控性不大，如想要根据某个条件切换，Future 的 poll 逻辑耦合度很大：
     * ```diff
     * impl Future for TimeFuture {
     *     type Output = ();
     *
     *     fn poll(
     *         self: Pin<&mut Self>,
     *         cx: &mut std::task::Context<'_>,
     *     ) -> std::task::Poll<Self::Output> {
     *         shared_state.waker = Some(cx.waker().clone());
     * +       shared_state.waker.wake();
     *
     *         ...
     *
     *         return match shared_state.status {
     *              ...
     *         };
     *     }
     * }
     * ```
     *
     * 第二种方式，有任务调度系统，由 Executor 调度管理，可操控性大，Future 的 poll 函数也不会引入无关的逻辑。分析流程：
     * 1. 构建一个消息任务队列，生成执行器（接收者）和创建者（发送者）。
     * 2. 执行器从消息通道中阻塞性的接收 Future，当 Future 状态未完成时，会默认调用(第1次或第N+1次) Future 的 poll 函数获取 Future 状态。
     * 3. 如果 Future 未完成，为了让执行器不停的 poll Future，要将 Future **重新发送到消息通道**中，这样就会重复2步骤，让执行器再次 poll Future。
     *
     * > 为什么要使用任务队列来存储待执行的 Future?
     * > 在使用 rust 提供的执行器时，提到过 Future 的执行方式：“事件通知 -\> 执行” 的方式可以精确的执行该 Future，要比定时轮询所有 Future 来的高效。
     * > 使用任务队列，就是为了提高效率。
     *
     * 构建消息通道，生成执行器（接收者）和创建者（发送者），伪代码：
     * ```rust
     * // 通过同步消息通道模拟 Executor 调度流程
     * struct FutureChannel(Spawner, Executor);
     * impl FutureChannel {
     *     fn new(size: usize) -> Self {
     *         let (tx, rx) = mpsc::sync_channel(size);
     *         Self(Spawner { task_sender: tx }, Executor { task_queue: rx })
     *     }
     * }
     * ```
     *
     * 执行器从消息通道中阻塞性的接收 Future，当 Future 状态为未完成时，调用 Future 的 poll 函数
     * ```rust
     * struct Executor {
     *     // 一个Future一般只会在一个线程中执行，不需要 Mutex，但是编译器无法知道`Future`只会在一个线程内被修改，并不会被跨线程修改。
     *     // 因此需要使用`Mutex`来满足编译器对线程安全的校验，如果是生产级的执行器实现，不会使用`Mutex`，因为会带来性能上的开销，取而代之的是使用`UnsafeCell`
     *     task_queue: Receiver<Arc<Mutex<BoxFuture<'static, ()>>>>,
     * }
     * impl Executor {
     *     fn run(&self) {
     *         // Executor 作为消息通道的接收者，可以从消息通道中取出需要被 poll 的 Future
     *         while let Ok(future) = self.task_queue.recv() {
     *             let mut mutex_future = future.lock().unwrap();
     *
     *             if mutex_future.as_mut().poll(context).is_pending() {
     *                 // 重新放回任务队列
     *             };
     *         }
     *     }
     * }
     * ```
     *
     * 很明显，在实现的代码中存在两个问题：
     * 1. 缺少 Future 特征 poll 函数的参数 Context，也就是类似 SimpleFuture 特征中用于唤起功能的 wake 函数，wake 函数怎么让 Future 重新放回消息通道
     * 2. 缺少发送者，无法将 Future 重新放回消息通道中
     *
     * 以上两点都是在解决怎么将 Future 重新放回消息通道，解决这个问题是自定义执行器的关键。
     *
     * 分析以上信息，可以得到两个重点：
     * 1. 如果一个 Future 被 Executor poll 后需要重新放入任务队列，那么 Executor 在 poll Future 时必须要拿到发送者，才可以将 Future 重新放入任务队列
     * 2. 如果调用 wake 函数后需要将 Future 重新放入任务队列，需要拿到发送者与 Future，才可以将 Future 重新放入任务队列
     *
     * 执行器在外部将 Future 再次放入任务队列的形式，也可以统一到调用 wake 将 Future 放入任务队列的形式上。
     *
     * 以上两个问题其实比较好解决，但是比较绕。将 Future 与发送者关联起来形成新的结构体 FutureWrapper，将新的结构体发送到任务队列，这样执行器拿到的 Future 都是带有发送者的 FutureWrapper。
     *
     * 在 rust 的 waker 的介绍中，有这么一段描述：
     * 每个 Future 在注册其 wake 函数时，可以将自身的信息存储在 Waker 中。当 wake 被调用时，Future 的自身信息会被传递给执行器，从而使执行器能够正确识别并调度特定的 Future。
     *
     * 这一段描述的就是将 Future 与发送者关联，并结合 Waker 的过程。因此，刚开始这句话并不是特别准确：
     * > wake 注册就是将 wake 传递给外部，用 wake 关联当前 Future 的过程，而让外部调用 wake 函数就是在让执行器再次 poll wake 关联的 Future 的过程。
     *
     * 修改消息通道发送的数据类型，构建发送者和接收者(执行器)：
     * ```rust
     * // 通过同步消息通道模拟 Executor 调度流程
     * struct FutureChannel(Spawner, Executor);
     * impl FutureChannel {
     *     fn new(size: usize) -> Self {
     *         let (tx, rx) = mpsc::sync_channel(size);
     *         Self(Spawner { task_sender: tx }, Executor { task_queue: rx })
     *     }
     * }
     *
     * struct FutureWrapper {
     *     future: Mutex<Option<BoxFuture<'static, ()>>>,
     *     task_sender: SyncSender<Arc<FutureWrapper>>,
     * }
     *
     * #[derive(Clone)]
     * struct Spawner {
     *     task_sender: SyncSender<Arc<FutureWrapper>>, // 发送 FutureWrapper
     * }
     * impl Spawner {
     *     fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
     *         let future = future.boxed();
     *         let wrapper = FutureWrapper {
     *             future: Mutex::new(Some(future)),
     *             task_sender: self.task_sender.clone(),
     *         };
     *         // 将 Future 发送到任务通道中
     *         self.task_sender
     *             .send(Arc::new(wrapper))
     *             .expect("任务队列已满");
     *     }
     * }
     *
     * struct Executor {
     *     task_queue: Receiver<Arc<FutureWrapper>>,
     * }
     * impl Executor {
     *     fn run(&self) {
     *         // Executor 作为消息通道的接收者，可以从消息通道中取出需要被 poll 的 Future
     *         while let Ok(wrapper) = self.task_queue.recv() {
     *             let mut mutex_future = wrapper.future.lock().unwrap();
     *             if let Some(mut future) = mutex_future.take() {
     *                 if future.as_mut().poll(cx).is_pending() {
     *                     // Future 未完成时，将 Future 再次放入任务队列中
     *                     wrapper.task_sender.send(wrapper.clone());
     *                 };
     * 
     *             }
     *         }
     *     }
     * }
     * ```
     * Executor 中已经可以拿到发送者，并将携带 Future 的 FutureWrapper 重新发送到任务队列，剩下一个问题，如何统一到调用 wake 将 Future 放入任务队列，它能使用在两方面：
     * - 在 poll 函数内部调用 wake，将 Future 重新放入任务队列
     * - 在 执行器中调用 wake，外部调用也能将 Future 放入任务队列
     *
     * SimpleFuture 的 wake 和 Future 的 Context 都属于唤起作用，即将 Future 重新放入消息通道中，不同的是 Context 携带了数据。
     * 其实，Waker 和 wake 函数并不是高深的魔法，Waker 是存储信息对象，wake 函数是一个触发操作，功能是将 Future 重新发送到消息队列中，阻塞等待的 Executor 就会接收并自动执行 Future。
     *
     * > 注意：虽然发送者和接收者是生成消息通道时产生的，但是这并不意味发送者和接收者不能进入消息通道，创建消息通道其实是创建了三个数据结构，发送者、接收者、消息通道。
     *
     * 现在任务队列的数据类型变为 `FutureWrapper`，它携带了 Future 和发送者，如果再让他实现一个操作 wake，利用自身的发送者将自身的 Future 发送到消息通道中，那么问题就可以解决了。
     *
     * 定义一个 MyWaker 特征，提供 wake 方法，能将自身重新发送到任务队列中：
     * ```rust
     * trait MyWaker {
     *     fn wake(self: &Arc<Self>);
     * }
     *
     * impl MyWaker for FutureWrapper {
     *     fn wake(self: &Arc<Self>) {
     *         // 利用自己的发送者，将自己重新发送到任务队列中
     *         let cloned = self.clone();
     *         self.task_sender.send(cloned).expect("任务队列已满")
     *     }
     * }
     *
     * impl Executor {
     *     fn run(&self) {
     *         // Executor 作为消息通道的接收者，可以从消息通道中取出需要被 poll 的 Future
     *         while let Ok(wrapper) = self.task_queue.recv() {
     *             let mut mutex_future = wrapper.future.lock().unwrap();
     *             if let Some(mut future) = mutex_future.take() {
     *                 if future.as_mut().poll(cx).is_pending() {
     *                     // Future 未完成时，将 Future 再次放入任务队列中
     *                     // wrapper.task_sender.send(wrapper.clone());
     *                     // MyWaker::wake(&wrapper)
     *                     wrapper.wake();
     *                 };
     * 
     *             }
     *         }
     *     }
     * }
     * ```
     *
     * 现在可以看成是 Waker 与 Executor 的交互可以使 Executor 不停的 poll Future。虽然 poll 包含 waker 的 Context 参数还未完全生成，但整体的触发和实现都体现了。
     * 生成完整的 Context 包含许多细节，这里利用 futures 提供的 ArcWaker 简化搭建简单可用的执行器这个过程。
     *
     * ```rust
     * struct FutureWrapper {
     *     future: Mutex<Option<BoxFuture<'static, ()>>>,
     *     task_sender: SyncSender<Arc<FutureWrapper>>,
     * }
     *
     * impl ArcWake for FutureWrapper {
     *     // arc_self 参数形式，FutureWrapper 的实例是无法直接调用的，需要参数为 self 才允许
     *     fn wake_by_ref(arc_self: &Arc<Self>) {
     *         let cloned = arc_self.clone();
     *         arc_self.task_sender.send(cloned).expect("任务队列已满")
     *     }
     * }
     *
     * #[derive(Clone)]
     * struct Spawner {
     *     task_sender: SyncSender<Arc<FutureWrapper>>, // 发送 FutureWrapper
     * }
     * impl Spawner {
     *     fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
     *         let future = future.boxed();
     *         let wrapper = FutureWrapper {
     *             future: Mutex::new(Some(future)),
     *             task_sender: self.task_sender.clone(),
     *         };
     *         // 将 Future 发送到任务通道中
     *         self.task_sender
     *             .send(Arc::new(wrapper))
     *             .expect("任务队列已满");
     *     }
     * }
     * ```
     *
     *
     *
     * TODO 为 FutureWrapper 实现 ArcWake 特征，最终实现 Context 和 wake 的调用
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
    enum FutureStatus {
        init,
        pending,
        completed,
    }
    struct SharedState {
        // 异步任务的状态
        status: FutureStatus,
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

            return match shared_state.status {
                FutureStatus::init => {
                    // 选择每次都`clone`的原因是： `TimerFuture`可以在执行器的不同任务间移动，如果只克隆一次，
                    // 那么获取到的`waker`可能已经被篡改并指向了其它任务，最终导致执行器运行了错误的任务
                    shared_state.waker = Some(cx.waker().clone());
                    shared_state.status = FutureStatus::pending;

                    let _shared_state = Arc::clone(&self.shared_state);
                    // 用线程休眠模拟异步任务
                    thread::spawn(move || {
                        thread::sleep(Duration::from_secs(6));
                        let mut mutex = _shared_state.lock().unwrap();
                        // 修改异步任务状态，模拟网络结束连接或IO关闭等场景。
                        // Future 一定要有一个表示执行异步任务状态的数据，这样才能让执行器在 Poll 当前 Future 时知道该结束 `Poll::Ready` 还是等待 `Poll::Pending`
                        mutex.status = FutureStatus::completed;

                        // 在异步任务结束后，调用 poll Future 的 waker
                        if let Some(waker) = mutex.waker.take() {
                            waker.wake()
                        }
                    });

                    std::task::Poll::Pending
                }
                FutureStatus::pending => std::task::Poll::Pending,
                FutureStatus::completed => {
                    println!("completed");
                    std::task::Poll::Ready(())
                }
            };
        }
    }
    // Future 生成
    impl TimeFuture {
        fn new() -> Self {
            let shared_state = Arc::new(Mutex::new(SharedState {
                status: FutureStatus::init,
                waker: None,
            }));

            Self { shared_state }
        }
    }

    futures::executor::block_on(TimeFuture::new());

    // TODO 继续完成消息通道管理的代码，统一 Executor 和 Spawner，以及解释为什么要从发送 Future 变为将 Future 包裹一层的 Task
    // 通过同步消息通道模拟 Executor 调度流程
    struct FutureChannel(Spawner, Executor);
    impl FutureChannel {
        fn new(size: usize) -> Self {
            let (tx, rx) = mpsc::sync_channel(size);
            Self(Spawner { task_sender: tx }, Executor { task_queue: rx })
        }
    }

    struct FutureWrapper {
        future: Mutex<Option<BoxFuture<'static, ()>>>,
        task_sender: SyncSender<Arc<FutureWrapper>>,
    }
    // 实现简单的 waker
    // trait MyWaker {
    //     fn wake(self: &Arc<Self>);
    // }
    // impl MyWaker for FutureWrapper {
    //     fn wake(self: &Arc<Self>) {
    //         // 利用自己的发送者，将自己重新发送到任务队列中
    //         let cloned = self.clone();
    //         self.task_sender.send(cloned).expect("任务队列已满")
    //     }
    // }

    impl ArcWake for FutureWrapper {
        // arc_self 参数形式，FutureWrapper 的实例是无法直接调用的，需要参数为 self 才允许
        fn wake_by_ref(arc_self: &Arc<Self>) {
            let cloned = arc_self.clone();
            arc_self.task_sender.send(cloned).expect("任务队列已满")
        }
    }

    #[derive(Clone)]
    struct Spawner {
        task_sender: SyncSender<Arc<FutureWrapper>>, // 发送 FutureWrapper
    }
    impl Spawner {
        fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
            let future = future.boxed();
            let wrapper = FutureWrapper {
                future: Mutex::new(Some(future)),
                task_sender: self.task_sender.clone(),
            };
            // 将 Future 发送到任务通道中
            self.task_sender
                .send(Arc::new(wrapper))
                .expect("任务队列已满");
        }
    }

    struct Executor {
        task_queue: Receiver<Arc<FutureWrapper>>,
    }
    impl Executor {
        fn run(&self) {
            // Executor 作为消息通道的接收者，可以从消息通道中取出需要被 poll 的 Future
            while let Ok(wrapper) = self.task_queue.recv() {
                let mut mutex_future = wrapper.future.lock().unwrap();

                if let Some(mut future) = mutex_future.take() {
                    // 生成关联的 waker
                    let waker = futures::task::waker_ref(&wrapper);
                    // 生成对应的 Context
                    let context = &mut Context::from_waker(&*waker);

                    // `BoxFuture<T>`是`Pin<Box<dyn Future<Output = T> + Send + 'static>>`的类型别名
                    // 通过调用`as_mut`方法，可以将上面的类型转换成`Pin<&mut dyn Future + Send + 'static>`
                    if future.as_mut().poll(context).is_pending() {
                        // Future 未完成时，将 Future 再次放入任务队列中
                        // wrapper.task_sender.send(wrapper.clone());
                    };
                }
            }
        }
    }
}
