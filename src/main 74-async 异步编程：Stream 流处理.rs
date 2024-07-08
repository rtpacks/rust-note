use futures::{channel::mpsc, SinkExt, StreamExt};

fn main() {
    /*
     *
     * ## async 异步编程：Stream 流处理
     * async/.await 是 Rust 语法的一部分，它在遇到阻塞操作时( 例如 IO )会让出当前线程的控制权而不是阻塞当前线程，这样就允许当前线程继续去执行其它代码，最终实现并发。
     *
     * 有两种方式可以使用 async： `async fn() {}` 用于声明函数，`async { ... }` 用于声明语句块，它们会返回一个实现 Future 特征的值:
     *
     * ```rust
     * async fn foo() -> i32 {
     *     5
     * }
     * let bar = async {
     *     let value = foo().await;
     *     println!("{}", value + 3);
     *     value + 3
     * };
     * futures::executor::block_on(bar);
     * ```
     *
     * async 是懒惰的，直到被执行器 poll 或者 .await 后才会开始运行，其中后者是最常用的运行 Future 的方法。
     * 当 .await 被调用时，它会尝试运行 Future 直到完成，但是若该 Future 进入阻塞，那就会让出当前线程的控制权。
     * 当 Future 后面准备再一次被运行时(例如从 socket 中读取到了数据)，执行器会得到通知，并再次运行该 Future ，如此循环，直到完成。
     *
     * 以上过程只是一个简述，详细内容在底层探秘中已经被深入讲解过，因此这里不再赘述。
     *
     * 注意：一个函数如果用 async 标识，但是 async 里面没有 poll Future 的流程，包括 await，那么 async 没有任何意义。只不过 async 这个整体作为一个 future，但是在 async 内部运行时是不会产生任何调度的。
     *
     * ### async 的生命周期
     * async fn 函数返回的 Future 的生命周期会受到 Future 所使用变量的生命周期的限制，如使用引用类型的变量。
     *
     * 分析 async fn 的生命周期，与等价的函数进行对比：
     * ```rust
     * async fn foo(x: &u8) -> u8 { *x }
     *
     * // 上面的函数跟下面的函数是等价的
     * fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
     *     async move { *x }
     * }
     * ```
     * 上面的生命周期表明：如果要 Future 运行正常( .await )，就必须保证 x 在 Future 运行时保持正常，即说 x 必须比 Future 活得更久。
     * 所以 Future 的生命周期受限于所使用变量的生命周期，必须保证所使用变量的声明周期包含 Future 的生命周期，Future 才能正常运行。
     *
     * 在一般情况下，在函数调用后就立即 .await 不会存在任何问题，例如foo(&x).await。
     * 但是如果 Future 被先存起来或发送到另一个任务或者线程，就可能存在问题了。
     *
     * 错误示例，以下代码会报错，因为 x 的生命周期只到 bad 函数的结尾。 但是 Future 显然会活得更久：
     * ```rust
     * async fn borrow_x(x: &u8) -> u8 { *x }
     *
     * fn bad() -> impl Future<Output = u8> {
     *     let x = 5;
     *     borrow_x(&x) // ERROR: `x` does not live long enough
     * }
     * ```
     *
     * 这种场景最常用的解决办法是返回一个新的 Future，将新的 Future 的生命周期变为 `'static`。将原有 Future 的调用与原有 Future 使用的变量用新 Future 包裹起来：
     *
     * ```rust
     * // fn bad() -> impl futures::Future<Output = u8> {
     * //     let x = 5;
     * //     borrow_x(&x) // ERROR: `x` does not live long enough
     * // }
     * fn good() -> impl futures::Future<Output = u8> {
     *     // 错误
     *     //     let x = 5;
     *     //     borrow_x(&x) // ERROR: `x` does not live long enough
     *
     *     // 正确
     *     async {
     *         let x = &5;
     *         borrow_x(x).await
     *     }
     * }
     * // async 写法
     * async fn good_async() -> u8 {
     *     let x = &5;
     *     borrow_x(x).await
     * }
     * ```
     *
     * 以上就是使用 async 和 `fn() -> impl Future<Output = T>` 两种语法的区别，新的 Future 正好与标注返回的 Future 类型保持了一致。。
     *
     * ### async move
     * async 允许像闭包那样使用 move 关键字来将环境中变量的**所有权转移**到语句块内，好处是不需要考虑如何解决借用生命周期的问题，坏处是无法与其它代码实现对变量的共享。
     *
     * ```rust
     * // 多个不同的 `async` 语句块可以访问同一个本地变量，只要它们在该变量的作用域内执行
     * async fn blocks() {
     *     let string = String::from("Hello World");
     *     let future1 = async {
     *         println!("{string}");
     *     };
     *     let future2 = async {
     *         println!("{string}");
     *     };
     *     // 运行两个 Future 直到完成
     *     futures::join!(future1, future2);
     * }
     * futures::executor::block_on(blocks());
     *
     *
     * // 由于 `async move` 会捕获环境中的变量，因此只有一个 `async move` 语句块可以访问该变量，
     * // 但是它也有非常明显的好处：变量可以转移到返回的 Future 中，不再受借用生命周期的限制
     * fn move_block() -> impl futures::Future<Output = ()> {
     *     let my_string = "foo".to_string();
     *     async move {
     *         // ...
     *         println!("{my_string}");
     *     }
     * }
     * futures::executor::block_on(move_block());
     * ```
     *
     * ### await 和多线程执行器
     * Future 并不只是在一个线程上运行的，当使用多线程 Future 执行器( executor )时，Future 内部的任何 .await 都可能导致它被切换到一个新线程上去执行。
     * 因此 Future 可能会在线程间被移动，async 语句块中的变量必须要能在线程间传递。
     *
     * 由于需要在多线程环境使用，意味着 Rc、 RefCell、没有实现 Send 的所有权类型、没有实现 Sync 的引用类型，它们在多线程环境中都是不安全的，因此无法被使用。
     * 当然，实际上它们还是有可能被使用的，只要在 .await 调用期间，它们没有在 await 调用的作用域范围内，不在作用域内就不会在线程池中移动，也就不会存在被多个线程访问导致数据竞争和其他线程安全问题。
     *
     * 类似的原因，在 .await 时使用普通的锁也不安全，例如 Mutex。原因是它可能会导致线程池被锁：
     * 当一个任务获取锁 A 后，若它将线程的控制权还给执行器，然后执行器又调度运行另一个任务，该任务也去尝试获取了锁 A ，结果当前线程会直接卡死，最终陷入死锁中。
     * 因此，为了避免这种情况的发生，我们需要使用 futures 包下的锁 futures::lock 来替代 Mutex 完成任务。
     *
     *
     * ### Stream 流处理
     * Stream 特征类似于 Future 特征，但是前者在完成前可以生成多个值，这种行为跟标准库中的 Iterator 特征非常相似。
     * 也就是 Stream 在一次被 poll 的过程中，并不是只有 Ready 一种的结束状态，而是 Ready(Some(v)) 和 Ready(None) 形式的多种状态。当然 Pending 还是一种。
     *
     * 参考阅读：https://juejin.cn/post/7217487697677156407
     *
     * ```rust
     * trait Stream {
     *     // Stream 生成的值的类型
     *     type Item;
     *
     *     // 尝试去解析 Stream 中的下一个值,
     *     // 若无数据，返回`Poll::Pending`, 若有数据，返回 `Poll::Ready(Some(x))`, `Stream`完成则返回 `Poll::Ready(None)`
     *     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
     *         -> Poll<Option<Self::Item>>;
     * }
     * ```
     *
     * Stream 的一个常见例子是消息通道（ futures 包中的）的消费者 Receiver。
     * 每次有消息从 Send 端发送后，它都可以接收到一个 Some(val) 值，一旦 Send 端关闭( drop )，且消息通道中没有消息后，它会接收到一个 None 值。
     * 不得不说，rust 这块的设计的非常灵活且巧妙。
     *
     * ```rust
     * async fn send_recv() {
     *     let (mut tx, mut rx) = mpsc::channel(100);
     *
     *     let result = tx.send(1).await;
     *     let result = tx.send(2).await;
     *
     *     // 清除发送者，避免无法释放
     *     drop(tx);
     *
     *     // `StreamExt::next` 类似于 `Iterator::next`, 但是前者返回的不是值，而是一个 `Future<Output = Option<T>>`，
     *     // 因此还需要使用`.await`来获取具体的值
     *     let option = rx.next().await;
     *     println!("{}", option.unwrap());
     *     let option = rx.next().await;
     *     println!("{}", option.unwrap());
     * }
     *
     * futures::executor::block_on(send_recv());
     * ```
     *
     * Iterator 和 Stream 的区别：
     * - Iterator 可以不断调用 next 方法，获得新的值，直到返回 None。Iterator 是**阻塞式**返回数据的，每次调用 next，必然独占 CPU 直到得到一个结果，而异步的 Stream 是**非阻塞**的，在等待的过程中会让出当前线程的控制权，空出 CPU 做其他事情。
     * - Stream 的 poll_next 方法与 Future 的 poll 方法很像，并且和 Iterator 的 next 的作用类似。在使用中，poll_next 调用起来不方便，需要自己处理 Poll 状态，所以 Rust 提供了 StreamExt，作为 Stream 的扩展，提供了 next 方法，返回一个实现了 Future trait 的 Next 结构体，这样就可以直接通过 stream.next().await 来迭代一个 stream。
     *
     * > 注：StreamExt 是 StreamExtension 的简写。在 Rust 中，通常的做法是只在一个文件中放入最小定义（比如 Stream），且在另一个扩展的相关文件中放入额外的 api（比如 StreamExt）。
     *
     * > 注：Stream trait 还没有像 future 一样在 Rust 的核心库(std::core)中，它在 future_utils crate 中，而 StreamExtensions 也不在标准库中。这意味着，由于不同的库提供不同的导入，你可能会得到冲突的导入。例如，tokio 提供不同的 StreamExt 与 futures_utils。如果可以的话，尽量使用 futures_utils，因为它是 async/await 最常用的 crate.
     *
     *
     * #### 迭代和并发
     * 与迭代器类似，可以对一个 Stream 迭代。例如使用 map，filter，fold 方法，以及它们的遇到错误提前返回的版本： try_map，try_filter，try_fold。
     * 但是跟迭代器又有所不同，for 循环无法迭代 Stream，但命令式风格的循环 while let 可以使用，同时还可以使用 next 和 try_next 方法:
     *
     * ```rust
     * async fn stream_next() {
     *     use futures::prelude::*;
     *     let mut st = stream::iter(1..4)
     *         .filter(|x| future::ready(x % 2 == 0))
     *         .map(|x| x * x);
     *
     *     // 迭代
     *     while let Some(x) = st.next().await {
     *         println!("Got item: {}", x);
     *     }
     * }
     * futures::executor::block_on(stream_next());
     * ```
     *
     * 如果希望并发，可以使用 for_each_concurrrent 或者 try_for_each_concurrent 方法：
     * ```rust
     * async fn stream_next_concurrent() {
     *     use futures::prelude::*;
     *     let mut stream = stream::iter(1..10).filter(|x| future::ready(x % 2 == 0));
     *
     *     async fn report_n_jumps(num: i32) -> Result<(), std::io::Error> {
     *         println!("report_n_jumps : {}", num);
     *         Ok(())
     *     }
     *
     *     // 迭代
     *     stream
     *         .for_each_concurrent(2, |x| async move {
     *             println!("stream_next_concurrent: {}", x);
     *             report_n_jumps(x).await;
     *         })
     *         .await;
     * }
     * futures::executor::block_on(stream_next_concurrent());
     * ```
     *
     * ### 更多阅读
     * - https://juejin.cn/post/7217487697677156407
     */

    // 目前处于稳定版本的 async fn 和 async {}
    async fn foo() -> i32 {
        5
    }
    let bar = async {
        let value = foo().await;
        println!("{}", value + 3);
        value + 3
    };
    futures::executor::block_on(bar);

    async fn borrow_x(x: &u8) -> u8 {
        *x
    }

    // fn bad() -> impl futures::Future<Output = u8> {
    //     let x = 5;
    //     borrow_x(&x) // ERROR: `x` does not live long enough
    // }
    fn good() -> impl futures::Future<Output = u8> {
        // 错误
        //     let x = 5;
        //     borrow_x(&x) // ERROR: `x` does not live long enough

        // 正确
        async {
            let x = &5;
            borrow_x(x).await
        }
    }
    // async 写法
    async fn good_async() -> u8 {
        let x = &5;
        borrow_x(x).await
    }

    // 多个不同的 `async` 语句块可以访问同一个本地变量，只要它们在该变量的作用域内执行
    async fn blocks() {
        let string = String::from("Hello World");
        let future1 = async {
            println!("{string}");
        };
        let future2 = async {
            println!("{string}");
        };

        // 运行两个 Future 直到完成
        futures::join!(future1, future2);
    }
    futures::executor::block_on(blocks());

    // 由于 `async move` 会捕获环境中的变量，因此只有一个 `async move` 语句块可以访问该变量，
    // 但是它也有非常明显的好处：变量可以转移到返回的 Future 中，不再受借用生命周期的限制
    fn move_block() -> impl futures::Future<Output = ()> {
        let my_string = "foo".to_string();
        async move {
            // ...
            println!("{my_string}");
        }
    }
    futures::executor::block_on(move_block());

    async fn send_recv() {
        let (mut tx, mut rx) = mpsc::channel(100);
        let result = tx.send(1).await;
        let result = tx.send(2).await;

        // 清除发送者，避免无法释放
        drop(tx);

        // `StreamExt::next` 类似于 `Iterator::next`, 但是前者返回的不是值，而是一个 `Future<Output = Option<T>>`，
        // 因此还需要使用`.await`来获取具体的值
        let option = rx.next().await;
        println!("{}", option.unwrap());
        let option = rx.next().await;
        println!("{}", option.unwrap());
    }
    futures::executor::block_on(send_recv());

    async fn send_recv_next() {
        use futures::stream::StreamExt; // 单独使用 Stream 时，需要引入 next
        let (mut tx, mut rx) = mpsc::channel(100);
        let result = tx.send(1).await;
        let result = tx.send(2).await;

        // 清除发送者，避免无法释放
        drop(tx);

        // `StreamExt::next` 类似于 `Iterator::next`, 但是前者返回的不是值，而是一个 `Future<Output = Option<T>>`，
        // 因此还需要使用`.await`来获取具体的值
        while let Some(item) = rx.next().await {
            println!("send_recv_next {}", item);
        }
    }
    futures::executor::block_on(send_recv_next());

    async fn send_recv_try_next() {
        use futures::stream::TryStreamExt; // 单独使用 Stream 时，需要引入 try_next
        let (mut tx, mut rx) = mpsc::channel(100);

        let result = tx.send(1).await;
        let result = tx.send(2).await;

        // 清除发送者，避免无法释放
        drop(tx);

        // `StreamExt::next` 类似于 `Iterator::next`, 但是前者返回的不是值，而是一个 `Future<Output = Option<T>>`，
        // 因此还需要使用`.await`来获取具体的值
        while let Some(item) = rx.try_next().unwrap() {
            println!("send_recv_try_next {}", item);
        }
    }
    futures::executor::block_on(send_recv_try_next());

    async fn stream_next() {
        use futures::prelude::*;
        let mut st = stream::iter(1..4)
            .filter(|x| future::ready(x % 2 == 0))
            .map(|x| x * x);

        // 迭代
        while let Some(x) = st.next().await {
            println!("Got item: {}", x);
        }
    }
    futures::executor::block_on(stream_next());

    async fn stream_next_concurrent() {
        use futures::prelude::*;
        let mut stream = stream::iter(1..10).filter(|x| future::ready(x % 2 == 0));

        async fn report_n_jumps(num: i32) -> Result<(), std::io::Error> {
            println!("report_n_jumps : {}", num);
            Ok(())
        }

        // 迭代
        stream
            .for_each_concurrent(2, |x| async move {
                println!("stream_next_concurrent: {}", x);
                report_n_jumps(x).await;
            })
            .await;
    }
    futures::executor::block_on(stream_next_concurrent());
}
