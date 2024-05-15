use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
    thread,
};

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
     * ### RefCell
     * 在实际开发中，程序操作的更多是一个复杂数据类型，如多字段深层结构体。Cell 适用于 实现了 Copy 特征的类型，显然当复杂类型没有实现 Copy 时就需要另外一个内部可变性的工具来代替 Cell。
     * rust针对复杂数据类型（未实现Copy）提供实现了内部可变性的 `RefCell`。
     *
     * **RefCell 的功能是通过 unsafe 操作，为一个类型（变量/值）对外提供该类型的不可变引用和可变引用，无论这个类型（变量/值）是否可变**。由于是 unsafe 的实现，不受借用规则限制。
     *
     * 对外暴露的不可变引用和可变引用操作是**有限制**的，必须要符合借用规则。
     * RefCell 关注点在为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，这里是 unsafe 的实现，不受借用规则限制。
     * 接收不可变引用和可变引用的变量不属于 RefCell 的关注点，它们依然要符合借用规则，以保证 RefCell 智能指针的正常运行。
     * RefCell 会在内部记录不可变引用（borrow方法）和可变引用（borrow_mut方法）的使用次数，通过**使用次数来判断此时是否符合借用规则**。
     *
     * ```rust
     * // **RefCell 的功能是通过 unsafe 操作，为一个类型（变量/值）对外提供该类型的不可变引用和可变引用，无论这个类型（变量/值）是否可变**。
     * // RefCell 会在内部记录不可变引用（borrow）和可变引用（borrow_mut）的使用次数，通过使用次数来判断此时是否符合借用规则
     * let s = RefCell::new(String::from("Hello World"));
     * let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是1，可变引用是0，符合借用规则，正常运行
     * let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是2，可变引用是0，符合借用规则，正常运行
     * // let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是2，可变引用是1，此时会报错，因为不能同时存在不可变引用和可变引用
     *
     * let s = RefCell::new(String::from("Hello World"));
     * let s1 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行
     * // let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是2，此时会报错，因为不能同时存在多个可变引用（一个可变引用周期内存在另外一个可变引用）
     * println!("{s1}");
     *
     * let s = RefCell::new(String::from("Hello World"));
     * *s.borrow_mut() = String::from("Hi"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
     * *s.borrow_mut() = String::from("Hello"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
     * println!("{s}");
     * ```
     *
     * 也就是 RefCell 实际上**没有解决可变引用和引用可以共存的问题**。
     * 它的关注点在于为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，这里是 **unsafe** 的实现，不受借用规则限制。
     * 所以 RefCell 只是绕过了编译期的错误，将报错从编译期推迟到运行时，从编译器错误变成了 panic 异常。
     *
     * #### 为什么需要 RefCell？
     * 既然没有解决问题，为什么还需要 RefCell？这是因为复杂类型的不可变与可变性。
     * 由于 Rust 的 mutable 特性，一个结构体中的字段，要么全都是 immutable，要么全部是 mutable，**不支持针对部分字段进行设置**。
     * 比如，在一个 struct 中，可能只有个别的字段需要修改，其他字段并不需要修改，为了一个字段而将整个 struct 变为 `&mut` 是不合理的。
     *
     * 而 RefCell 通过 unsafe 操作，可以为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，只需要接收的变量遵守借用规则就不会出现运行时错误。
     *
     * 这意味着可以**通过 RefCell 让一个结构体既有不可变字段，也有可变字段**，例如：
     * ```rust
    * // 通过 RefCell，让一个结构体既有不可变字段，也有可变字段
    * #[derive(Debug)]
    * struct Person {
    *     name: RefCell<String>,
    *     age: i32,
    * }
    * let p = Person {
    *     name: RefCell::new(String::from("L")),
    *     age: 18,
    * };
    * // p.age = 22; 错误的，如果需要age可更改，需要p是可变的。
    * *p.name.borrow_mut() = String::from("M"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    * *p.name.borrow_mut() = String::from("N"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是2，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    * println!("{p:?}");
     * ```
     * 
     * 对于大型的复杂程序，可以选择使用 RefCell 来让事情简化。例如在 Rust 编译器的ctxt结构体中有大量的 RefCell 类型的 map 字段，主要的原因是：这些 map 会被分散在各个地方的代码片段所广泛使用或修改。由于这种分散在各处的使用方式，导致了管理可变和不可变成为一件非常复杂的任务（甚至不可能），你很容易就碰到编译器抛出来的各种错误。而且 RefCell 的运行时错误在这种情况下也变得非常有用：一旦有人做了不正确的使用，代码会 panic，然后告诉我们哪些借用冲突了。
     * 
     * 总之，当有一个复杂类型，既有可变又有不可变，又或者需要被四处使用和修改然后导致借用关系难以管理时，都可以优先考虑使用 RefCell。
     * 
     * #### RefCell 总结
     * - RefCell 适用Copy和非Copy类型，一般来说Copy可直接选择Cell
     * - RefCell 只是绕过编译期的借用规则，程序运行期没有绕过
     * - RefCell 适用于编译期误报或者一个引用被在多处代码使用、修改以至于难于管理借用关系时
     * - 使用 RefCell 时，`borrow` 和 `borrow_mut` 提供不可变引用和可变引用不能违背借用规则，否则会导致运行期的 panic
     *
     * TODO 选择Cell还是RefCell
     * 
     * ### Rc/Arc + RefCell 的组合使用
     * 可以将所有权、借用规则和这些智能指针做一个对比：
     * | Rust 规则                          | 智能指针带来的额外规则                 |
     * | ---------------------------------  | ------------------------------------ |
     * | 一个数据只有一个所有者               | Rc/Arc让一个数据可以拥有多个所有者     |
     * | 要么多个不可变借用，要么一个可变借用  | RefCell实现编译期可变、不可变引用共存   |
     * | 违背规则导致编译错误                 | 违背规则导致运行时panic               |
     * `Rc/Arc` 和 `RefCell` 合理结合，可以解决 Rust 中严苛的所有权和借用规则带来的某些场景下难使用的问题。
     *
     * 但是这个结合并不是运行有效的：
     *
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

    // **RefCell 的功能是通过 unsafe 操作，为一个类型（变量/值）对外提供该类型的不可变引用和可变引用，无论这个类型（变量/值）是否可变**。
    // RefCell 会在内部记录不可变引用（borrow）和可变引用（borrow_mut）的使用次数，通过使用次数来判断此时是否符合借用规则
    let s = RefCell::new(String::from("Hello World"));
    let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是1，可变引用是0，符合借用规则，正常运行
    let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是2，可变引用是0，符合借用规则，正常运行
                         // let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是2，可变引用是1，此时会报错，因为不能同时存在不可变引用和可变引用

    let s = RefCell::new(String::from("Hello World"));
    let s1 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行
                             // let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是2，此时会报错，因为不能同时存在多个可变引用（一个可变引用周期内存在另外一个可变引用）
    println!("{s1}");

    let s = RefCell::new(String::from("Hello World"));
    let mut s2 = s.borrow_mut(); // 给出原始数据的可变引用
    *s2 = String::from("Hi");
    println!("{:?}", &s2); // 运行成功，无论是编译器还是运行时，都是符合rust的借用规则的

    let mut s = String::from("Hello World");
    let s_ref = RefCell::new(s);

    // 通过 RefCell，让一个结构体既有不可变字段，也有可变字段
    #[derive(Debug)]
    struct Person {
        name: RefCell<String>,
        age: i32,
    }
    let p = Person {
        name: RefCell::new(String::from("L")),
        age: 18,
    };
    // p.age = 22; 错误的，如果需要age可更改，需要p是可变的。
    *p.name.borrow_mut() = String::from("M"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    *p.name.borrow_mut() = String::from("N"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是2，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    println!("{p:?}");
}
