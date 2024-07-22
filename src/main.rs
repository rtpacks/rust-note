use mini_redis::{client, Connection, Frame, Result};
use tokio::net::TcpListener;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    /*
     *
     * ## 实战：mini-redis - task
     * > https://github.com/tokio-rs/mini-redis
     *
     * mini-redis 是 tokio 编写的优秀 rust 实践案例，它的实现需要综合之前所学的各种知识。在这里，以 tokio 作为 Async Rust 的运行时，实现一版 mini-redis。
     *
     * ### 测试环境配置
     * 使用 tokio 提供的完成 mini-redis 作为项目的测试工具：
     * ```shell
     * cargo install mini-redis
     * ```
     *
     * 启动服务端和客户端：
     * ```shell
     * # 服务端
     * mini-redis-server
     *
     * # 客户端
     * mini-redis-cli set foo 1
     * mini-regis-cli get foo # "1"
     * ```
     *
     * 保持开启 mini-redis-server，接下来在 rust 代码中访问 mini-redis。
     *
     * ### 样例与原理解释
     *
     * 添加 cargo 依赖：
     * ```toml
     * tokio = { version = "1.38.0", features = ["full"] }
     * mini-redis = "0.4.1"
     * ```
     *
     * 样例代码：
     * ```rust
     * use mini_redis::{Result};
     *
     * #[tokio::main]
     * async fn main() -> Result<()> {
     *     let mut client = client::connect("127.0.0.1:6379").await?;
     *     client.set("foo", "Hello".into()).await?;
     *     println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
     *     println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
     *     Ok(())
     * }
     * ```
     *
     * 之前使用的 main 函数中很少会有属性标记，在使用 async-std 作为 async Web 服务器时曾解释，main 函数可以选择两种方式：
     * 1. 使用 `async_std::task::block_on` 函数
     * 2. 使用属性标记，将 main 函数改造成 async fn `#[async_std::main]`
     *
     * 使用 tokio 也是类似的，这里选择将 main 函数改造成 async fn。异步 main 函数有以下意义：
     * - .await 只能在 async 函数中使用，fn main 内部是无法直接使用 async 函数，这个会极大的限制使用场景
     * - 异步运行时本身需要初始化
     *
     * 因此 `#[tokio::main]` 宏在将 async fn main 隐式的转换为 fn main 的同时还对整个异步运行时进行了初始化：
     * ```rust
     * // 转换前：
     * #[tokio::main]
     * async fn main() {
     *     println!("hello");
     * }
     *
     * // 转换后：
     * fn main() {
     *     let mut rt = tokio::runtime::Runtime::new().unwrap();
     *     rt.block_on(async {
     *         println!("hello");
     *     })
     * }
     * ```
     *
     * > async fn 实际上返回的是一个实现了 Future 特征的**匿名类型**: `impl Future<Output = T>`。async/await 是 future 的语法糖，最终会被编译器编译成巨大的 Future 状态机。
     *
     *
     * 在 cargo 添加依赖中，有这么一行：
     * ```toml
     * tokio = { version = "1.38.0", features = ["full"] }
     * ```
     *
     * 其中的 `features = ["full"]` 表示开启所有的功能特性。
     *
     * 一个库有很多功能和特性，但不是每个应用都需要一个库所有的特性。为了优化编译时间和最终生成可执行文件大小、内存占用大小，应用可以对这些特性进行可选引入。
     * 这与前端工程中 `tree-sharking` 概念非常类似。
     *
     *
     * ### redis-server
     *
     * #### Tcp 连接
     * 作为服务器端，首先应该要接收外部进来的 TCP 连接，可以通过 tokio::net::TcpListener 来完成。
     * > Tokio 中大多数类型的名称都和标准库中对应的同步类型名称相同，而且如果没有特殊原因，Tokio 的 API 名称也和标准库保持一致，只不过用 async fn 取代 fn 来声明函数。
     *
     * 基于 Tcp 可以读取字节流，为了更简单的实现与客户端相互通信，以及支持 redis 的读写规则，这里可以使用 mini-redis 已经封装好的 Connection 结构体。
     * 它支持以一个一个数据帧frame(数据帧 = redis命令 + 数据)的读取，而不是更底层的字节流。
     *
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6379").await?;
     * let (stream, addr) = listener.accept().await?;
     *
     * // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
     * let mut connection = Connection::new(stream);
     * if let Some(frame) = connection.read_frame().await.unwrap() {
     *     println!("GOT: {}", frame);
     *
     *     // 先回复一个未实现服务的错误
     *     let response = Frame::Error("unimplemented".to_string());
     *     connection.write_frame(&response).await?;
     * }
     * ```
     *
     * 通过之前实现的客户端代码测试响应，将原有的客户端测试代码迁移到 `examples/redis-server-test.rs` 中：
     * ```rust
     * let mut client = client::connect("127.0.0.1:6379").await?;
     * client.set("foo", "Hello".into()).await?;
     * println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
     * println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
     * ```
     *
     * 然后修改 Cargo.toml：
     * ```toml
     * [[example]]
     * name = "redis-server-test"
     * path = "examples/redis-server-test.rs"
     * ```
     *
     * 最后测试，返回响应错误 `Error: "unimplemented"`：
     * ```shell
     * cargo run --example redis-server-test
     * ```
     *
     * 或者不通过客户端代码，直接通过 mini-redis-cli 测试响应，返回响应错误 `Error: "unimplemented"`：
     * ```shell
     * mini-redis-cli set foo 1
     * ```
     *
     * #### 生成任务
     * 只能接受和处理一条 TCP 连接，需要改造成并发/并行执行，改造时需要注意，不能在同一个任务中使用 `await`，否则会形成任务串行，无法并发的问题。
     *
     * 错误示例，无法形成并发，因为 loop 和 `connection.write_frame(frame).await` 在同一个任务中：
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6379").await?;
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *
     *     // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
     *     let mut connection = Connection::new(stream);
     *     if let Some(frame) = connection.read_frame().await.unwrap() {
     *         println!("GOT: {}", frame);
     *
     *         // 先回复一个未实现服务的错误
     *         let response = Frame::Error("unimplemented".to_string());
     *         connection.write_frame(&response).await?;
     *     }
     * }
     * ```
     *
     * 原因在于 loop 循环中的 await 会导致**当前任务**进入阻塞等待，也就是 loop 循环会被阻塞，只有等当前的处理完并结束后，才能开始接收下一条连接。
     *
     * 正确修改，使用 `tokio::task::spawn` 为每一条进来的连接都生成( spawn )一个新的任务, 然后在该任务中处理连接：
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6379").await?;
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *
     *     tokio::spawn(async move {
     *         // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
     *         let mut connection = Connection::new(stream);
     *         if let Some(frame) = connection.read_frame().await.unwrap() {
     *             println!("GOT: {}", frame);
     *
     *             // 先回复一个未实现服务的错误
     *             let response = Frame::Error("unimplemented".to_string());
     *             connection.write_frame(&response).await.unwrap();
     *         }
     *     });
     * }
     * ```
     *
     *
     * **任务**
     *
     * 一个 Tokio 任务是一个异步的**绿色线程**（又称协程、纤程），它们通过 tokio::spawn 函数进行创建，tokio::spawn 函数会返回一个 JoinHandle 类型的句柄，调用者可以使用该句柄与创建的任务进行交互。
     * tokio::spawn 函数的参数是一个 async 语句块，该语句块可以返回一个值，调用者可以通过 JoinHandle 句柄获取该值，通过 JoinHandle 获取的返回值是一个 `Result<T, Err>` 类型，
     * 表示若 spawn 创建的任务正常运行结束，则返回一个 Ok(T) 的值，否则会返回一个错误 Err，例如任务内部发生了 panic 或任务因为运行时关闭被强制取消时。
     *
     * **任务是调度器管理的执行单元**。spawn 生成的任务会首先提交给调度器，然后由它负责调度执行。
     * 需要注意的是，**执行任务的线程未必是创建任务的线程**，任务完全有可能运行在另一个不同的线程上，而且任务在生成后，它还可能会**在线程间被移动**。
     * **任务在 Tokio 中远比看上去要更轻量**，例如创建一个任务仅仅需要一次 64 字节大小的内存分配。因此应用程序在生成任务上，不需要有任何心理负担。
     *
     *
     * **'static 约束**
     *
     * 当使用 Tokio 创建一个任务时，该任务类型的生命周期必须是 'static。'static 是一种特殊的生命周期，表示引用在整个程序的生命周期内都是有效的。
     * 'static 生命周期意味着 Tokio 任务不能使用外部数据的引用，因为引用无法保证值活的像 'static 一样久。
     *
     * ```rust
     * let v = vec![1, 2, 3];
     *
     * task::spawn(async {
     *     println!("Here's a vec: {:?}", v);
     * });
     * ```
     *
     * 上面的代码会报错，因为在异步任务中使用了引用，这是因为默认情况下，变量并不是通过 move 的方式转移进 async 语句块的， v 变量的所有权依然属于 main 函数，因为任务内部的 println! 是通过借用的方式使用了 v，但是这种借用并不能满足 'static 生命周期的要求。
     * 如果希望数据在多个线程中使用，可以使用 `Arc`，`Arc` 可以轻松解决该问题，并且是线程安全的。
     *
     * ```rust
     * let v = Arc::new(vec![1, 2, 3]);
     * let _v = Arc::clone(&v);
     *
     * task::spawn(async move {
     *     println!("Here's a vec: {:?}", _v);
     * });
     * ```
     *
     * 此外，有些时候会有报错：function requires argument type to outlive `'static`，函数要求参数类型的生命周期必须比 'static 长。
     * 其实，没有任何类型会比 'static 活得更长，'static 是最长的生命周期，这里意味着要求引用会在程序的整个生命周期内有效，用 'static 约束即可。
     *
     *
     * **Send 约束**
     *
     * 上面提到过：**执行某个任务的线程未必是创建该任务的线程**，任务完全有可能运行在另一个不同的线程上。在任务生成后，它还可能会**在线程间被移动**。
     * 即当这些任务在 .await 过程发生阻塞时，Tokio 调度器会调度任务在线程间移动。这意味着 tokio::spawn 生成的任务必须实现 Send 特征，用以保证线程安全。
     *
     * 一个任务要实现 Send 特征，那它**在 .await 调用的过程中所持有的全部数据都必须实现 Send 特征**。
     * 当 .await 调用发生阻塞时，任务会让出当前线程所有权给调度器，然后当任务准备好后，调度器会从上一次暂停的位置继续执行该任务。
     * 该流程能正确的工作，任务必须将.await之后使用的所有状态保存起来，这样才能在中断后恢复现场并继续执行。
     * 若这些状态实现了 Send 特征(可以在线程间安全地移动)，那任务自然也就可以在线程间安全地移动。
     *
     * 简单的理解，.await 时刻需要保存任务的状态，并且 .await 时刻是可能发生线程切换的，所以如果变量的作用域横跨 .await 时刻，该变量就需要实现 Send 特征。
     *
     * 变量的作用域不横跨 .await 时刻，.await 无需保存该变量状态，也就不会涉及到线程安全问题，所以使用 `!Send` 特征也不会报错：
     * ```rust
     * tokio::spawn(async {
     *     // 语句块的使用强制了 `rc` 会在 `.await` 被调用前就被释放，因此 `rc` 并不会影响 `.await`的安全性
     *     {
     *         let rc = Rc::new("hello");
     *         println!("{}", rc);
     *     }
     *
     *     // `rc` 的作用范围已经失效，因此当任务让出所有权给当前线程时，它无需作为状态被保存起来
     *     yield_now().await;
     * });
     * ```
     *
     * 变量的作用域横跨 .await 时刻，.await 需要保存变量的状态，.await 可能涉及到线程切换，所以使用 `!Send` 特征报错，需要使用 `Send` 特征：
     * ```rust
     * tokio::spawn(async {
     *     // `Rc` 是 `!Send`，属于非线程安全的
     *     let rc = Rc::new("hello");
     *     println!("{}", rc);
     *
     *     // `rc` 的作用范围此时还未结束，与 async 语句块的结束位置一致，因此当任务让出所有权给当前线程时，它需要作为状态被保存起来
     *     yield_now().await;
     * });
     * ```
     *
     * **.await 时刻需要保存任务的状态，并且 .await 时刻是可能发生线程切换的，所以如果变量的作用域横跨 .await 时刻，该变量就需要实现 Send 特征。**
     *
     */

    // 客户端
    // {
    // let mut client = client::connect("127.0.0.1:6379").await?;
    // client.set("foo", "Hello".into()).await?;
    // println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
    // println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
    // }

    // Connection 结构体
    // {
    //     let listener = TcpListener::bind("127.0.0.1:6379").await?;
    //     let (stream, addr) = listener.accept().await?;

    //     // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
    //     let mut connection = Connection::new(stream);
    //     if let Some(frame) = connection.read_frame().await.unwrap() {
    //         println!("GOT: {}", frame);

    //         // 先回复一个未实现服务的错误
    //         let response = Frame::Error("unimplemented".to_string());
    //         connection.write_frame(&response).await?;
    //     }
    // }

    {
        let listener = TcpListener::bind("127.0.0.1:6379").await?;

        // loop {
        //     let (stream, addr) = listener.accept().await?;

        //     // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
        //     let mut connection = Connection::new(stream);
        //     if let Some(frame) = connection.read_frame().await.unwrap() {
        //         println!("GOT: {}", frame);

        //         // 先回复一个未实现服务的错误
        //         let response = Frame::Error("unimplemented".to_string());
        //         connection.write_frame(&response).await?;
        //     }
        // }
        loop {
            let (stream, addr) = listener.accept().await?;

            let v = vec![1, 2, 3];

            tokio::spawn(async move {
                // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
                let mut connection = Connection::new(stream);
                if let Some(frame) = connection.read_frame().await.unwrap() {
                    println!("v = {:?}", &v);
                    println!("GOT: {}", frame);

                    // 先回复一个未实现服务的错误
                    let response = Frame::Error("unimplemented".to_string());
                    connection.write_frame(&response).await.unwrap();
                }
            });
        }
    }

    Ok(())
}
