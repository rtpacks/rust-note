use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

use bytes::Bytes;
use mini_redis::{
    Command::{self, Get, Set},
    Connection, Frame, Result,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    /*
     *
     * ## 实战：mini-redis - state
     * > https://github.com/tokio-rs/mini-redis
     *
     * 将处理任务的逻辑抽离到单独的函数中，便于后续解耦开发：
     * ```rust
     * async fn process(stream: TcpStream) {
     *     // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
     *     let mut connection = Connection::new(stream);
     *     if let Some(frame) = connection.read_frame().await.unwrap() {
     *         println!("GOT: {}", frame);
     *
     *         // 先回复一个未实现服务的错误
     *         let response = Frame::Error("unimplemented".to_string());
     *         connection.write_frame(&response).await.unwrap();
     *     }
     * }
     *
     * let listener = TcpListener::bind("127.0.0.1:6379").await?;
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move {
     *         process(stream).await;
     *     });
     * }
     * ```
     *
     * ### HashMap 存储数据
     * 客户端通过 SET 命令存值，通过 GET 命令来取值，这些相应的值将被存储在 HashMap 结构体中。
     * 与 Connection 一样，为了更简单的实现与客户端相互通信，以及支持 redis 的读写规则，这里可以使用 mini-redis 已经封装好的 Command、Get、Set 结构体。
     *
     * 其中有几个关键点
     * 1. 原有代码的连接逻辑只处理一个帧结构数据，事实上 redis 可以利用一个连接做多个操作，也就是一个连接中，会有多个帧结构数据（redis 命令+数据）
     * 2. key 被存储为 String，value 被存储为 Vec<u8>，也就是字节列表
     *
     * HashMap 的具体类型会由 db.insert 推断，所以不需要手动标注。
     *
     * > https://github.com/sunface/rust-course/discussions/888#discussioncomment-5538867
     * >
     * > HashMap 通过 match 分支中的 db.insert 进行推断类型，这个是有一定限制的。
     * > process 函数中推断 db 泛型参数与 match 分支顺序有关系，在这里只有 Set 分支能推断出完整的 `HashMap<String, Vec<u8>>` 类型。
     * >
     * > 当 Set 作为第一分支时可以推断 db 泛型，但如果先遇到 Get 分支，那就会推断不全。
     * > 语义上 match 各分支是没有先后的，类型推断也应当尽力而为，感觉这是语言的一个缺陷。
     * > https://github.com/rust-lang/rust/issues/25165
     *
     * ```rust
     * async fn process(stream: TcpStream) {
     *     // `mini-redis` 提供的便利函数，使用返回的 `connection` 可以用于从 socket 中读取数据并解析为数据帧
     *     // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
     *     let mut connection = Connection::new(stream);
     *
     *     // 生成存储数据的HashMap
     *     let mut db = HashMap::new();
     *
     *     // 在一个连接中可以传送多个帧数据，因此需要使用 while let 而不是 if let
     *     while let Some(frame) = connection.read_frame().await.unwrap() {
     *         println!("GOT: {}", frame);
     *
     *         let response = match Command::from_frame(frame).unwrap() {
     *             Set(cmd) => {
     *                 // 值被存储为 `Vec<u8>` 的形式
     *                 db.insert(cmd.key().to_string(), cmd.value().to_vec());
     *                 Frame::Simple("OK".to_string())
     *             }
     *             Get(cmd) => {
     *                 // `Frame::Bulk` 期待数据的类型是 `Bytes`，`&Vec<u8>` 可以使用 `into()` 方法转换成 `Bytes` 类型
     *                 if let Some(value) = db.get(cmd.key()) {
     *                     Frame::Bulk(value.clone().into())
     *                 } else {
     *                     Frame::Null
     *                 }
     *             }
     *             cmd => panic!("unimpement {:?}", cmd),
     *         };
     *
     *         connection.write_frame(&response).await.unwrap();
     *     }
     * }
     * ```
     *
     * 使用 cargo run 运行服务器，然后再打开另一个终端窗口，运行 redis-server-test 客户端:
     * ```shell
     * cargo run --example redis-server-test
     * ```
     *
     * 服务可以正常响应，但是根据代码逻辑，每个连接都会生成一个 db，不同的连接没有共享存储的状态，很明显这是有问题的。
     *
     * ### 存储类型与共享状态
     *
     * **存储类型**
     *
     * 在数据共享状态过程中，对数据的操作是不可避免的。由于 rust 的所有权和借用规则的限制，如果直接存储 `Vec<u8>` 类型，那么在未来操作时可能会处理非常复杂的生命周期和借用关系，比如使用原始数据的切片需要手动保证原始数据的生命周期足够长。
     * 虽然使用 `Vec<u8>` 保存目标数据非常便捷，但为了避免处理复杂的生命周期和借用关系以及提高效率，这里将 `bytes` 库提供的 `Bytes` 类型作为存储类型，以避免这些问题。
     * Bytes 是一个引用计数类型，它基于 Arc 实现并提供了一些额外的能力。
     *
     * 1. 引用计数和共享内存
     * Bytes 使用引用计数来管理数据的所有权。这意味着多个 Bytes 实例可以共享同一块内存，而不会产生多余的拷贝。相反，如果使用 `Vec<u8>`，即使只传递一个切片，也需要确保原始数据的生命周期足够长，或者进行显式的内存管理。
     *
     * 2. 零拷贝 (Zero-Copy)
     * Bytes 的设计目标之一是支持零拷贝操作。在网络编程中，经常需要从网络读取数据到缓冲区，再处理这些数据。如果使用 `Vec<u8>`，每次处理时都需要拷贝数据，导致额外的开销。Bytes 可以在不拷贝数据的情况下处理这些数据，提高性能。
     *
     * 3. 高效的切片操作
     * Bytes 提供了一种更高效的切片操作方式。Bytes 的切片操作不会重新分配内存，而是创建一个新的 Bytes 实例，共享底层数据。这种方式避免了不必要的内存分配和拷贝。
     *
     * 4. 更好的 API 支持
     * Bytes 提供了很多便捷的方法和 API，用于处理二进制数据。这些方法和 API 使得处理数据更加方便和直观。
     *
     * Cargo.toml 添加 bytes 依赖：
     * ```toml
     * bytes = "1.6.1"
     * ```
     *
     *
     * **共享状态**
     *
     * 由于 HashMap 许哟啊在多线程中访问，因此需要考虑多线程数据安全，即 HashMap 需要使用 Arc + Mutex 守护：
     * ```rust
     * use bytes::Bytes;
     * use std::collections::HashMap;
     * use std::sync::{Arc, Mutex};
     *
     * type DB = Arc<Mutex<HashMap<String, Bytes>>>;
     * ```
     * 直接编写存储类型让代码的可读性变得非常低，因此这里使用了类型别名 `type DB` 简化存储类型 `Arc<Mutex<String, Bytes>>`。
     *
     * 为了在多线程中共享 DB，DB 不应该在每个 Tcp 连接中初始化，而是应该在程序主线程初始化时初始化DB，然后将 DB 作为参数传递给 Tcp 的处理函数 process：
     * ```rust
     * type DB = Arc<Mutex<HashMap<String, Bytes>>>;
     *
     * let listener = TcpListener::bind("127.0.0.1:6379").await?;
     * let db: DB = Arc::new(Mutex::new(HashMap::new()));
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     let _db = Arc::clone(&db);
     *     tokio::spawn(async move {
     *         process(stream, _db).await;
     *     });
     * }
     * ```
     *
     * 更新 process 函数签名和逻辑：
     * ```rust
     * async fn process(stream: TcpStream, db: DB) {
     *     // `mini-redis` 提供的便利函数，使用返回的 `connection` 可以用于从 socket 中读取数据并解析为数据帧
     *     // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
     *     let mut connection = Connection::new(stream);
     *
     *     // 在一个连接中可以传送多个帧数据，因此需要使用 while let 而不是 if let
     *     while let Some(frame) = connection.read_frame().await.unwrap() {
     *         println!("GOT: {}", frame);
     *
     *         let response = match Command::from_frame(frame).unwrap() {
     *             Set(cmd) => {
     *                 // 值被存储为 `Vec<u8>` 的形式
     *                 db.lock()
     *                     .unwrap()
     *                     .insert(cmd.key().to_string(), cmd.value().clone());
     *                 Frame::Simple("OK".to_string())
     *             }
     *             Get(cmd) => {
     *                 // `Frame::Bulk` 期待数据的类型是 `Bytes`，`&Vec<u8>` 可以使用 `into()` 方法转换成 `Bytes` 类型
     *                 if let Some(value) = db.lock().unwrap().get(cmd.key()) {
     *                     Frame::Bulk(value.clone().into())
     *                 } else {
     *                     Frame::Null
     *                 }
     *             }
     *             cmd => panic!("unimpement {:?}", cmd),
     *         };
     *
     *         connection.write_frame(&response).await.unwrap();
     *     }
     * }
     * ```
     *
     * 以上服务端的状态存储和共享就基本实现了，但是有一些重点需要梳理。
     *
     *
     * ### 锁与 .await
     * **std::sync::Mutex**
     *
     * 上面的代码中，DB 有一个非常重要的设计，Mutex 是标准库中的互斥锁 `std::sync::Mutex`，并不是 tokio 库中的互斥锁 `tokio::sync::Mutex`。
     * 在绝大多数场景下，标准库中的互斥锁是满足使用需求的，只有在异步代码中锁需要跨越多个 .await 调用时，才需要考虑 tokio 提供的互斥锁 `tokio::sync::Mutex`。
     *
     * 这是因为标准库的 Mutex 在异步代码中跨越 .await 调用时可能会导致死锁。
     * 例如某个任务刚获取完锁，还没使用完释放就因为 .await 让出了当前线程的执行权，如果下个任务去获取锁，由于锁在上一个任务中未释放，获取锁就会导致当前线程阻塞，最后造成死锁。
     *
     * > 很多时候死锁是线程阻塞造成的，简单理解死锁是多个执行单元相互阻塞的表现，比如多个线程相互阻塞，多个任务相互阻塞。
     *
     * Tokio 的 Mutex 实际上是基于标准库的 Mutex 实现的，但它提供了额外的机制来避免在跨线程间转移所有权时的死锁问题。
     *
     * 在异步代码中，关于锁的使用有以下经验之谈：
     * - 锁如果在多个 .await 过程中持有，应该使用 Tokio 提供的锁，因为 .await 过程中锁可能在线程间转移，`std::sync::Mutex` 可能会造成死锁。其他情况 `std::sync::Mutex` 一般都满足场景需求。
     * - 锁竞争不多的情况下，使用 std::sync::Mutex
     * - 锁竞争多，可以考虑使用第三方库提供的性能更高的锁，例如 parking_lot::Mutex
     *
     *
     * 锁 Mutex 没有实现 Send 特征，如果锁的作用域横跨了 .await 时刻，那么编译器就会报错：
     * ```rust
     * async fn increment_and_do_stuff() {
     *     async fn do_something_async() {}
     *
     *     let mutex = Mutex::new(2);
     *     let mut lock = mutex.lock().unwrap();
     *     *lock += 1;
     *
     *     do_something_async().await;
     *     // Mutex 横跨了 .await，作用域横跨 .await 的变量要求实现 Send 特征，以保证线程安全
     *
     * } // 锁在这里结束作用域，释放
     *
     * tokio::spawn(increment_and_do_stuff()); // 报错，Mutex 没有实现 Send 特征
     * ```
     *
     * 在 【unit 83-实战：mini-redis - task】 中总结过 .await 跨线程的认识：
     * 简单的理解，.await 时刻当前任务会让出当前线程的执行权，.await 需要保存当前任务的状态，等待下次执行使用。
     * 同时，当该任务 .await 后下一次被执行时，由于执行器的调度，当前线程可能正在执行其他任务，那么该任务就可能被分配到另外一个线程去执行，
     * 即 .await 前后，任务可能会发生线程切换，所以如果变量的作用域横跨 .await 时刻，该变量就需要实现 Send 特征，这样才能保证多线程变量安全。
     *
     * > 根据上面的分析，标准库的互斥锁 `std::sync::Mutex` 是不可以实现 Send 特征的，因为如果一个任务获取了锁，然后还没释放就在 .await 期间被挂起，接着开始执行另一个任务，这个任务又去获取锁，就会导致死锁。
     *
     * 解决方法很简单，让锁不横跨 .await 时刻就可以了：
     * ```rust
     * async fn increment_and_do_stuff() {
     *     async fn do_something_async() {}
     *
     *     {
     *         let mutex = Mutex::new(2);
     *         let mut lock = mutex.lock().unwrap();
     *         *lock += 1;
     *     } // 锁在这里结束作用域，释放
     *
     *     do_something_async().await;
     * }
     *
     * tokio::spawn(increment_and_do_stuff());
     * ```
     *
     * 有一点需要注意，这里是指变量的作用域横跨 .await 时刻，并不是变量的有效期：
     * ```rust
     * async fn increment_and_do_stuff() {
     *     async fn do_something_async() {}
     *
     *     let mutex = Mutex::new(2);
     *     let mut lock = mutex.lock().unwrap();
     *     *lock += 1;
     *      drop(lock);
     *      drop(mutex);
     *
     *     do_something_async().await;
     *     // Mutex 横跨了 .await，作用域横跨 .await 的变量要求实现 Send 特征，以保证线程安全
     *
     * } // 锁在这里结束作用域，释放
     *
     * tokio::spawn(increment_and_do_stuff());
     * ```
     * 编译器在这里不够聪明，目前它只能根据作用域的范围来判断，drop 虽然释放了锁，但是锁的作用域依然会持续到函数的结束。
     *
     *
     * 在实际使用中，不会直接使用语句块限制锁的作用域，而是会将锁放在结构体的同步方法中，让代码的内聚性更高：
     * ```rust
     * use std::sync::Mutex;
     *
     * struct CanIncrement {
     *     mutex: Mutex<i32>,
     * }
     * impl CanIncrement {
     *     // 该方法不是 `async`
     *     fn increment(&self) {
     *         let mut lock = self.mutex.lock().unwrap();
     *         *lock += 1;
     *     }
     * }
     *
     * async fn increment_and_do_stuff(can_incr: &CanIncrement) {
     *     can_incr.increment();
     *     do_something_async().await;
     * }
     * ```
     *
     *
     * **tokio::sync::Mutex**
     *
     * Tokio 提供的锁最大的优点就是：它可以在 .await 执行期间被持有，而且不会有任何问题。但是代价就是，这种异步锁的性能开销会更高，因此如果可以，使用之前的两种方法来解决会更好。
     * ```rust
     * async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
     *     let mut lock = mutex.lock().await;
     *     *lock += 1;
     *
     *     do_something_async().await;
     * } // 锁在这里被释放
     * ```
     *
     * ### 任务、线程和锁竞争
     * 锁的竞争非常激烈并且很容易导致死锁，将锁分片的难度非常高，因此选择多线程轮询方式并不理想。
     * 阅读：https://course.rs/advance-practice/shared-state.html#任务线程和锁竞争
     *
     */

    // 客户端
    // {
    // let mut client = client::connect("127.0.0.1:6379").await?;
    // client.set("foo", "Hello".into()).await?;
    // println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
    // println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
    // }

    // {
    //     async fn process(stream: TcpStream) {
    //         // `mini-redis` 提供的便利函数，使用返回的 `connection` 可以用于从 socket 中读取数据并解析为数据帧
    //         // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
    //         let mut connection = Connection::new(stream);

    //         // 生成存储数据的HashMap
    //         let mut db = HashMap::new();

    //         // 在一个连接中可以传送多个帧数据，因此需要使用 while let 而不是 if let
    //         while let Some(frame) = connection.read_frame().await.unwrap() {
    //             println!("GOT: {}", frame);

    //             let response = match Command::from_frame(frame).unwrap() {
    //                 Set(cmd) => {
    //                     // 值被存储为 `Vec<u8>` 的形式
    //                     db.insert(cmd.key().to_string(), cmd.value().to_vec());
    //                     Frame::Simple("OK".to_string())
    //                 }
    //                 Get(cmd) => {
    //                     // `Frame::Bulk` 期待数据的类型是 `Bytes`，`&Vec<u8>` 可以使用 `into()` 方法转换成 `Bytes` 类型
    //                     if let Some(value) = db.get(cmd.key()) {
    //                         Frame::Bulk(value.clone().into())
    //                     } else {
    //                         Frame::Null
    //                     }
    //                 }
    //                 cmd => panic!("unimpement {:?}", cmd),
    //             };

    //             connection.write_frame(&response).await.unwrap();
    //         }
    //     }

    //     let listener = TcpListener::bind("127.0.0.1:6379").await?;
    //     loop {
    //         let (stream, addr) = listener.accept().await?;
    //         tokio::spawn(async move {
    //             process(stream).await;
    //         });
    //     }
    // }

    // {
    //     type DB = Arc<Mutex<HashMap<String, Bytes>>>;

    //     let listener = TcpListener::bind("127.0.0.1:6379").await?;
    //     let db: DB = Arc::new(Mutex::new(HashMap::new()));
    //     loop {
    //         let (stream, addr) = listener.accept().await?;
    //         let _db = Arc::clone(&db);
    //         tokio::spawn(async move {
    //             process(stream, _db).await;
    //         });
    //     }

    //     async fn process(stream: TcpStream, db: DB) {
    //         // `mini-redis` 提供的便利函数，使用返回的 `connection` 可以用于从 socket 中读取数据并解析为数据帧
    //         // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
    //         let mut connection = Connection::new(stream);

    //         // 在一个连接中可以传送多个帧数据，因此需要使用 while let 而不是 if let
    //         while let Some(frame) = connection.read_frame().await.unwrap() {
    //             println!("GOT: {}", frame);

    //             let response = match Command::from_frame(frame).unwrap() {
    //                 Set(cmd) => {
    //                     // 值被存储为 `Vec<u8>` 的形式
    //                     db.lock()
    //                         .unwrap()
    //                         .insert(cmd.key().to_string(), cmd.value().clone());
    //                     Frame::Simple("OK".to_string())
    //                 }
    //                 Get(cmd) => {
    //                     // `Frame::Bulk` 期待数据的类型是 `Bytes`，`&Vec<u8>` 可以使用 `into()` 方法转换成 `Bytes` 类型
    //                     if let Some(value) = db.lock().unwrap().get(cmd.key()) {
    //                         Frame::Bulk(value.clone().into())
    //                     } else {
    //                         Frame::Null
    //                     }
    //                 }
    //                 cmd => panic!("unimpement {:?}", cmd),
    //             };

    //             connection.write_frame(&response).await.unwrap();
    //         }
    //     }
    // }

    // {
    //     async fn increment_and_do_stuff() {
    //         async fn do_something_async() {}

    //         let mutex = Mutex::new(2);
    //         let mut lock = mutex.lock().unwrap();
    //         *lock += 1;

    //         do_something_async().await;
    //         // Mutex 横跨了 .await，作用域横跨 .await 的变量要求实现 Send 特征，以保证线程安全
    //     } // 锁在这里结束作用域，释放

    //     tokio::spawn(increment_and_do_stuff());

    //     async fn increment_and_do_stuff() {
    //         async fn do_something_async() {}

    //         {
    //             let mutex = Mutex::new(2);
    //             let mut lock = mutex.lock().unwrap();
    //             *lock += 1;
    //         } // 锁在这里结束作用域，释放

    //         do_something_async().await;
    //     }

    //     tokio::spawn(increment_and_do_stuff());
    // }

    {
        type DB = Arc<Mutex<HashMap<String, Bytes>>>;

        let listener = TcpListener::bind("127.0.0.1:6379").await?;
        let db: DB = Arc::new(Mutex::new(HashMap::new()));
        loop {
            let (stream, addr) = listener.accept().await?;
            let _db = Arc::clone(&db);
            tokio::spawn(async move {
                process(stream, _db).await;
            });
        }

        async fn process(stream: TcpStream, db: DB) {
            // `mini-redis` 提供的便利函数，使用返回的 `connection` 可以用于从 socket 中读取数据并解析为数据帧
            // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
            let mut connection = Connection::new(stream);

            // 在一个连接中可以传送多个帧数据，因此需要使用 while let 而不是 if let
            while let Some(frame) = connection.read_frame().await.unwrap() {
                println!("GOT: {}", frame);

                let response = match Command::from_frame(frame).unwrap() {
                    Set(cmd) => {
                        // 值被存储为 `Vec<u8>` 的形式
                        db.lock()
                            .unwrap()
                            .insert(cmd.key().to_string(), cmd.value().clone());
                        Frame::Simple("OK".to_string())
                    }
                    Get(cmd) => {
                        // `Frame::Bulk` 期待数据的类型是 `Bytes`，`&Vec<u8>` 可以使用 `into()` 方法转换成 `Bytes` 类型
                        if let Some(value) = db.lock().unwrap().get(cmd.key()) {
                            Frame::Bulk(value.clone().into())
                        } else {
                            Frame::Null
                        }
                    }
                    cmd => panic!("unimpement {:?}", cmd),
                };

                connection.write_frame(&response).await.unwrap();
            }
        }
    }

    Ok(())
}
