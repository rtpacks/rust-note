use std::{pin::Pin, rc::Rc};

use futures::{executor, Future};
use tokio::runtime::{Builder, Runtime};

fn main() {
    /*
     *
     * ## async 异步编程：一些疑难问题
     * async 在 Rust 中属于比较新的概念，因此存在一些解决方案不完善的地方。
     *
     * ### async 语句块使用 ?
     * async 语句块和 async fn 最大的区别就是 async 语句块无法显式的声明返回值，当配合 `?`（错误传播）一起使用时就会有类型错误。
     * 原因在于编译器无法推断出 `Result<T, E>` 中的 E 的类型。
     *
     * 有些时候编译器还会提示 consider giving `fut` a type，这个也不需要尝试，因为目前还没有办法为 async 语句块指定返回类型。
     * ```rust
     * async fn foo() -> Result<i32, String> {
     *     Ok(1)
     * }
     * async fn bar() -> Result<i32, String> {
     *     Ok(2)
     * }
     * let async_block = async {
     *     // the `?` operator can only be used in an async block that returns `Result` or `Option` (or another type that implements `FromResidual`)
     *     //  `?` 必须要在返回 Result 和 Option 的代码块中使用
     *     let foo = foo().await?;
     *     let bar = bar().await?;
     *     println!("async_block exec, foo = {}, bar = {}", foo, bar);
     *     // Ok(()) 编译器目前无法推断错误类型，需要手动指定类型，例如下一行使用 turbofish 声明错误类型
     *     Ok::<_, String>(())
     * };
     * executor::block_on(async_block);
     * ```
     *
     * ### async 函数和 Send 特征
     * 在多线程中，Send 和 Sync 是绕不开的两个标记特征，要在多线程中安全的使用变量，就必须使用 Send 或 Sync：
     * - 实现 Send 的类型可以在线程间安全地传递所有权
     * - 实现 Sync 的类型可以在线程间安全地共享引用
     *
     * 由于 rust async/await 的调度，Future 可能运行在不同的线程上，由于多线程需要保证数据的所有权和引用的正确性。
     * 所以当处于多线程时 Future 需要关注 **.await 运行过程中**，传递给 Future 作用域的变量类型是否是 Send。
     *
     * 在单线程上，不需要实现 Send 也可以正常运行，使用 `futures::executor::block_on` 构建单线程运行环境：
     * ```rust
     * // !Send 特征
     * #[derive(Default)]
     * struct NotSend(Rc<()>);
     *
     * async fn foo(x: NotSend) {
     *     println!("{:?}", x.0);
     * }
     * async fn bar() {
     *     let x = NotSend::default();
     *     foo(x).await;
     * }
     * executor::block_on(bar());
     * ```
     *
     * 使用 tokio 构建多线程运行环境，运行携带实现 `!Send` 特征的数据类型：
     * ```rust
     * // !Send 特征
     * async fn foo(x: Rc<i32>) {
     *     println!("{:?}", x);
     * }
     * async fn bar() {
     *     let x = Rc::new(2);
     *     foo(x).await;
     * }
     * // !Send 特征不满足多线程运行的要求，编译报错
     * let rt = Runtime::new().unwrap();
     * let handle = rt.spawn(bar());
     * rt.block_on(handle);
     * ```
     *
     * 如果 await 在不涉及非 `!Send` 的情况下 rust 编译器报错，可以利用代码块将 `!Send` 类型包裹起来，这个规则可以帮助解决很多借用冲突问题，特别是在 NLL 出来之前。
     *
     * ### async 与递归
     * 递归涉及到动态大小类型（不定长类型 DST），它的大小只有到了**程序运行时**才能动态获知。
     * rust 编译器不允许直接使用动态大小类型，在之前的章节【unit 49-不定长类型 DST 和定长类型 Sized】与【智能指针（二）Box 对象分配】中处理不定长类型的方式就是将其转换成定长类型，如将特征转换成特征对象。
     *
     * > 关键点回忆1：
     * >
     * > 不能简单的将变量与类型视为只是一块栈内存或一块堆内存数据，比如 Vec 类型，rust将其分成两部分数据：存储在堆中的实际类型数据与存储在栈上的管理信息数据。
     * > 其中存储在栈上的管理信息数据是引用类型，包含实际类型数据的地址、元素的数量，分配的空间等信息，**rust 通过栈上的管理信息数据掌控实际类型数据的信息**。
     * > 这种**存储自身大小信息的类型**就可以称为定长类型（固定尺寸）。
     *
     *
     * > 关键点回忆2：
     * >
     * > 特征是一种动态尺寸类型（Dynamically Sized Types，DST），即特征本身不具有固定的大小，因此不能直接实例化为对象。
     * > 在Rust中，特征通常通过指针（如 `Box<T>、&T`）来使用，这些指针指向实现了该特征的具体类型的实例。
     * > 这些**对动态尺寸类型的一种封装，使其可以通过具体的、已知大小的指针类型（如 `Box<dyn Trait>` 或 `&dyn Trait`）来使用，这种封装类型就是一个特征对象**。因此特征对象可以被视为具体的、已知大小的类型。
     * > 在这里需要更新前几章的描述：特征对象是动态尺寸类型，这是有误的。正确的认识是：特征是动态尺寸类型，而特征对象是对特征的一种封装，使特征可以通过具体的，已知大小的指针类型来描述，因此特征对象是一个定长类型（Sized）。
     *
     * 在内部实现中，async fn 被编译成一个 Future 状态机，在递归中使用 async fn 会将流程变得更为复杂，因为编译后的状态机还需要包含自身。
     * 这是典型的动态大小类型，它的大小会无限增长，因此编译器会直接报错。
     * ```rust
     * // foo函数:
     * async fn foo() {
     *     step_one().await;
     *     step_two().await;
     * }
     *
     * // 会被编译成类似下面的类型：
     * enum Foo {
     *     First(StepOne),
     *     Second(StepTwo),
     * }
     *
     * // 因此 recursive 函数
     * async fn recursive() {
     *     recursive().await;
     *     recursive().await;
     * }
     *
     * // 会生成类似以下的类型
     * enum Recursive {
     *     First(Recursive),
     *     Second(Recursive),
     * }
     * ```
     *
     * 之前将递归不定长类型变为定长类型是通过 box 智能指针来实现的，但是在递归中不能使用 box 来实现。
     *
     * 编译器会将 async fn 编译为一个匿名的类型，该类型实现了 Future。当尝试直接使用 Box::pin 包裹一个 async fn 时，会遇到编译器的限制，主要是因为生命周期和自引用的问题。
     *
     * 在 rust 1.79 中，简单的递归已经可以直接使用 Box::pin，但是建议还是通过将递归逻辑提取到一个**普通函数**中，并返回一个 `Box<dyn Future<Output = T> + Send>` 解决问题。
     *
     * ```rust
     * // 将递归逻辑抽离到普通函数，显式的返回 `Box<dyn Future<Output = T> + Send>` 类型
     * fn fibonacci_impl(n: u32) -> Pin<Box<dyn Future<Output = u64> + Send>> {
     *     Box::pin(async move {
     *         match n {
     *             0 => 0,
     *             1 => 1,
     *             _ => {
     *                 let a = fibonacci_impl(n - 1).await;
     *                 let b = fibonacci_impl(n - 1).await;
     *                 a + b
     *             }
     *         }
     *     })
     * }
     * async fn fibonacci() {
     *     let res = fibonacci_impl(4).await;
     *     println!("{res}");
     * }
     * executor::block_on(fibonacci());
     * ```
     *
     * ### async 与 trait
     *
     * 从 1.75 版本开始，rust 支持在 trait 中定义 async 异步函数，不再需要通过 async-trait 库来支持。具体来看，trait async 分为两种：
     * - 有代价 async trait
     * - 无代价的 async trait
     * https://blog.csdn.net/qq_54714089/article/details/137723868
     */

    {
        async fn foo() -> Result<i32, String> {
            Ok(1)
        }
        async fn bar() -> Result<i32, String> {
            Ok(2)
        }
        let async_block = async {
            // the `?` operator can only be used in an async block that returns `Result` or `Option` (or another type that implements `FromResidual`)
            // `?` 必须要在返回 Result 和 Option 的代码块中使用
            let foo = foo().await?;
            let bar = bar().await?;
            println!("async_block exec, foo = {}, bar = {}", foo, bar);
            // Ok(())
            Ok::<_, String>(())
        };
        executor::block_on(async_block);
    }

    {
        // !Send 类型
        async fn foo(x: Rc<i32>) {
            println!("{:?}", x);
        }
        async fn bar() {
            let x = Rc::new(2);
            foo(x).await;
        }
        executor::block_on(bar());
    }

    {
        // !Send 类型
        async fn foo(x: Rc<i32>) {
            println!("{:?}", x);
        }
        async fn bar() {
            let x = Rc::new(2);
            foo(x).await;
        }
        let rt = Runtime::new().unwrap();
        // !Send 特征不满足多线程运行的要求，编译报错
        // let handle = rt.spawn(bar());
        // rt.block_on(handle);
    }

    {
        async fn recursive_function(n: u32) {
            if n == 0 {
                println!("Reached the base case");
                return;
            }
            println!("Current value: {}", n);
            Box::pin(recursive_function(n - 1)).await;
        }
        executor::block_on(recursive_function(10));

        async fn fibonacci(n: u32) -> u64 {
            println!("Current n: {}", n);
            match n {
                0 => 0,
                1 => 1,
                _ => {
                    let a = Box::pin(fibonacci(n - 1)).await;
                    let b = Box::pin(fibonacci(n - 2)).await;
                    a + b
                }
            }
        }
        async fn run() {
            let res = fibonacci(4).await;
            println!("{res}");
        }
        executor::block_on(run());
    }

    {
        // 将递归逻辑抽离到普通函数，显式的返回 `Box<dyn Future<Output = T> + Send>` 类型
        fn fibonacci_impl(n: u32) -> Pin<Box<dyn Future<Output = u64> + Send>> {
            Box::pin(async move {
                match n {
                    0 => 0,
                    1 => 1,
                    _ => {
                        let a = fibonacci_impl(n - 1).await;
                        let b = fibonacci_impl(n - 1).await;
                        a + b
                    }
                }
            })
        }
        async fn fibonacci() {
            let res = fibonacci_impl(4).await;
            println!("{res}");
        }
        executor::block_on(fibonacci());
    }
}
