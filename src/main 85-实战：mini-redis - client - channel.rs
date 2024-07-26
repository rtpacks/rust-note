use std::time::Duration;

use bytes::Bytes;
use mini_redis::{client, Connection, Frame, Result};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> Result<()> {
    /*
     *
     * ## 实战：mini-redis - client - channel
     * > https://github.com/tokio-rs/mini-redis
     *
     * server 基础状态存储完成后，将其移动到 `/bin/server.rs` 中，并在 Cargo.toml 中添加：
     * ```toml
     * [[bin]]
     * name = "server"
     * path = "bin/server.rs"
     * ```
     *
     * 通过 `cargo run --bin server` 启动服务端等待使用，现在完成客户端的基本功能。
     *
     * 在 examples/redis-server-test 中，是串行的操作 mini-redis：
     * ```rust
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
     * 如果改成并发操作 mini-redis：
     * ```rust
     * let mut client = client::connect("127.0.0.1:6379").await?;
     * let h1 = tokio::spawn(async move {
     *     client.set("foo", "Hello".into()).await.unwrap();
     *     println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
     * });
     * let h2 = tokio::spawn(async move {
     *     println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
     * });
     *
     * tokio::join!(h1, h2);
     * ```
     *
     * 很明显报错了，因为 client 不能被多次 move。
     *
     * 根据上一节【unit 84-实战：mini-redis - state】中的分析，在 async 编程中，标准库中的同步锁不适合用在跨 .await 的逻辑中，因为这可能会导致死锁。
     * Tokio 提供的锁虽然可以跨 .await 使用，但是一方面它的性能会受到一定影响，另一方面由于锁的存在，同一时刻只能有一个执行线程使用锁住的共享资源，这样很容易出现性能瓶颈。
     *
     * 使用 tokio 提供的锁 `tokio::sync::Mutex`：
     * ```rust
     * let client = client::connect("127.0.0.1:6379").await?;
     * let client = Arc::new(tokio::sync::Mutex::new(client));
     * let _client = Arc::clone(&client);
     * let h1 = tokio::spawn(async move {
     *     let mut client = _client.lock().await;
     *     client.set("foo", "Hello".into()).await.unwrap();
     *     println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
     * });
     *
     * let _client = Arc::clone(&client);
     * let h2 = tokio::spawn(async move {
     *     let mut client = _client.lock().await;
     *     client.set("bar", "Hello".into()).await.unwrap();
     *     println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
     * });
     *
     * tokio::join!(h1, h2);
     * ```
     *
     * 运行程序，注意如果在生成 bin/server.rs 后 Cargo.toml 没有设置 default-run 属性，无法正常启动，这个时候可以在 Cargo.toml 中设置 default-run 与项目同名：
     *
     * ```toml
     * [package]
     * name = "ilearn"
     * version = "0.1.0"
     * edition = "2021"
     * default-run = "ilearn"
     * ```
     *
     * 或者直接通过 `cargo run --bin ilearn` 运行 main.rs。
     *
     *
     * ### 消息通道
     * tokio::sync::Mutex 稍不注意就会达到性能瓶颈，分析上面带锁的代码逻辑，虽然可以并发执行任务 Future，但其实因为锁只有在一个请求完成后才会释放，导致另外一个请求在等待中，这意味着代码虽然是并发执行的，但请求是串行的。
     * 不仅请求是串行的，假设当前任务已经请求完成，但需要做其他后续操作，由于锁还没有释放，导致其他任务无法发起操作请求，需要等待当前任务完成后释放锁，也就是说只有一个任务执行完成后，另外一个任务才会开始请求并处理后续操作。
     *
     *
     * 在高并发状态下，锁的竞争会非常激烈，再次影响程序性能。
     *
     * 在【unit 61-线程同步：消息传递】介绍中，除了锁，rust 还支持另外一种并发模型：通过通信来共享内存。
     *
     * > 在 Go 语言中有一句很经典的话：
     * > Do not communicate by sharing memory; instead, share memory by communicating
     * > 不要通过共享内存来进行通信，而是通过通信来共享内存
     * >
     * > 简单理解：尽量避免访问同一块内存空间来通信，因为它会造成的并发问题如竞争条件（Race condition），死锁（Deadlocks）等。
     * > 而是应该通过消息通知（触发）进行数据传递，例如消息队列、Socket 等方法。不同进程或线程之间通过这些通信机制共享数据，避免共享内存造成的并发问题。
     *
     * 这里同样可以使用消息通道来完成，之前使用的是标准库提供的同步消息通道，同步消息通道接收消息时会阻塞当前的线程，不适用于 async 编程。
     *
     * 现在可以使用 tokio 提供的**异步消息通道**，Tokio 提供了多种消息通道，可以满足不同场景的需求:
     * - mpsc, 多生产者，单消费者模式
     * - oneshot, 单生产者，单消费者，一次只能发送一条消息
     * - broadcast，多生产者，多消费者，每一条发送的消息都可以被所有接收者收到，因此是广播
     * - watch，单生产者，多消费者，只保存一条最新的消息，因此接收者只能看到最近的一条消息，这种模式适用于配置文件变化的监听
     *
     * 当然，如果需要 “多生产者、多消费者，且每一条消息只能被其中一个消费者接收” 的消息通道，可以使用 async-channel 包。
     *
     * 在上面提到过，由于 client 被锁住，所以即使当前任务的 redis 请求已经完成，但是还需要处理后续逻辑，导致锁还未被释放，以致其他任务被阻塞无法执行，只有一个任务执行完成后，另外一个任务才会开始请求并处理后续操作。
     *
     * 在这里设想，如果可以从一个指令队列中取出不同的指令，让某个任务仅处理与 redis 服务端交互的这部分逻辑，即专门去执行这些 redis 指令，
     * 执行 redis 指令，获取结果后就通知发起 redis 指令的原始任务，这样就可以避免指令请求的逻辑与其他后续的处理逻辑在同一个任务中处理，导致这些发起指令的原始任务在串行执行的逻辑。
     * 原始任务需要在发起 redis 指令后阻塞的等待 redis 指令完成后的通知，这个场景可以由另外一个消息通道完成，即由专门执行 redis 指令的任务发起通知，原始任务接收通知。
     *
     * 这里最大的改动就是将 redis 指令处理的这部分逻辑与其他后续处理逻辑分开，避免原始任务锁住 client，减少了强占 client 的时间，避免了**虽然任务在并发执行，但其实是串行的逻辑**。
     *
     * > 不恰当的使用锁就会导致看似并发执行任务，但其实是串行的逻辑。因为锁让其他任务在等待，并没有在执行。
     *
     * 1. 定义一个消息通道，用来传送客户端发送给服务端的 redis 指令
     * 2. 原始任务发出 redis 指令，阻塞等待指令完成通知
     * 3. client 任务管理 client 连接，接收并执行 redis 指令，在指令完成后通知对应的原始任务
     *
     * 生产者 Producer 和消费者 Consumer 进行通信的消息通道是有缓冲的，当大量的消息发送给消费者时，首先会放入消息通道的缓冲区中，如果消息通道已满，那么发送消息的任务就会被阻塞。
     * 当消费者处理完一条消息后，再从该缓冲区中取出下一条消息进行处理，这种方式跟消息队列( Message queue ) 非常类似，可以实现更高的吞吐。
     * 而且这种方式还有利于实现连接池，例如不止一个 P 和 C 时，多个 P 可以往消息通道中发送消息，同时多个 C，其中每个 C 都维护一条连接，并从消息通道获取消息。
     *
     * ```rust
     * enum CMD {
     *     Get,
     *     Set,
     * }
     *
     * let (tx, mut rx) = tokio::sync::mpsc::channel(32);
     * // 并发执行任务，不再因为锁导致串行
     * let tx1 = tx.clone();
     * tokio::spawn(async move {
     *     println!("task: start redis CMD::Set");
     *     tx1.send((CMD::Set, "foo 1".to_string())).await.unwrap();
     *     tokio::time::sleep(Duration::from_secs(4)).await;
     *     println!("task: end redis CMD::Set");
     * });
     * let tx2 = tx.clone();
     * tokio::spawn(async move {
     *     println!("task: start redis CMD::Get");
     *     tx2.send((CMD::Get, "foo".to_string())).await.unwrap();
     *     tokio::time::sleep(Duration::from_secs(2)).await;
     *     println!("task: end redis CMD::Get");
     * });
     *
     * // 注意需要清除所有的发送者之后，接收者才会受到通道关闭的消息，然后主动释放，不再阻塞主线程
     * drop(tx);
     *
     * while let Some(cmd) = rx.recv().await {
     *     match cmd {
     *         (CMD::Set, map) => {
     *             println!("CMD::Set {:?}", map)
     *         }
     *         (CMD::Get, key) => {
     *             println!("CMD::Get {:?}", key)
     *         }
     *     }
     * }
     * ```
     *
     * 当然，仅使用 `CMD::Get` 和 `CMD::Set` 太简单了，将指令和数据封装起来，并调用 client 进行真实的指令操作：
     *
     * ```rust
     * #[derive(Debug)]
     * enum Command {
     *     Set { key: String, val: Bytes },
     *     Get { key: String },
     * }
     *
     * let (tx, mut rx) = tokio::sync::mpsc::channel(32);
     * // 并发执行任务，不再因为锁导致串行
     * let tx1 = tx.clone();
     * let p1 = tokio::spawn(async move {
     *     println!("task: start redis CMD::Set");
     *     tx1.send(Command::Set {
     *         key: "foo".to_string(),
     *         val: "1".into(),
     *     })
     *     .await
     *     .unwrap();
     *
     *     tokio::time::sleep(Duration::from_secs(4)).await;
     *     println!("task: end redis CMD::Set");
     * });
     * let tx2 = tx.clone();
     * let p2 = tokio::spawn(async move {
     *     println!("task: start redis CMD::Get");
     *     tx2.send(Command::Get {
     *         key: "foo".to_string(),
     *     })
     *     .await
     *     .unwrap();
     *
     *     tokio::time::sleep(Duration::from_secs(2)).await;
     *     println!("task: end redis CMD::Get");
     * });
     *
     * let mut client = client::connect("127.0.0.1:6379").await?;
     * let c = tokio::spawn(async move {
     *     while let Some(cmd) = rx.recv().await {
     *         match cmd {
     *             Command::Set { key, val } => {
     *                 client.set(&key, val).await.unwrap();
     *             }
     *             Command::Get { key } => {
     *                 let value = client.get(&key).await.unwrap().unwrap();
     *                 println!("{:?}", value);
     *             }
     *         }
     *     }
     * });
     *
     * // 注意需要清除所有的发送者之后，接收者才会受到通道关闭的消息，然后主动释放，不再阻塞主线程
     * drop(tx);
     * tokio::join!(p1, p2, c);
     * ```
     *
     * 这里还剩下一个问题，当原始任务发出 redis 指令后，需要阻塞等待 redis 任务的指令执行完成通知，然后原始任务继续完成后续的逻辑。
     * 这里可以使用 oneshot 消息通道，因为它针对一发一收的使用类型做过特别优化，且特别适用于此时的场景：接收一条从管理任务发送的结果消息。
     *
     * 使用方式跟 mpsc 很像，但是它并没有缓存长度，因为只能发送一条，接收一条，还有一点不同：无法对返回的两个句柄进行 clone，因为它属于单发送者单接收者。
     *
     * ```rust
     * use tokio::sync::oneshot;
     * let (tx, rx) = oneshot::channel();
     * ```
     *
     * 在这里考虑，redis 任务应该通过 oneshot 消息通道通知原始任务，或者从另外一个角度看，怎么获取 oneshot 消息通道的发送者。
     *
     * 为了精准的通知原始任务，oneshot 消息通道的发送者必须要随着命令一起发出，原始任务 oneshot 消息通道的接收端。
     * 最后 redis 任务执行指令成功后调用 oneshot 发送者向原始任务发送通知。
     *
     * 这里一个比较好的实现就是将 oneshot 的发送者放入 Command 的数据结构中，使用一个别名来代表该发送端，补齐 redis 任务逻辑：
     *
     * ```rust
     * /// 管理任务可以使用该发送端将命令执行的结果传回给发出命令的任务
     * type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;
     *
     * #[derive(Debug)]
     * enum Command {
     *     Set {
     *         key: String,
     *         val: Bytes,
     *         resp: Responder<()>,
     *     },
     *     Get {
     *         key: String,
     *         resp: Responder<Option<Bytes>>,
     *     },
     * }
     *
     * let (tx, mut rx) = tokio::sync::mpsc::channel(32);
     * // 并发执行任务，不再因为锁导致串行
     * let tx1 = tx.clone();
     * let p1 = tokio::spawn(async move {
     *     println!("task: start redis CMD::Set");
     *
     *     let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
     *     tx1.send(Command::Set {
     *         key: "foo".to_string(),
     *         val: "1".into(),
     *         resp: resp_tx,
     *     })
     *     .await
     *     .unwrap();
     *
     *     // 任务阻塞等待 redis 任务的通知，通知后恢复执行
     *     let res = resp_rx.await;
     *
     *     tokio::time::sleep(Duration::from_secs(4)).await;
     *     println!("task: end redis CMD::Set");
     * });
     * let tx2 = tx.clone();
     * let p2 = tokio::spawn(async move {
     *     println!("task: start redis CMD::Get");
     *
     *     let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
     *     tx2.send(Command::Get {
     *         key: "foo".to_string(),
     *         resp: resp_tx,
     *     })
     *     .await
     *     .unwrap();
     *
     *     // 任务阻塞等待 redis 任务的通知，通知后恢复执行
     *     let res = resp_rx.await;
     *
     *     tokio::time::sleep(Duration::from_secs(2)).await;
     *     println!("task: end redis CMD::Get");
     * });
     *
     * let mut client = client::connect("127.0.0.1:6379").await?;
     * let c = tokio::spawn(async move {
     *     while let Some(cmd) = rx.recv().await {
     *         match cmd {
     *             Command::Set { key, val, resp } => {
     *                 let res = client.set(&key, val).await;
     *                 resp.send(res).unwrap();
     *             }
     *             Command::Get { key, resp } => {
     *                 let value = client.get(&key).await;
     *                 println!("{:?}", value);
     *                 resp.send(value).unwrap();
     *             }
     *         }
     *     }
     * });
     *
     * // 注意需要清除所有的发送者之后，接收者才会受到通道关闭的消息，然后主动释放，不再阻塞主线程
     * drop(tx);
     * tokio::join!(p1, p2, c);
     * ```
     *
     * 有一点值得注意，往 oneshot 中发送消息时，并没有使用 .await，原因是该发送操作要么直接成功、要么失败，并不需要等待。
     * 在 redis 任务发送通知后，resp_tx 会被释放，然后原始任务中的 resp_rx 接收第一个消息后就不会再阻塞任务，最后原始任务运行结束。
     *
     *
     * ### 控制并发程度
     * async 操作在 Tokio 中是惰性的，必须要显式地引入并发和队列:
     * - tokio::spawn
     * - select!
     * - join!
     * - mpsc::channel
     *
     * 在使用通道时，需要小心的控制并发度来确保系统的安全。例如，当使用一个循环去接收 TCP 连接时，你要确保当前打开的 socket 数量在可控范围内，而不是毫无原则的接收连接。
     *
     * 无论何时使用消息通道，都需要对缓存队列的长度进行限制，这样系统才能优雅的处理各种负载状况。
     * 如果不限制，假设接收端无法及时处理消息，那消息就会迅速堆积，就可能导致整体性能的大幅下降，最终可能会使内存消耗殆尽。
     *
     * 阅读：https://course.rs/advance-practice/channels.html#对消息通道进行限制
     *
     *
     * ### 总结
     * 这里使用了两个消息通道完成发送者 `->` 接收者 `->` 发送者的流程。
     * 最大的改动就是将 redis 指令处理的这部分逻辑与其他后续处理逻辑分开，避免原始任务锁住 client，减少了强占 client 的时间，避免了**虽然任务在并发执行，但其实是串行的逻辑**。
     *
     */

    // {
    //     let mut client = client::connect("127.0.0.1:6379").await?;
    //     let h1 = tokio::spawn(async move {
    //         client.set("foo", "Hello".into()).await.unwrap();
    //         println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
    //     });
    //     let h2 = tokio::spawn(async move {
    //         println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
    //     });
    //     tokio::join!(h1, h2);
    // }

    // {
    //     let client = client::connect("127.0.0.1:6379").await?;
    //     let client = Arc::new(tokio::sync::Mutex::new(client));
    //     let _client = Arc::clone(&client);
    //     let h1 = tokio::spawn(async move {
    //         let mut client = _client.lock().await;
    //         client.set("foo", "Hello".into()).await.unwrap();
    //         println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
    //     });
    //     let _client = Arc::clone(&client);
    //     let h2 = tokio::spawn(async move {
    //         let mut client = _client.lock().await;
    //         client.set("bar", "Hello".into()).await.unwrap();
    //         println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
    //     });
    //     tokio::join!(h1, h2);
    // }

    // {
    //     use tokio::sync::mpsc;

    //     let (tx, mut rx) = mpsc::channel(32);
    //     let tx2 = tx.clone();

    //     tokio::spawn(async move {
    //         tx.send("sending from first handle").await;
    //     });

    //     tokio::spawn(async move {
    //         tx2.send("sending from second handle").await;
    //     });

    //     while let Some(message) = rx.recv().await {
    //         println!("GOT = {}", message);
    //     }
    // }

    // {
    //     enum CMD {
    //         Get,
    //         Set,
    //     }

    //     let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    //     // 并发执行任务，不再因为锁导致串行
    //     let tx1 = tx.clone();
    //     tokio::spawn(async move {
    //         println!("task: start redis CMD::Set");
    //         tx1.send((CMD::Set, "foo 1".to_string())).await.unwrap();
    //         tokio::time::sleep(Duration::from_secs(4)).await;
    //         println!("task: end redis CMD::Set");
    //     });
    //     let tx2 = tx.clone();
    //     tokio::spawn(async move {
    //         println!("task: start redis CMD::Get");
    //         tx2.send((CMD::Get, "foo".to_string())).await.unwrap();
    //         tokio::time::sleep(Duration::from_secs(2)).await;
    //         println!("task: end redis CMD::Get");
    //     });

    //     // 注意需要清除所有的发送者之后，接收者才会受到通道关闭的消息，然后主动释放，不再阻塞主线程
    //     drop(tx);

    //     while let Some(cmd) = rx.recv().await {
    //         match cmd {
    //             (CMD::Set, map) => {
    //                 println!("CMD::Set {:?}", map)
    //             }
    //             (CMD::Get, key) => {
    //                 println!("CMD::Get {:?}", key)
    //             }
    //         }
    //     }
    // }

    // {
    //     #[derive(Debug)]
    //     enum Command {
    //         Set { key: String, val: Bytes },
    //         Get { key: String },
    //     }

    //     let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    //     // 并发执行任务，不再因为锁导致串行
    //     let tx1 = tx.clone();
    //     let p1 = tokio::spawn(async move {
    //         println!("task: start redis CMD::Set");
    //         tx1.send(Command::Set {
    //             key: "foo".to_string(),
    //             val: "1".into(),
    //         })
    //         .await
    //         .unwrap();

    //         tokio::time::sleep(Duration::from_secs(4)).await;
    //         println!("task: end redis CMD::Set");
    //     });
    //     let tx2 = tx.clone();
    //     let p2 = tokio::spawn(async move {
    //         println!("task: start redis CMD::Get");
    //         tx2.send(Command::Get {
    //             key: "foo".to_string(),
    //         })
    //         .await
    //         .unwrap();

    //         tokio::time::sleep(Duration::from_secs(2)).await;
    //         println!("task: end redis CMD::Get");
    //     });

    //     let mut client = client::connect("127.0.0.1:6379").await?;
    //     let c = tokio::spawn(async move {
    //         while let Some(cmd) = rx.recv().await {
    //             match cmd {
    //                 Command::Set { key, val } => {
    //                     client.set(&key, val).await.unwrap();
    //                 }
    //                 Command::Get { key } => {
    //                     let value = client.get(&key).await.unwrap().unwrap();
    //                     println!("{:?}", value);
    //                 }
    //             }
    //         }
    //     });

    //     // 注意需要清除所有的发送者之后，接收者才会受到通道关闭的消息，然后主动释放，不再阻塞主线程
    //     drop(tx);
    //     tokio::join!(p1, p2, c);
    // }

    {
        /// 管理任务可以使用该发送端将命令执行的结果传回给发出命令的任务
        type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

        #[derive(Debug)]
        enum Command {
            Set {
                key: String,
                val: Bytes,
                resp: Responder<()>,
            },
            Get {
                key: String,
                resp: Responder<Option<Bytes>>,
            },
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        // 并发执行任务，不再因为锁导致串行
        let tx1 = tx.clone();
        let p1 = tokio::spawn(async move {
            println!("task: start redis CMD::Set");

            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            tx1.send(Command::Set {
                key: "foo".to_string(),
                val: "1".into(),
                resp: resp_tx,
            })
            .await
            .unwrap();

            // 任务阻塞等待 redis 任务的通知，通知后恢复执行
            let res = resp_rx.await;

            tokio::time::sleep(Duration::from_secs(4)).await;
            println!("task: end redis CMD::Set");
        });
        let tx2 = tx.clone();
        let p2 = tokio::spawn(async move {
            println!("task: start redis CMD::Get");

            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            tx2.send(Command::Get {
                key: "foo".to_string(),
                resp: resp_tx,
            })
            .await
            .unwrap();

            // 任务阻塞等待 redis 任务的通知，通知后恢复执行
            let res = resp_rx.await;

            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("task: end redis CMD::Get");
        });

        let mut client = client::connect("127.0.0.1:6379").await?;
        let c = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    Command::Set { key, val, resp } => {
                        let res = client.set(&key, val).await;
                        resp.send(res).unwrap();
                    }
                    Command::Get { key, resp } => {
                        let value = client.get(&key).await;
                        println!("{:?}", value);
                        resp.send(value).unwrap();
                    }
                }
            }
        });

        // 注意需要清除所有的发送者之后，接收者才会受到通道关闭的消息，然后主动释放，不再阻塞主线程
        drop(tx);
        tokio::join!(p1, p2, c);
    }

    Ok(())
}
