use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use ilearn::{run, Config};

fn main() {
    /*
     * ## 循环引用与自引用
     * 链表在 Rust 中之所以这么难，完全是因为循环引用和自引用的问题引起的，这两个问题可以说综合了 Rust 的很多难点。
     * rust 的安全措施可以避免大部分的内存问题，但是不代表它不会出现问题。一个典型的例子就是同时使用 `Rc` 和 `RefCell` 出现的循环引用，导致引用计数无法归零，最终出现内存问题。
     *
     * 在 Rc/Arc 介绍的章节中，使用 Rc/Arc 解决图数据结构引用复杂的问题，但因为 Rc/Arc 是不可变引用，所以不能通过 Rc/Arc 修改节点的数据。
     *
     * 同时在上一章提过 Rc 与 RefCell 的不同组合（`Rc<RefCell<T>>` 和 `RefCell<Rc<T>>`）的特点。
     * - `Rc<RefCell<T>>` 类型是一个通过 Rc 可供多个变量引用，通过 RefCell 可提供不可变/可变借用的高级类型，也就是每个该类型的变量都可单独读写真实数据。
     * - `RefCell<Rc<T>>` 类型也是一个内部可变性的高级类型，不仅可以提供不可变/可变借用和无需手动管理复杂的生命周期，
     *      **`RefCell<Rc<T>>` 还能通过 `Rc` 智能指针可以指向 T 类型的不同实例**，`RefCell<T>` 只能指向 T 类型的一个实例，因为这是内部可变性，而不是整体可变。
     *
     * - `Rc<RefCell<T>>` 是常用的一种组合，它给变量提供了自动管理引用释放转换不可变/可变借用的功能
     * - `RefCell<Rc<T>>` 是解决内部可变性整体不可变的限制，它为变量提供不可变/可变引用和存储数据指针的空间
     *
     * 现在利用 RefCell 与 Rc 的组合 `RefCell<Rc<T>>` 就可以实现一个简单的链表。
     * ```rust
     * [derive(Debug)]
     * enum List {
     *     Cons(i32, RefCell<Rc<List>>),
     *     Nil,
     * }
     *
     * let node = List::Cons(12, RefCell::new(Rc::new(List::Nil)));
     * println!("{:#?}", node);
     *
     * match &node {
     *     List::Cons(v, next) => {
     *         // next 原有的指针信息是指向 List::Nil 的，现在指向 List::Cons(20, RefCell::new(Rc::new(List::Nil)))
     *         // 此时是存储的指针信息发生改变，而不是存储空间发生改变
     *         *next.borrow_mut() = Rc::new(List::Cons(20, RefCell::new(Rc::new(List::Nil))))
     *     }
     *     List::Nil => {}
     * }
     * println!("{:#?}", node);
     * ```
     *
     * 上面注意 `RefCell<Rc<T>>` 更改存储的指针就能达到更改指向的节点的功能，这与更改节点数据使节点看起来是另外一个节点是两回事。
     * `RefCell<Rc<T>>` 关注的是指向其他节点，而 `Rc<RefCell<T>>` 只是更改节点内容信息。
     *
     * 以上链表的实现非常简单，没有考虑循环引用以及自引用的情况，看一段循环引用的实现：
     * - 创建 a，创建 b，利用 `Rc::clone(&a)` 让 b 的 next 指向 a，即 b 引用了 a
     * - 利用 `Rc::clone(&b)` 让 a 的 next 指向 b，即 a 引用了 b
     * ```rust
     * // 定义取出节点的下一节点的方法，不拿走下一节点的所有权，即只拿引用，又因为下一节点是RefCell类型，所以可以只拿不可变引用就能得到可变引用
     * impl List {
     *     fn tail(&self) -> Option<&RefCell<Rc<List>>> {
     *         match self {
     *             List::Cons(_, next) => Some(next),
     *             List::Nil => None,
     *         }
     *     }
     * }
     *
     * // - 创建 a，创建 b，利用 `Rc::clone(&a)` 让 b 的 next 指向 a，即 b 引用了 a
     * // - 利用 `Rc::clone(&b)` 让 a 的 next 指向 b，即 a 引用了 b
     * let a = Rc::new(List::Cons(1, RefCell::new(Rc::new(List::Nil))));
     * let b = Rc::new(List::Cons(2, RefCell::new(Rc::new(List::Nil))));
     * println!(
     *     "Initial: a.strone_count = {}, b.strone_count = {}",
     *     Rc::strong_count(&a),
     *     Rc::strong_count(&b)
     * );
     * if let Some(next) = b.tail() {
     *     *next.borrow_mut() = Rc::clone(&a); // b 的 next 指向 a，即 b 引用了 a
     * }
     * println!(
     *     "b.next: a.strone_count = {}, b.strone_count = {}",
     *     Rc::strong_count(&a),
     *     Rc::strong_count(&b)
     * );
     * if let Some(next) = a.tail() {
     *     *next.borrow_mut() = Rc::clone(&b); // a 的 next 指向 b，即 a 引用了 b
     * }
     * println!(
     *     "a.next: a.strone_count = {}, b.strone_count = {}",
     *     Rc::strong_count(&a),
     *     Rc::strong_count(&b)
     * );
     * // 成功创建了循环引用a-> b -> a -> b ····
     *
     * // 循环引用，造成8MB的主线程栈溢出
     * // println!("{:?}", a.tail());
     * ```
     * 在 main 函数结束前，a 和 b 的引用计数均是 2，随后变量 a 触发 Drop，此时值的 Rc 引用计数变为 1，由于 b.next 的引用，Rc 引用计数并不会归 0，因此变量 a 所指向内存（值）不会被释放，同理 b 指向的内存（值）也不会被释放，最终发生了内存泄漏。
     *
     * 只需要访问 a 或 b 的下一节点，就会触发循环引用：
     * ```rust
     * // 循环引用，造成8MB的主线程栈溢出
     * // println!("{:?}", a.tail());
     * ```
     * `a.tail` 调用返回 `a.next` 引用即 `b` 智能指针，打印 `b` 智能指针时会打印 `b.next` 智能指针（打印一个完整的链表数据）。
     * `b.next` 即 `a`，打印 `a` 时同样因为要打印完整的数据，会打印 `a.next` 即 `b`，最后出现循环引用不断解析引用指向的数据。
     * Rust 试图打印出 `a -> b -> a ···` 的所有内容，最终 main 线程终于不堪重负，发生了栈溢出。
     * 
     * 创建循环引用并不简单，但是也并不是完全遇不到，当使用 `RefCell<Rc<T>>` 或者**具备内部可变性和引用计数的嵌套组合类型**时，就要打起万分精神，避免出现循环引用。
     * 
     * ### Weak
     * 
     * 
     * 
     *
     *
     *
     *
     *
     *
     *
     *
     */

    #[derive(Debug)]
    enum List {
        Cons(i32, RefCell<Rc<List>>),
        Nil,
    }

    let node = List::Cons(12, RefCell::new(Rc::new(List::Nil)));
    println!("{:#?}", node);

    match &node {
        List::Cons(v, next) => {
            // next 原有的指针信息是指向 List::Nil 的，现在指向 List::Cons(20, RefCell::new(Rc::new(List::Nil)));
            // 此时是存储的指针信息发生改变，而不是存储空间发生改变
            *next.borrow_mut() = Rc::new(List::Cons(20, RefCell::new(Rc::new(List::Nil))))
        }
        List::Nil => {}
    }
    println!("{:#?}", node);

    // 定义取出节点的下一节点的方法，不拿走下一节点的所有权，即只拿引用，又因为下一节点是RefCell类型，所以可以只拿不可变引用就能得到可变引用
    impl List {
        fn tail(&self) -> Option<&RefCell<Rc<List>>> {
            match self {
                List::Cons(_, next) => Some(next),
                List::Nil => None,
            }
        }
    }

    // - 创建 a，创建 b，利用 `Rc::clone(&a)` 让 b 的 next 指向 a，即 b 引用了 a
    // - 利用 `Rc::clone(&b)` 让 a 的 next 指向 b，即 a 引用了 b
    let a = Rc::new(List::Cons(1, RefCell::new(Rc::new(List::Nil))));
    let b = Rc::new(List::Cons(2, RefCell::new(Rc::new(List::Nil))));
    println!(
        "Initial: a.strone_count = {}, b.strone_count = {}",
        Rc::strong_count(&a),
        Rc::strong_count(&b)
    );
    if let Some(next) = b.tail() {
        *next.borrow_mut() = Rc::clone(&a); // b 的 next 指向 a，即 b 引用了 a
    }
    println!(
        "b.next: a.strone_count = {}, b.strone_count = {}",
        Rc::strong_count(&a),
        Rc::strong_count(&b)
    );
    if let Some(next) = a.tail() {
        *next.borrow_mut() = Rc::clone(&b); // a 的 next 指向 b，即 a 引用了 b
    }
    println!(
        "a.next: a.strone_count = {}, b.strone_count = {}",
        Rc::strong_count(&a),
        Rc::strong_count(&b)
    );

    // 成功创建了循环引用a-> b -> a -> b ····

    // 循环引用，造成8MB的主线程栈溢出
    // println!("{:?}", a.tail());
}
