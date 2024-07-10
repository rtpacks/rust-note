use std::rc::Rc;

use futures::executor;
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
     *
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

    {}
}
