use std::{
    sync::mpsc::{self, Sender, SyncSender},
    thread::{self, JoinHandle},
    time::Duration,
};

fn main() {
    /*
     *
     * ## 线程同步：消息传递
     * > 注意：在 rust 线程中借用外部的引用必须拥有 `'static` 生命周期。
     *
     * 在多线程间有多种方式可以共享和传递数据，最常用有两种：
     * - 消息传递
     * - 锁和Arc联合使用
     *
     * 对于消息传递，在编程界有一个大名鼎鼎的 **Actor 线程模型**为其背书，典型的有 Erlang 语言，还有 Go 语言。
     *
     * > 在 Go 语言中有一句很经典的话：
     * > Do not communicate by sharing memory; instead, share memory by communicating
     * > 不要通过共享内存来进行通信，而是通过通信来共享内存
     * >
     * > 简单理解：尽量避免访问同一块内存空间来通信，因为它会造成的并发问题如竞争条件（Race condition），死锁（Deadlocks）等。
     * > 而是应该通过消息通知（触发）进行数据传递，例如消息队列、Socket 等方法。不同进程或线程之间通过这些通信机制共享数据，避免共享内存造成的并发问题。
     *
     * 与 Go 语言直接内置 chan 关键字不同，rust 通过标准库的 `channel` 提供消息通道。
     *
     * 消息常常被视为信息的反映形式之一，是信息的外壳。但消息/信息没有个统一认可的定义。在香浓的《通信数学理论》中，他认为：
     * 从通信角度看，信息是通信的内容。通信的目的就是要减少或消除接收端(信宿)对于发出端(信源)可能会发出哪些消息的不确定性。
     * 所谓不确定性，就是指人们对客观事物的不了解或不清楚程度。
     * 人们通过某种方式或手段，获取了新的情况或知识，就可从对客观事物的不清楚变为较清楚或完全清楚，不确定性也就减少或消除了。
     * 这种使人们减少或消除不确定性的东西就是信息。
     *
     * 简单理解，消息是发送者发信息给接收者的音讯，它更多的是指一个音讯整体，包含发送者和接收者。
     * 消息通过消息通道进行传播，一个消息通道可以传播多个消息，因此消息通道应该支持多个发送者和接收者。
     *
     * 在实际使用中，需要使用不同的库来满足诸如：`多发送者 -> 单接收者`，`多发送者 -> 多接收者` 等场景形式。
     * > 消息管道一般不区分单发送者和多发送者，因为支持多发送者就是支持单发送者。
     * > 1. 在实际应用中，通常需要多个发送者向同一个接收者发送消息，单发送者的场景相对较少。
     * > 2. 多发送者形式更加灵活和通用，能满足单个发送者功能。
     * > 3. 从设计的角度来看，多发送者形式更加符合消息管道的本质。消息管道的目的是将消息从发送者传递到接收者，而不管发送者和接收者的数量。
     *
     * 当发送者或接收者任一被丢弃时可以认为通道被关闭（closed）了。
     *
     * ### 多发送者，单接收者
     * 标准库提供了通道 `std::sync::mpsc`，其中 `mpsc` 是 `multiple producer, single consumer` 的缩写，代表了该通道支持多个发送者，但是只支持唯一的接收者。
     * 当然，支持多个发送者也意味着支持单个发送者。
     *
     * 在实际使用过程中，发送者 `transmitter` 常被简写为 `tx`，接收者 `receiver` 被简写为 `rx`。
     *
     * **单发送者，单接收者**
     * ```rust
     * //  创建消息通道，返回发送者、接收者元组（transmitter，receiver）
     * // let (tx, rx) = mpsc::channel::<i32>(); // 手动指定消息通道类型
     * let (tx, rx) = mpsc::channel(); // 可以手动指定类型，也可以由编译器推导类型，如果编译器没有推导出类型，则会报错
     *
     * let handle = thread::spawn(move || {
     *     tx.send(1).unwrap(); // 编译器自动推导出类型，发送者为 Sender<i32>，接收者为 Receiver<i32>，后续管道无法发送其他类型
     *                          // send 方法返回 Result，说明它有可能返回一个错误，例如接收者被 drop 导致了发送的值不会被任何人接收，此时继续发送毫无意义，因此返回一个错误最为合适
     *
     *     // tx.send(Some(1)); 错误，经过 `tx.send(1)` 后管道被推导为只能传送 i32 类型
     * });
     * println!("{}", rx.recv().unwrap()); // recv 方法会阻塞当前线程，直到读取到值或者通道被关闭才会解除阻塞
     * ```
     *
     * 以上代码并不复杂，但仍有几点需要注意：
     * - tx,rx 对应发送者和接收者，它们的类型由编译器自动推导: 因为 tx.send(1) 发送了整数，所以编译器推导它们分别是 `mpsc::Sender<i32>` 和 `mpsc::Receiver<i32>` 类型
     * - 由于通道内部是泛型实现，一旦类型被推导确定，该通道就只能传递对应类型的值，否则会导致类型错误。
     * - 接收消息的操作 rx.recv() 会阻塞当前线程，直到读取到值，或者通道被关闭
     * - 需要使用 move 将 tx 的所有权转移到子线程的闭包中
     *
     * send 方法返回一个 `Result<T,E>`，说明它有可能返回一个错误，例如接收者被 drop 导致**发送的值不会被任何人接收**，此时继续发送毫无意义，因此返回一个错误最为合适。
     * 同样的，对于 recv 方法来说，当发送者关闭时，它也会接收到一个错误，用于说明**不会再有任何值被发送过来**。
     *
     * ### 不阻塞的 try_recv
     *
     * recv方法在通道中没有消息时会阻塞当前线程，如果不希望阻塞线程，可以使用 try_recv，try_recv 会尝试接收一次消息，如果通道中没有消息，会立刻返回一个错误。
     * ```rust
     * // try_recv 会立即尝试接收一次消息，如果通道中没有消息则会返回一个错误
     * let (tx, rx) = mpsc::channel();
     * thread::spawn(move || {
     *     tx.send(1);
     * });
     * match rx.try_recv() {
     *     Ok(n) => println!("{n}"),
     *     Err(e) => eprintln!("{}", e), // 在子线程未创建前，通道中没有信息，try_recv 返回 empty channel 错误
     * }
     * match rx.recv() {
     *     Ok(n) => println!("{n}"),
     *     Err(e) => eprintln!("{}", e), // 利用 recv 阻塞，区分两种类型的错误
     * }
     * match rx.try_recv() {
     *     Ok(n) => println!("{n}"),
     *     Err(e) => eprintln!("{}", e), // 在子线程结束后，通道被关闭，try_recv 返回 closed channel 错误
     * }
     * ```
     * 由于子线程的创建需要时间，第一个 `match rx.try_recv` 执行时子线程的消息还未发出。因为消息没有发出，try_recv 在立即尝试读取一次消息后就会报错，返回 empty channel 错误。
     * 当子线程创建成功且发送消息后，主线程会接收到 Ok(1) 的消息内容，紧接着子线程结束，发送者也随着被 drop，此时接收者又会报错，但是这次错误原因有所不同：closed channel 代表发送者已经被关闭。
     *
     * ### 传输数据的所有权
     * 使用通道来传输数据，一样要遵循 Rust 的所有权规则：
     * - 若值的类型实现了 Copy 特征，则直接复制一份该值，然后传输
     * - 若值没有实现 Copy 特征，则它的所有权会被**转移给接收端**，在发送端继续使用该值将报错
     *
     * ```rust
     * // 消息管理会转移非 Copy 类型的所有权
     * let (tx, rx) = mpsc::channel();
     * thread::spawn(move || {
     *     let s = String::from("Hello World");
     *     tx.send(s);
     *     // println!("{s}"); 不能再使用s，s的所有权被转移
     * });
     * println!("{}", rx.recv().unwrap());
     * ```
     *
     * 假如没有所有权的保护，String 字符串将被两个线程同时持有，任何一个线程对字符串内容的修改都会导致另外一个线程持有的字符串被改变，除非故意这么设计，否则这就是不安全的隐患。
     *
     * ### 循环接收消息
     * 消息通道中的消息数量是不确定的，为了方便接收所有消息以及在通道关闭时自动停止接收者接收消息，rust 为接收者 Receiver 实现了可迭代特征协议(IntoIterator)。
     *
     * ```rust
     * impl<T> Iterator for IntoIter<T> {
     *     type Item = T;
     *     fn next(&mut self) -> Option<T> {
     *         self.rx.recv().ok()
     *     }
     * }
     *
     * impl<T> IntoIterator for Receiver<T> {
     *     type Item = T;
     *     type IntoIter = IntoIter<T>;
     *
     *     fn into_iter(self) -> IntoIter<T> {
     *         IntoIter { rx: self }
     *     }
     * }
     * ```
     *
     * `rx.recv()` 阻塞当前线程直到发送者或通道关闭，结合迭代器说明可以对 `rx` 进行循环操作，即可取出通道内的所有消息。
     * ```rust
     * // Receiver 接收者实现了可迭代特征，可以使用 for 遍历 Receiver 接收者
     * let (tx, rx) = mpsc::channel();
     * thread::spawn(move || {
     *     let msgs = vec![
     *         String::from("Test"),
     *         String::from("Hello"),
     *         String::from("World"),
     *         String::from("!"),
     *     ];
     *     for msg in msgs {
     *         tx.send(msg);
     *     }
     * });
     * // 消费了一条消息，消息通道内减少一条
     * match rx.recv() {
     *     Ok(msg) => println!("{msg}"),
     *     Err(e) => eprintln!("{e}"),
     * }
     *
     * // 使用 for 遍历 Receiver 接收者，即可取出通道内的消息
     * for msg in rx {
     *     print!("{msg}");
     * }
     * ```
     *
     * ### mpsc 的多发送者
     * 发送者 Sender 和 Arc 一样实现了 Send 特征，可以在多线程中共享数据。
     *
     * 使用多发送者时，和在多线程中使用 Arc 一样，复制一份引用即可：
     * ```rust
     * // 使用多发送者，Sender 和 Arc 一样实现了 Send 特征，可以在多线程中共享数据
     * let (tx, rx) = mpsc::channel();
     * let count = 5;
     * let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
     * for i in 0..count {
     *     // Sender 和 Arc 一样实现了 Send 特征，可以在多线程中共享数据
     *     let _tx = Sender::clone(&tx);
     *     handles.push(thread::spawn(move || {
     *         _tx.send(i).unwrap();
     *     }));
     * }
     * // 只有所有发送者释放后，消息通道才会因为没有发送者而关闭，进而释放 rx，这里需要在阻塞线程前主动释放 tx
     * drop(tx);
     * // 使用 for 遍历 Receiver 接收者，即可取出通道内的消息，
     * for msg in rx {
     *     println!("{}", msg);
     * }
     * ```
     * 有几点需要注意:
     * - 需要所有的发送者都被 drop 掉后，接收者 rx 才会收到错误，进而跳出 for 循环，最终结束主线程，因此要提前销毁 tx
     * - 由于子线程谁先创建完成是未知的，因此哪条消息先发送也是未知的，最终主线程消息的**输出顺序也不确定**
     *
     * ### 同步和异步通道
     * Rust 标准库的 mpsc 通道其实分为两种类型：同步和异步。
     *
     * #### 异步通道
     * 异步：发送操作不会阻塞当前线程，无论消息是否被接收，继续执行当前线程。即无论接收者是否正在接收消息，消息发送者在发送消息时都不会阻塞。
     * ```rust
     * // mpsc::channel 是一个异步管道，发送操作不会阻塞当前线程
     * let (tx, rx) = mpsc::channel();
     * let count = 2;
     * let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
     * for i in 0..count {
     *     // Sender 和 Arc 一样实现了 Send 特征，可以在多线程中共享数据
     *     let _tx = Sender::clone(&tx);
     *     handles.push(thread::spawn(move || {
     *         println!("发送之前");
     *         _tx.send(i).unwrap(); // 发送操作不会阻塞当前线程
     *         println!("发送之后");
     *     }));
     * }
     * // 主线程阻塞，还未开始接收消息，但是子线程中发送操作正常运行
     * thread::sleep(Duration::from_secs(2));
     * // 只有所有发送者释放后，消息通道才会因为没有发送者而关闭，进而释放 rx，这里需要在阻塞线程前主动释放 tx
     * drop(tx);
     * // 使用 for 遍历 Receiver 接收者，即可取出通道内的消息，
     * for msg in rx {
     *     println!("{}", msg);
     * }
     * ```
     * 主线程因为睡眠阻塞了 2 秒，并没有进行消息接收，而子线程却在此期间轻松完成了消息的发送。发送之前和发送之后是连续输出的，没有受到接收端主线程的任何影响，
     * 等睡眠结束后，主线程才姗姗来迟的从通道中接收了子线程老早之前发送的消息。因此通过 `mpsc::channel` 创建的通道是**异步通道**。
     *
     * #### 同步通道
     * 与异步通道相反，同步通道的发送者**发送操作是可以阻塞当前线程的**，只有等发送者发出的消息被接收后，发送者所在的线程才会解除阻塞并继续执行。
     * 
     * ```rust
     * // mpsc::sync_channel 是一个同步通道，发送操作可以阻塞当前线程，只有等发出的消息被接收后，发送者所在的线程才会解除阻塞并继续执行
     * let (tx, rx) = mpsc::sync_channel(0);
     * let count = 3;
     * for i in 0..count {
     *     let _tx = SyncSender::clone(&tx);
     *     thread::spawn(move || {
     *         println!("同步通道，发送之前，idx = {i}");
     *         _tx.send(i).unwrap(); // 只有等消息被接收后才会解除阻塞，让当前线程继续执行
     *         println!("同步通道，发送之后，idx = {i}");
     *     });
     * }
     * drop(tx);
     * for msg in rx {
     *     println!("同步通道，接收消息，idx = {}", msg); // 与“发送之后”的输出顺序是不确定的
     * }
     * ```
     *
     */

    //  创建消息通道，返回发送者、接收者元组（transmitter，receiver）
    // let (tx, rx) = mpsc::channel::<i32>(); // 手动指定消息通道类型
    let (tx, rx) = mpsc::channel(); // 可以手动指定类型，也可以由编译器推导类型，如果编译器没有推导出类型，则会报错

    let handle = thread::spawn(move || {
        tx.send(1).unwrap(); // 编译器自动推导出类型，发送者为 Sender<i32>，接收者为 Receiver<i32>，后续管道无法发送其他类型
                             // send 方法返回 Result，说明它有可能返回一个错误，例如接收者被 drop 导致了发送的值不会被任何人接收，此时继续发送毫无意义，因此返回一个错误最为合适

        // tx.send(Some(1)); 错误，经过 `tx.send(1)` 后管道被推导为只能传送 i32 类型
    });
    println!("{}", rx.recv().unwrap());

    // try_recv 会立即尝试接收一次消息，如果通道中没有消息则会返回一个错误
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        tx.send(1);
    });
    match rx.try_recv() {
        Ok(n) => println!("{n}"),
        Err(e) => eprintln!("{}", e), // 在子线程未创建前，通道中没有信息，try_recv 返回 empty channel 错误
    }
    match rx.recv() {
        Ok(n) => println!("{n}"),
        Err(e) => eprintln!("{}", e), // 利用 recv 阻塞，区分两种类型的错误
    }
    match rx.try_recv() {
        Ok(n) => println!("{n}"),
        Err(e) => eprintln!("{}", e), // 在子线程结束后，通道被关闭，try_recv 返回 closed channel 错误
    }

    // 消息管理会转移非 Copy 类型的所有权
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let s = String::from("Hello World");
        tx.send(s);
        // println!("{s}"); 不能再使用s，s的所有权被转移
    });
    println!("{}", rx.recv().unwrap());

    // Receiver 接收者实现了可迭代特征，可以使用 for 遍历 Receiver 接收者
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let msgs = vec![
            String::from("Test"),
            String::from("Hello"),
            String::from("World"),
            String::from("!"),
        ];
        for msg in msgs {
            tx.send(msg);
        }
    });
    // 消费了一条消息，消息通道内减少一条
    match rx.recv() {
        Ok(msg) => println!("{msg}"),
        Err(e) => eprintln!("{e}"),
    }
    // 使用 for 遍历 Receiver 接收者，即可取出通道内的消息
    for msg in rx {
        println!("{msg}");
    }

    // 使用多发送者，Sender 和 Arc 一样实现了 Send 特征，可以在多线程中共享数据
    let (tx, rx) = mpsc::channel();
    let count = 5;
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
    for i in 0..count {
        // Sender 和 Arc 一样实现了 Send 特征，可以在多线程中共享数据
        let _tx = Sender::clone(&tx);
        handles.push(thread::spawn(move || {
            _tx.send(i).unwrap();
        }));
    }
    // 只有所有发送者释放后，消息通道才会因为没有发送者而关闭，进而释放 rx，这里需要在阻塞线程前主动释放 tx
    drop(tx);
    // 使用 for 遍历 Receiver 接收者，即可取出通道内的消息，
    for msg in rx {
        println!("{}", msg);
    }

    // mpsc::channel 是一个异步管道，发送操作不会阻塞当前线程
    let (tx, rx) = mpsc::channel();
    let count = 2;
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
    for i in 0..count {
        // Sender 和 Arc 一样实现了 Send 特征，可以在多线程中共享数据
        let _tx = Sender::clone(&tx);
        handles.push(thread::spawn(move || {
            println!("发送之前");
            _tx.send(i).unwrap(); // 发送操作不会阻塞当前线程
            println!("发送之后");
        }));
    }
    // 主线程阻塞，还未开始接收消息，但是子线程中发送操作正常运行
    thread::sleep(Duration::from_secs(2));
    // 只有所有发送者释放后，消息通道才会因为没有发送者而关闭，进而释放 rx，这里需要在阻塞线程前主动释放 tx
    drop(tx);
    // 使用 for 遍历 Receiver 接收者，即可取出通道内的消息，
    for msg in rx {
        println!("{}", msg);
    }

    // mpsc::sync_channel 是一个同步通道，发送操作可以阻塞当前线程，只有等发出的消息被接收后，发送者所在的线程才会解除阻塞并继续执行
    let (tx, rx) = mpsc::sync_channel(0);
    let count = 3;
    for i in 0..count {
        let _tx = SyncSender::clone(&tx);
        thread::spawn(move || {
            println!("同步通道，发送之前，idx = {i}");
            _tx.send(i).unwrap(); // 只有等消息被接收后才会解除阻塞，让当前线程继续执行
            println!("同步通道，发送之后，idx = {i}");
        });
    }
    drop(tx);
    for msg in rx {
        println!("同步通道，接收消息，idx = {}", msg); // 与“发送之后”的输出顺序是不确定的
    }
}
