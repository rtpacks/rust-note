use core::panic;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    /*
     * ## 返回值和错误处理
     * 错误对于软件来说是不可避免的，因此一门优秀的编程语言必须有其完整的错误处理哲学。
     * 在很多情况下，Rust 需要你承认自己的代码可能会出错，并提前采取行动，来处理这些错误。
     *
     * Rust 中的错误主要分为两类：
     * - 可恢复错误，通常用于从系统全局角度来看可以接受的错误，例如处理用户的访问、操作等错误，这些错误只会影响某个用户自身的操作进程，而不会对系统的全局稳定性产生影响
     * - 不可恢复错误，刚好相反，该错误通常是全局性或者系统性的错误，例如数组越界访问，系统启动时发生了影响启动流程的错误等等，这些错误的影响往往对于系统来说是致命的
     *
     * 很多编程语言，并不会区分这些错误，而是直接采用异常的方式去处理。
     * Rust 没有异常，但是 Rust 也有自己的卧龙凤雏：Result<T, E> 用于可恢复错误，panic! 用于不可恢复错误。
     *
     * ### 1. panic! 与不可恢复错误
     * 对于严重到影响程序运行的错误，触发 panic 是很好的解决方式。在 Rust 中触发 panic 有两种方式：被动触发和主动调用。
     *
     * #### 1. 被动触发
     * 被动触发是指代码中一些错误语法或错误指令触发的，如数组越界、访问不存在的对象等。
     * ```rs
     * let arr = [1, 2, 3];
     * println!("{:#?}", arr[99]);
     * ```
     * 被动触发的 panic 是我们日常开发中最常遇到的，这也是 Rust 给我们的一种保护，毕竟错误只有抛出来，才有可能被处理，否则只会偷偷隐藏起来，寻觅时机给你致命一击。
     *
     * #### 2. 主动触发
     * 在某些特殊场景中，开发者想要主动抛出一个异常，例如读取文件失败时抛出异常。
     * 对此，Rust 为我们提供了 panic! 宏，当调用执行该宏时，程序会打印出一个错误信息，展开报错点往前的函数调用堆栈，最后退出程序。
     *
     * > 切记，一定是不可恢复的错误，才调用 panic! 处理，如果知识因为用户随便传入一个非法参数，没有必要调用panic。只有当你**不知道该如何处理**时，再去调用 panic!.
     * ```rs
     * thread 'main' panicked at 'crash and burn', src/main.rs:2:5
     * note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
     * ```
     *
     * 主动触发panic时，输出的内容包含两条重要信息：
     * 1. main 函数所在的线程崩溃了，发生的代码位置是 src/main.rs 中的第 x 行第 y 个字符（去除该行前面的空字符）
     * 2. 在使用时加上一个环境变量可以获取更详细的栈展开信息：
     *     - Linux/macOS 等 UNIX 系统： RUST_BACKTRACE=1 cargo run
     *     - Windows 系统（PowerShell）： $env:RUST_BACKTRACE=1 ; cargo run
     *
     * #### 3. panic 时停止程序的两种方式
     * 当出现 panic! 时，程序提供了两种方式来处理终止流程：栈展开和直接终止。默认的方式就是栈展开.
     * - 栈展开： Rust 会回溯栈上数据和函数调用，因此也意味着更多的善后工作，好处是可以给出充分的报错信息和栈调用信息，便于事后的问题复盘。
     * - 直接终止，顾名思义，不清理数据就直接退出程序，善后工作交与操作系统来负责。
     *
     * #### 4. 线程 panic 后，程序是否会终止？
     * 如果是 main 线程，则程序会终止，如果是其它子线程，该线程会终止，但是不会影响 main 线程。因此，**尽量不要在 main 线程中做太多任务，将这些任务交由子线程去做**，就算子线程 panic 也不会导致整个程序的结束。
     * 具体解析见 [panic 原理剖析](https://course.rs/basic/result-error/panic.html#panic-%E5%8E%9F%E7%90%86%E5%89%96%E6%9E%90)。
     *
     * ### 2. Result 与可恢复的错误
     * #### 1. 使用Result
     * 大部分错误并没有严重到需要程序完全停止执行。有时，一个函数会因为一个容易理解并做出反应的原因失败。
     * 例如，如果因为打开一个并不存在的文件而失败，此时我们可能想要创建这个文件，而不是终止进程。
     *
     * 假设我们有一台消息服务器，每个用户都通过 websocket 连接到该服务器来接收和发送消息，该过程就涉及到 socket 文件的读写。
     * 如果一个用户的读写发生了错误，显然不能直接 panic，否则服务器会直接崩溃，所有用户都会断开连接。
     * 因此我们需要一种更温和的错误处理方式：Result<T, E>。
     *
     * Result的定义如下，泛型参数 T 代表成功时存入的正确值的类型，存放方式是 Ok(T)，E 代表错误时存入的错误值，存放方式是 Err(E)。
     * ```rs
     * enum Result<T, E> {
     *      Ok(T),
     *      Err(E)
     * }
     * ```
     * 打开文件的示例，File::open 返回一个 Result 类型：`std::result::Result<std::fs::File, std::io::Error>`
     * ```rs
     * use std::fs::File;
     * fn main() {
     *     let f = File::open("hello.txt");
     * }
     * ```
     * > **如何获知变量类型或者函数的返回类型？**
     * > 有几种常用的方式，此处更推荐第二种方法：
     * > - 第一种是查询标准库或者三方库文档，搜索 File，然后找到它的 open 方法
     * > - 在 Rust IDE 章节，我们推荐了 VSCode IDE 和 rust-analyzer 插件，如果你成功安装的话，那么就可以在 VSCode 中很方便的通过代码跳转的方式查看代码，同时 rust-analyzer 插件还会对代码中的类型进行标注，非常方便好用！
     * > - 你还可以尝试故意标记一个错误的类型，然后让编译器告诉你。
     *
     * 这个返回值类型说明 File::open 调用可能会成功并返回一个可以进行读写的文件句柄。
     * 这个函数也可能会失败：例如，文件可能并不存在，或者可能没有访问文件的权限。
     * File::open 需要一个方式告诉我们是成功还是失败，并同时提供给我们文件句柄或错误信息。而这些信息正是 Result 枚举可以提供的。
     *
     * ```rs
     * let f = File::open("hello.txt");
     * let f = match f {
     *     Ok(file) => file,
     *     Err(error) => {
     *         panic!("Problem opening the file: {:?}", error)
     *     },
     * };
     * ```
     * 代码很清晰，对打开文件后的 Result<T, E> 类型进行匹配取值。
     * 如果是成功，则将 Ok(file) 中存放的的文件句柄 file 赋值给 f，如果失败，则将 Err(error) 中存放的错误信息 error 使用 panic 抛出来，进而结束程序。
     *
     * #### 2. 对返回值进行处理
     * 在文件打开的示例代码中，当打开文件错误时，被错误分支捕获并直接执行错误分支panic!，显然这不符合使用Result的可处理错误的目标。
     * 文件读取失败的原因有很多种，我们需要对部分错误进行特殊处理，而不是所有错误都直接崩溃。
     * ```rs
     * let f = File::open("hello.txt");
     * let f = match f {
     *     Ok(file) => file,
     *     Err(error) => match error.kind() {
     *         ErrorKind::NotFound => match File::create("hello.txt") {
     *             Ok(fc) => fc,
     *             Err(e) => panic!("Problem creating the file: {:?}", e),
     *         },
     *         other_error => panic!("Problem opening the file: {:?}", other_error),
     *     },
     * };
     * ```
     * 上面代码在匹配出 error 后，又对 error 进行了详细的匹配解析，最终结果：
     * - 如果是文件不存在错误 ErrorKind::NotFound，就创建文件，这里创建文件File::create 也是返回 Result，因此继续用 match 对其结果进行处理：创建成功，将新的文件句柄赋值给 f，如果失败，则 panic。
     * - 剩下的错误，一律 panic
     *
     * ### 3. 何时该使用 `panic!` 和  `Result` ？
     * 先来一点背景知识，在前面章节我们粗略讲过 Result<T, E> 这个枚举类型，它是用来表示函数的返回结果：
     * ```rs
     * enum Result<T, E> {
     *     Ok(T),
     *     Err(E),
     * }
     * ```
     * 当没有错误发生时，函数返回一个用 Result 类型包裹的值 Ok(T)，当错误时，返回一个 Err(E)。对于 Result 返回我们有很多处理方法，最简单粗暴的就是 unwrap 和 expect，这两个函数非常类似，我们以 unwrap 举例：
     * ```rs
     * use std::net::IpAddr;
     * let home: IpAddr = "127.0.0.1".parse().unwrap();
     * ```
     * 上面的 parse 方法试图将字符串 "127.0.0.1" 解析为一个 IP 地址类型 IpAddr，它返回一个 Result<IpAddr, E> 类型，如果解析成功，则把 Ok(IpAddr) 中的值赋给 home，如果失败，则不处理 Err(E)，而是直接 panic。
     * 因此 unwrap 简而言之：**成功则返回值，失败则 panic**，总之不进行任何错误处理。
     *
     * 当错误预期会出现时，返回一个可处理的错误较为合适，当错误不可预期时，比如内存安全（数组越界），使用panic!更为合适。
     *
     * #### 1. 示例、原型、测试
     * 这几个场景下，需要快速地搭建代码，错误处理会拖慢编码的速度，也不是特别有必要，因此通过 unwrap、expect 等方法来处理是最快的。
     * 同时，当我们回头准备做错误处理时，可以全局搜索这些方法，不遗漏地进行替换。
     *
     * 简单来说，在示例，原型，测试这些快速开发的情况下，可以错误的处理，只使用正确的结果，也就是可以通过unwrap和expect两个函数来处理。
     *
     * #### 2. 确切程序正确
     * 你确切的知道你的程序是正确时，可以使用 panic。因为 panic 的触发方式比错误处理要简单，因此可以让代码更清晰，可读性也更加好。
     * 当我们的代码注定是正确时，你可以用 unwrap 等方法直接进行处理，反正也不可能 panic ：
     * ```rs
     * use std::net::IpAddr;
     * let home: IpAddr = "127.0.0.1".parse().unwrap();
     * ```
     * 例如上面的例子，"127.0.0.1" 就是 ip 地址，因此我们知道 parse 方法一定会成功，那么就可以直接用 unwrap 方法进行处理。
     * 当然，如果该字符串是来自于用户输入，那在实际项目中，就必须用错误处理的方式，而不是 unwrap！
     *
     * #### 3. 可能导致全局有害状态时
     * 有害状态大概分为几类：
     * - 非预期的错误
     * - 后续代码的运行会受到显著影响
     * - 内存安全的问题
     *
     * 当错误预期会出现时，返回一个可处理的错误较为合适，例如解析器接收到格式错误的数据，HTTP 请求接收到错误的参数甚至该请求内的任何错误（不会导致整个程序有问题，只影响该次请求）。因为错误是可预期的，因此也是可以处理的。
     * 当启动时某个流程发生了错误，对后续代码的运行造成了影响，那么就应该使用 panic，而不是处理错误后继续运行，当然你可以通过重试的方式来继续。
     * 而数组访问越界，就要 panic 的原因，这个就是属于内存安全的范畴，一旦内存访问不安全，那么我们就无法保证自己的程序会怎么运行下去，也无法保证逻辑和数据的正确性。
     *
     * ### 4. unwrap 与 expect
     * 在不需要处理错误的场景，例如写原型、示例时，我们不想使用 match 去匹配 Result<T, E> 以获取其中的 T 值，因为 match 的穷尽匹配特性，你总要去处理下 Err 分支。那么有没有办法简化这个过程？有，答案就是 unwrap 和 expect。
     * 
     * expect 跟 unwrap 很像，也是遇到错误直接 panic, 但是会带上自定义的错误提示信息，相当于重载了错误打印的函数：
     * ```rs
     * let f = File::open("hello.txt").expect("Failed to open hello.txt");
     * ```
     * 因此，expect 相比 unwrap 能提供更精确的错误信息，在有些场景也会更加实用。
     * 
     * 
     * ### 4. panic! 原理解析
     *
     * [panic! 原理解析](https://course.rs/basic/result-error/panic.html#panic-%E5%8E%9F%E7%90%86%E5%89%96%E6%9E%90)
     *
     */

    // 被动触发不可恢复错误
    let arr = [1, 2, 3];
    // println!("{}", arr[99]); // 数组越界

    // 主动触发不可恢复错误
    panic!("主动触发错误信息");

    // 打开文件示例
    let file = File::open("./main 29-返回值和错误处理.rs");
    // 直接取值
    let file = File::open("./main 29-返回值和错误处理.rs").unwrap();
    println!("{:#?}", file);

    let f = File::open("hello.txt");

    let f = match f {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating the file: {:?}", e),
            },
            other_error => panic!("Problem opening the file: {:?}", other_error),
        },
    };
    println!("{:#?}", f);
}
