use std::{cell::Cell, rc::Rc, sync::Arc, thread};

use ilearn::{run, Config};

fn main() {
    /*
     * ## 内部可变性的 Cell 与 RefCell
     * Rust 通过严格的规则来保证所有权和借用的正确性，这带来安全提升的同时，损失了灵活性，比如结构体可变必须要求结构体所有字段可变。
     *
     * 这是由于 Rust 的 mutable 特性，一个结构体中的字段，要么全都是 immutable，要么全部是 mutable，**不支持针对部分字段进行设置**。
     * 比如，在一个 struct 中，可能只有个别的字段需要修改，其他字段并不需要修改，为了一个字段而将整个 struct 变为 `&mut` 是不合理的。
     *
     * rust提供实现了**内部可变性** Cell 和 RefCell 解决这类问题，通过**内部可变性**可以实现 struct 部分字段可变，而不用将整个 struct 设置为 mutable。
     * > 内部可变性的实现是因为 Rust 使用了 unsafe 来做到这一点，但是对于使用者来说，这些都是透明的，因为这些不安全代码都被封装到了安全的 API 中。
     * 简而言之，**可以在拥有不可变引用的同时修改目标数据**。
     *
     * ### Cell
     * Cell 和 RefCell 在功能上没有区别，区别在于 `Cell<T>` 适用于 T 实现 Copy 特征的情况：
     * ```rust
     * //  use std::cell::Cell;
     * let s_cell = Cell::new("Hello World");
     * let s = s_cell.get(); // 获取内部数据
     * s_cell.set("Hi"); // 不可变引用直接修改内部数据
     * println!("{s_cell:?}, {s}");
     * ```
     * 
     * 以上代码展示了 Cell 的基本用法，有几点值得注意：
     * - "Hello World" 是 `&str` 类型，它实现了 Copy 特征
     * - get 用来取值，set 用来设置新值
     * 
     * 取到值保存在 s 变量后，还能同时进行修改，这个违背了 Rust 的借用规则，但是由于实现了内部可变性的结构体 Cell 的存在，可以优雅地做到用不可变引用修改目标数据。
     * 
     * Cell适用于实现Copy的类型，如果尝试在 Cell 中存放 String，编译器会立刻报错，这是因为 `String` 没有实现 Copy 特征：
     * ```rust
     * let c = Cell::new(String::from("asdf")); 错误，String没有实现Copy特征
     * ```
     *
     * 如果是自定义的结构体实现，会发现safe代码中不能实现在拥有不可变引用的情况下修改数据。因为这与方法接收者的类型不一致，不可变引用不能调用可变引用的方法（点操作符的隐式转换）：
     * ```rust
     * struct MyCell<T: Copy> {
     *     value: T,
     * }
     * impl<T: Copy> MyCell<T> {
     *     fn new(v: T) -> MyCell<T> {
     *         MyCell { value: v }
     *     }
     *     fn set(&mut self, v: T) {
     *         self.value = v;
     *     }
     * }
     * let my_cell = MyCell::new("Hello World");
     * my_cell.set("Hi"); 错误，set函数 `set(&mut self, v: T)` 要求接收者是可变引用 `self: &mut Self`，而此时的 `my_cell` 是一个不可变引用。
     * ```
     *
     */

    //  use std::cell::Cell;
    let s_cell = Cell::new("Hello World");
    let s = s_cell.get(); // 获取内部数据
    s_cell.set("Hi"); // 不可变引用直接修改内部数据
    println!("{s_cell:?}, {s}");

    struct MyCell<T: Copy> {
        value: T,
    }
    impl<T: Copy> MyCell<T> {
        fn new(v: T) -> MyCell<T> {
            MyCell { value: v }
        }
        fn set(&mut self, v: T) {
            self.value = v;
        }
    }
    let my_cell = MyCell::new("Hello World");
    // my_cell.set("Hi"); 错误，set函数 `set(&mut self, v: T)` 要求接收者是可变引用 `self: &mut Self`，而此时的 `my_cell` 是一个不可变引用。
}
