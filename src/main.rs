use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use lazy_static::lazy_static;

fn main() {
    /*
     *
     * ## 全局变量
     * 在一些场景，我们可能需要全局变量来简化状态共享的代码，包括全局 ID，全局数据存储等等。
     *
     * 全局变量是一种特殊的变量，在 rust 中相对复杂，但有一点可以肯定，全局变量的生命周期肯定是'static，但是不代表它需要用static来声明。
     *
     * 具体来说，全局变量分为编译期初始化和运行期初始化两种。
     *
     * **常量与普通变量的区别**
     * - 关键字是const而不是let
     * - 定义常量必须指明类型（如 i32），不能省略
     * - 定义常量时变量的命名规则一般是全部大写
     * - 对于变量出现重复的定义(绑定)会发生变量遮盖，后面定义的变量会遮住前面定义的变量，常量则不允许出现重复的定义
     * - 常量可以在任意作用域进行定义，其生命周期贯穿整个程序的生命周期。编译时编译器会尽可能将其内联到代码中，所以在不同地方对同一常量的引用并不能保证引用到相同的内存地址
     * - 编译期初始化常量的赋值只能是常量表达式/数学表达式，也就是说必须是在编译期就能计算出的值，如果需要在运行时才能得出结果的值比如函数，则不能赋值给常量表达式。即常量的赋值不能在程序运行时通过配置实现。
     *
     * ### 编译期初始化
     * 大多数使用的全局变量都只需要在编译期初始化，例如静态配置、计数器、状态值等等。
     *
     * #### 静态常量
     *
     * 全局常量可以在程序任何一部分使用，如果它是定义在某个模块中，则需要引入对应的模块才能使用。全局常量很适合用作静态配置：
     * ```rust
     * const MAX_ID: usize =  usize::MAX / 2;
     * println!("最大的用户 ID = {}", MAX_ID);
     * ```
     *
     * 常量可以在任意作用域中定义，编译时编译器会尽可能将其内联到代码中，所以在不同地方对同一常量的引用并不能保证引用到相同的内存地址。
     *
     *
     * #### 静态变量
     *
     * 静态变量允许声明一个全局的变量，常用于全局数据统计，例如统计总请求数：
     * ```rust
     * // 静态变量
     * static mut REQUEST_COUNT: usize = 0;
     * unsafe {
     *     // 操作 static 类型的变量需要 unsafe 模块
     *     // 因为这种使用方式往往并不安全，当在多线程中同时去修改时，会不可避免的遇到脏数据
     *     REQUEST_COUNT = 2;
     * }
     * unsafe {
     *     println!("REQUEST_COUNT = {}", REQUEST_COUNT);
     * }
     * ```
     * 操作 static 类型的变量需要 unsafe 作用域，因为这种使用方式往往并不安全，当在多线程中同时去修改时，会不可避免的遇到脏数据。
     *
     * 和常量相同，定义静态变量的时候必须赋值为在编译期就可以计算出的值(常量表达式/数学表达式)，不能是运行时才能计算出的值(如函数)，即不能通过程序运行时再配置定义静态变量。
     *
     * 编译期初始化错误的案例：
     * ```rust
     * static NAMES: Mutex<String> = Mutex::new(String::from("Sunface, Jack, Allen")); // 错误，静态变量不能通过函数在编译期初始化
     * ```
     *
     *
     * #### 静态变量和常量的区别
     *
     * - 静态变量不会被内联，在整个程序中，静态变量只有一个实例，所有的引用都会指向同一个地址
     * - 为了能在多线程中正常使用，存储在静态变量中的值必须要实现 Sync trait
     *
     * #### 原子类型
     * 原子类型是多线程共享数据的线程安全的最好方式之一：
     * ```rust
     * // 原子类型是共享状态最好的一种方式
     * static REQUEST_RECV: AtomicUsize = AtomicUsize::new(0);
     * let mut handles: Vec<JoinHandle<()>> = Vec::new();
     * for i in 0..100 {
     *     handles.push(thread::spawn(move || {
     *         REQUEST_RECV.fetch_add(i, Ordering::SeqCst);
     *     }));
     * }
     * for h in handles {
     *     h.join().unwrap();
     * }
     * println!("REQUEST_RECV = {}", REQUEST_RECV.load(Ordering::SeqCst));
     * ```
     *
     * ### 运行期初始化
     * 为什么需要运行期初始化呢？常见的场景是：一个全局的动态配置，它在程序开始后，才加载数据进行初始化，最终可以让各个线程直接访问使用。
     *
     * 编译期初始化最大的限制是必须赋值为在编译期就可以计算出的值(常量表达式/数学表达式)，不能是运行时才能计算出的值(如函数)：
     * ```rust
     * static NAMES: Mutex<String> = Mutex::new(String::from("Hello World")); // 错误，静态变量不能通过函数在编译期初始化
     * ```
     *
     * 因为 Rust 的**借用和生命周期规则的限制**，如果需要在运行期初始化一个全局变量，就需要考虑 `lazy_static`、`Box::leak` 等方式。
     *
     * #### lazy_static
     * lazy_static 是社区提供的非常强大的宏，用于懒初始化静态变量，之前的静态变量都是在编译期初始化的，因此无法使用函数调用进行赋值，而 lazy_static 允许在运行期初始化静态变量！
     *
     * lazy_static 宏，匹配的是 static ref，所以定义的静态变量都是不可变引用。
     *
     * ```rust
     * // 使用 lazy_static 在运行期初始化一个全局变量
     * lazy_static! {
     *     static ref NAMES: Mutex<String> = Mutex::new(String::from("Hello"));
     * }
     * {
     *     let mut names = NAMES.lock().unwrap();
     *     names.push_str(" World");
     *     names.push('!');
     * }
     * println!("NAMES = {:?}", NAMES.lock().unwrap());
     * ```
     *
     * 需要注意的是，lazy_static 直到运行到 main 中的第一行代码时，才进行初始化。
     * 并且使用 lazy_static 在每次访问静态变量时，会有轻微的性能损失，因为其内部实现用了一个底层的并发原语 std::sync::Once，在每次访问该变量时，程序都会执行一次原子指令用于确认静态变量的初始化是否完成。
     *
     * #### Box::leak
     * 在正常的生命周期中，rust 是不允许将一个只有局部生命周期的变量赋值给 `'static` 全局的生命周期，因为这容易造成访问未定义的行为，非常不安全。
     *
     * 通过 `Box::leak` 将一个变量从内存中泄露，使其成为 `'static` 生命周期，这样就可以赋值给 `'static` 生命周期的全局变量，也就能达到在运行时初始化全局变量的目的。
     * ```rust
     * // Box::leak 将变量从内存中泄露出去，使其成为 'static 的生命周期，这样就可以赋值给具有 'static 生命周期的全局变量，也就能达到在运行时初始化全局变量的目的
     * #[derive(Debug)]
     * struct Config {
     *     secret: String,
     * }
     * static mut CONFIG: Option<&mut Config> = None;
     *
     * let config = Box::new(Config {
     *     secret: String::from("Hello World"),
     * });
     * unsafe {
     *     CONFIG = Some(Box::leak(config));
     *     println!("{:?}", CONFIG);
     * }
     * ```
     *
     * #### 函数返回全局变量
     * 借助 Box::leak 通过借用和生命周期规则校验，将一个变量变为 `'static`，保证与程序活的一样久即可：
     * ```rust
     * // 函数返回一个全局变量，借助 Box::leak 即可
     * fn init_static() -> &'static mut Config {
     *     Box::leak(Box::new(Config {
     *         secret: String::from("None DO"),
     *     }))
     * }
     * unsafe {
     *     CONFIG = Some(init_static());
     *     println!("{:?}", CONFIG);
     * }
     * ```
     *
     * ### OnceCell 和 OnceLock
     * 在 Rust 标准库中提供了实验性的 lazy::OnceCell 和 lazy::SyncOnceCell (在 Rust 1.70.0 版本及以上的标准库中，替换为稳定的 cell::OnceCell 和 sync::OnceLock)两种 Cell。
     * 前者用于单线程，后者用于多线程，它们用来存储堆上的信息，并且具有最多只能赋值一次的特性。 
     * 
     * 阅读：https://course.rs/advance/global-variable.html#标准库中的-oncecell
     * 
     * ### 总结
     * - 编译期初始化的全局变量，const创建常量，static创建静态变量，Atomic创建原子类型
     * - 运行期初始化的全局变量，lazy_static用于懒初始化，Box::leak利用内存泄漏将一个变量的生命周期变为'static
     *
     *
     *
     */

    //  静态常量
    const MAX_ID: usize = usize::MAX / 2;
    println!("最大的用户 ID = {}", MAX_ID);
    // 静态变量不能通过函数在编译期初始化
    // static NAMES: Mutex<String> = Mutex::new(String::from("Hello World"));

    // 静态变量
    static mut REQUEST_COUNT: usize = 0;
    unsafe {
        // 操作 static 类型的变量需要 unsafe 作用域
        // 因为这种使用方式往往并不安全，当在多线程中同时去修改时，会不可避免的遇到脏数据
        REQUEST_COUNT = 2;
    }
    unsafe {
        println!("REQUEST_COUNT = {}", REQUEST_COUNT);
    }

    // 原子类型是共享状态最好的一种方式
    static REQUEST_RECV: AtomicUsize = AtomicUsize::new(0);
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..100 {
        handles.push(thread::spawn(move || {
            REQUEST_RECV.fetch_add(i, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("REQUEST_RECV = {}", REQUEST_RECV.load(Ordering::SeqCst));

    // 使用 lazy_static 在运行期初始化一个全局变量
    lazy_static! {
        static ref NAMES: Mutex<String> = Mutex::new(String::from("Hello"));
    }
    {
        let mut names = NAMES.lock().unwrap();
        names.push_str(" World");
        names.push('!');
    }
    println!("NAMES = {:?}", NAMES.lock().unwrap());

    // Box::leak 将变量从内存中泄露出去，使其成为 'static 的生命周期，这样就可以赋值给具有 'static 生命周期的全局变量，也就能达到在运行时初始化全局变量的目的
    #[derive(Debug)]
    struct Config {
        secret: String,
    }
    static mut CONFIG: Option<&mut Config> = None;

    let config = Box::new(Config {
        secret: String::from("Hello World"),
    });
    unsafe {
        CONFIG = Some(Box::leak(config));
        println!("{:?}", CONFIG);
    }

    // 函数返回一个全局变量，借助 Box::leak 即可
    fn init_static() -> &'static mut Config {
        Box::leak(Box::new(Config {
            secret: String::from("None DO"),
        }))
    }
    unsafe {
        CONFIG = Some(init_static());
        println!("{:?}", CONFIG);
    }
}
