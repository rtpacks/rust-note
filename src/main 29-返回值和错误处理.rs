use core::panic;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, Read};

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
     * ### 5. 传播错误
     *
     * #### 1. 错误传播示例
     * 程序不太可能只有 A->B 形式的函数调用，一个设计良好的程序，一个功能往往涉及十几层的函数的调用。
     * 错误处理也往往不是哪里调用出错，就在哪里处理。实际应用中，大概率会把错误层层上传然后交给调用链的上游函数进行处理，**错误传播**将极为常见。
     *
     * ```rs
     * use std::fs::File;
     * use std::io::{self, Read};
     *
     * fn read_username_from_file() -> Result<String, io::Error> {
     *     // 打开文件，f是`Result<文件句柄,io::Error>`
     *     let f = File::open("hello.txt");
     *
     *     let mut f = match f {
     *         // 打开文件成功，将file句柄赋值给f
     *         Ok(file) => file,
     *         // 打开文件失败，将错误返回(向上传播)
     *         Err(e) => return Err(e),
     *     };
     *     // 创建动态字符串s
     *     let mut s = String::new();
     *     // 从f文件句柄读取数据并写入s中
     *     match f.read_to_string(&mut s) {
     *         // 读取成功，返回Ok封装的字符串
     *         Ok(_) => Ok(s),
     *         // 将错误向上传播
     *         Err(e) => Err(e),
     *     }
     * }
     * ```
     * 可以先不用考虑mut等的实现，只需要注意以下几点：
     * - 该函数返回一个 Result<String, io::Error> 类型，当读取用户名成功时，返回 Ok(String)，失败时，返回 Err(io:Error)
     * - File::open 和 f.read_to_string 返回的 Result<T, E> 中的 E 就是 io::Error
     *
     * 由此可见，该函数将 io::Error 的错误往上进行传播，该函数的调用者最终会对 Result<String,io::Error> 进行再处理，至于怎么处理就是调用者的事。
     * 如果是错误，它可以选择继续向上传播错误，也可以直接 panic，亦或将具体的错误原因包装后写入 socket 中呈现给终端用户。
     *
     * #### 2. 传播错误的简写：? 运算符
     * 上面的 `read_username_from_file` 函数的match模式匹配中，有很多简单但是又不得不写的逻辑：成功返回值，失败返回错误原因。
     * 如果结果是 Ok(T)，则把 T 赋值给 f，如果结果是 Err(E)，则返回该错误，所以 ? 特别适合用来传播错误（**return 关键字！**）。
     *
     * ```rs
     * fn read_username_from_file() -> Result<String, io::Error> {
     *     let mut f = File::open("hello.txt")?;
     *     let mut s = String::new();
     *     f.read_to_string(&mut s)?;
     *     Ok(s)
     * }
     * ```
     *
     * 其实 ? 就是一个宏，它的作用跟上面的 match 几乎一模一样：
     * ```rs
     * let mut f = File::open("hello.txt")?; // ?
     *
     * let mut f = match f {
     *     // 打开文件成功，将file句柄赋值给f
     *     Ok(file) => file,
     *     // 打开文件失败，将错误返回(向上传播)
     *     Err(e) => return Err(e),
     * };
     * ```
     *
     * #### 3. `?` 的优势
     * 虽然 ? 和 match 功能一致，但是事实上 ? 会更胜一筹。
     * 1. 类型提升/转换
     * 想象一下，一个设计良好的系统中，肯定有自定义的错误特征，错误之间很可能会存在上下级关系。
     * 例如标准库中的 std::io::Error 和 std::error::Error，前者是 IO 相关的错误结构体，后者是一个最最通用的标准错误特征，同时前者**实现**了后者，因此 std::io::Error 可以转换为 std:error::Error。
     * 明白了以上的错误转换，? 的更胜一筹就很好理解了，它可以**自动进行类型提升**（转换）：
     *
     * ```rs
     * fn open_file() -> Result<File, Box<dyn std::error::Error>> {
     *     let mut f = File::open("hello.txt")?;
     *     Ok(f)
     * }
     * ```
     * File::open 报错时返回的错误是 std::io::Error 类型，但是 open_file 函数返回的错误类型是 std::error::Error 的特征对象，可以看到一个错误类型通过 ? 返回后，变成了另一个错误类型，这就是 ? 的神奇之处。
     *
     * `?` 能自动转换的根本原因是：标准库中定义的 From 特征，该特征有一个方法 from，用于把一个类型转成另外一个类型。? 可以自动调用该方法，然后进行隐式类型转换。
     * 因此只要函数返回的错误 ReturnError 实现了 From<OtherError> 特征，那么 ? 就会自动把 OtherError 转换为 ReturnError。
     *
     * 这种转换非常好用，意味着你可以用一个大而全的 ReturnError 来覆盖所有错误类型，只需要为各种子错误类型实现这种转换即可。
     *
     * 2. 链式调用
     * ```rs
     * fn read_username_from_file() -> Result<String, io::Error> {
     *     let mut s = String::new();
     *     File::open("hello.txt")?.read_to_string(&mut s)?;
     *     Ok(s)
     * }
     * ```
     * 除了支持自动类型提升外，`?` 还支持链式调用。如以上代码，File::open 遇到错误就返回，没有错误就将 Ok 中的值取出来用于下一个方法调用，
     *
     * 3. Option传播
     * 除了Result传播外，`?` 还可以用于Option传播。Result 通过 ? 返回**值或者错误**，Option 通过 ? 返回**值或者None**。注意是值而不是Result或Option
     * ```rs
     * pub enum Option<T> {
     *     Some(T),
     *     None
     * }
     * // 简化match
     * fn first(arr: &[i32]) -> Option<&i32> {
     *     let v = arr.get(0)?;
     *     Some(v)
     * }
     * // 链式调用
     * fn last_char_of_first_line(text: &str) -> Option<char> {
     *     text.lines().next()?.chars().last()
     * }
     * ```
     *
     * 在使用 `?` 的过程中，常常会犯直接取值返回的错误，需要记住：**`?` 直接取出并返回的是值，打断当前执行并返回的是错误**：
     * ```rs
     * // 常见错误
     * fn first3(arr: &[i32]) -> Option<&i32> {
     *     // arr.get(0)? 直接返回值或错误，而不是返回Option包裹的值
     *     Some(arr.get(0)?) // 正确
     * }
     * fn first4(arr: &[i32]) -> Option<String> {
     *     // arr.get(0)?.to_string() 直接返回值
     *     Some(arr.get(0)?.to_string()) // 正确
     * }
     * ```
     * 总结：**`?` 直接取出并返回的是值，打断当前执行并返回的是错误**
     *
     * ### 6. panic! 原理解析
     *
     * [panic! 原理解析](https://course.rs/basic/result-error/panic.html#panic-%E5%8E%9F%E7%90%86%E5%89%96%E6%9E%90)
     *
     * ### 7. 拓展阅读
     * - [带返回值的 main 函数](https://course.rs/basic/result-error/result.html#%E5%B8%A6%E8%BF%94%E5%9B%9E%E5%80%BC%E7%9A%84-main-%E5%87%BD%E6%95%B0)
     * - [try!](https://course.rs/basic/result-error/result.html#try)
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

    // 错误传播
    fn read_username_from_file() -> Result<String, io::Error> {
        // 打开文件，f是`Result<文件句柄,io::Error>`
        let f = File::open("hello.txt");

        let mut f = match f {
            // 打开文件成功，将file句柄赋值给f
            Ok(file) => file,
            // 打开文件失败，将错误返回(向上传播)
            Err(e) => return Err(e),
        };
        // 创建动态字符串s
        let mut s = String::new();
        // 从f文件句柄读取数据并写入s中
        match f.read_to_string(&mut s) {
            // 读取成功，返回Ok封装的字符串
            Ok(_) => Ok(s),
            // 将错误向上传播
            Err(e) => Err(e),
        }
    }

    fn read_from_file() -> Result<String, io::Error> {
        let f = File::open("hello.txt");

        let mut f = match f {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        // 创建动态字符串
        let mut s = String::new();

        match f.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(e) => Err(e),
        }
    }

    // 使用?简化错误传播形式
    fn read_username_from_file2() -> Result<String, io::Error> {
        // 创建动态字符串s
        let mut s = String::new();
        // 打开文件，f是`Result<文件句柄,io::Error>`
        let mut f = File::open("hello.txt")?;
        // 从f文件句柄读取数据并写入s中
        f.read_to_string(&mut s)?;
        Ok(s)
    }

    // ? 支持链式调用，简化形式
    fn read_username_from_file3() -> Result<String, io::Error> {
        let mut s = String::new();
        File::open("hello.txt")?.read_to_string(&mut s)?;
        Ok(s)
    }

    // 支持Option
    fn first(arr: &[i32]) -> Option<&i32> {
        arr.get(0)
    }
    // 链式
    fn first2(arr: &[i32]) -> Option<String> {
        Some(arr.get(0)?.to_string())
    }
    // 常见错误
    fn first3(arr: &[i32]) -> Option<&i32> {
        // arr.get(0)? 直接返回值或错误，而不是返回Option包裹的值
        Some(arr.get(0)?) // 正确
    }
    fn first4(arr: &[i32]) -> Option<String> {
        // arr.get(0)?.to_string() 直接返回值
        Some(arr.get(0)?.to_string()) // 正确
    }
}
