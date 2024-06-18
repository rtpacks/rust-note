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
     * ## 转换和边界异常处理
     *
     * Result 和 Option 在业务程序中很常见，并且通常还需要自定义错误类型以便快速定位问题。
     *
     * ### 组合器
     *
     * 因为 Result 和 Option 类型很常见，但使用真实值时需要取出再判断显得比较琐碎，所以 rust 提供了一些组合器简化这些操作。
     *
     * 组合器不同于组合模式，组合器更多的是用于对返回结果的类型进行变换：例如使用 ok_or 将一个 Option 类型转换成 Result 类型。
     * > 组合模式：将对象组合成树形结构以表示“部分整体”的层次结构。组合模式使得用户对单个对象和组合对象的使用具有一致性。–GoF \<\<设计模式\>\>
     *
     * ```rust
     * let id: Option<i32> = Some(1);
     * let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
     * println!("{}", id.unwrap());
     * ```
     *
     *
     *
     *
     */

    let id: Option<i32> = Some(1);
    let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
    println!("{}", id.unwrap());
}
