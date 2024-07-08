use futures::{channel::mpsc, join, try_join, SinkExt, StreamExt, TryFutureExt};

fn main() {
    /*
     *
     * ## async 异步编程：join! 和 select!
     *
     * 在 JavaScript 中，如果希望同时运行多个 async 并在对应时机处理 async 结果，一般会使用 `Promise.all` 或 `Promise.allSettled` 方法。
     * 在 Rust 中，也有同时运行多个 async 的方法，例如 `join!` 和 `select!`。
     *
     * ### join!
     * `join!` 类似 `Promise.allSettled` 方法，它会等待所有的 Future 执行结束（成功、失败），然后再返回结果。
     * 当然 `Promise.allSettled` 也需要经过 `await Promise.allSettled` 才能在同步的逻辑中编写异步的代码：
     * ```rust
     * async fn join1() {
     *     println!("join1");
     * }
     * async fn join2() {
     *     println!("join2");
     * }
     * async fn join3() {
     *     // join 需要在 async 函数中使用
     *     join!(join1(), join2());
     *     println!("join3");
     * }
     * futures::executor::block_on(join3());
     * ```
     *
     * 特别注意，以下写法是串行的：
     * ```rust
     * async fn join4() {
     *     // 注意，这种写法是串行的
     *     (join1().await, join2().await);
     * }
     * ```
     *
     * 如果希望某一个 Future 报错后就立即停止所有 Future 的执行，可以使用 try_join!，功能上类似 `Promise.all` 函数。
     *
     * 注意，因为 try_join! 在某一个 Future 报错后就会停止所有的 Future 并返回 Error，所以 try_join! 并发的 Future 需要返回同一个 Error 类型。
     * 这样无论哪一个 Future 发生错误，错误类型也是匹配的。如果错误类型不同，可以考虑使用来自 futures::future::TryFutureExt 模块的 map_err 和 err_into 方法将错误进行转换。
     *
     * 使用方法与 join! 类似，也需要在 async 函数中使用 try_join!。
     * ```rust
     * async fn try_join1() -> Result<(), String> {
     *     println!("try_join1");
     *     Ok(())
     * }
     * async fn try_join2() -> Result<(), i32> {
     *     println!("try_join2");
     *     Ok(())
     * }
     * async fn try_join3() {
     *     // try_join 需要在 async 函数中使用，并且需要统一错误类型
     *     // 如果错误类型不同，可以考虑使用来自 futures::future::TryFutureExt 模块的 map_err 和 err_info 方法将错误进行转换。
     *     let fut_1 = try_join1();
     *     let fut_2 = try_join2().map_err(|num| num.to_string());
     *     try_join!(fut_1, fut_2);
     *     println!("try_join3");
     * }
     * futures::executor::block_on(try_join3());
     * ```
     *
     * 如果希望以数组的形式传递多个 Future，可以使用 `futures::future::join_all` 方法。
     */

    async fn join1() {
        println!("join1");
    }
    async fn join2() {
        println!("join2");
    }
    async fn join3() {
        // join 需要在 async 函数中使用
        join!(join1(), join2());
        println!("join3");
    }
    futures::executor::block_on(join3());

    async fn join4() {
        // 注意，这种写法是串行的
        (join1().await, join2().await);
    }

    async fn try_join1() -> Result<(), String> {
        println!("try_join1");
        Ok(())
    }
    async fn try_join2() -> Result<(), i32> {
        println!("try_join2");
        Ok(())
    }
    async fn try_join3() {
        // try_join 需要在 async 函数中使用，并且需要统一错误类型
        // 如果错误类型不同，可以考虑使用来自 futures::future::TryFutureExt 模块的 map_err 和 err_into 方法将错误进行转换。
        let fut_1 = try_join1();
        let fut_2 = try_join2().map_err(|num| num.to_string());
        try_join!(fut_1, fut_2);
        println!("try_join3");
    }
    futures::executor::block_on(try_join3());
}
