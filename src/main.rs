use std::{
    borrow::Borrow,
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
     * // - 创建 a，创建 b，利用 `Rc::clone(&b)` 让 a 的 next 指向 b，即 a 引用了 b
     * // - 利用 `Rc::clone(&a)` 让 b 的 next 指向 a，即 b 引用了 a
     * let a = Rc::new(List::Cons(1, RefCell::new(Rc::new(List::Nil))));
     * let b = Rc::new(List::Cons(2, RefCell::new(Rc::new(List::Nil))));
     * println!(
     *     "Initial: a.strone_count = {}, b.strone_count = {}",
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
     * if let Some(next) = b.tail() {
     *     *next.borrow_mut() = Rc::clone(&a); // b 的 next 指向 a，即 b 引用了 a
     * }
     * println!(
     *     "b.next: a.strone_count = {}, b.strone_count = {}",
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
     * 在上面的循环引用案例中，main 函数结束前释放变量，由于值被其他变量引用所以不能被释放。这种引用属于强引用，只要存在强引用，值就不会被释放，换句话说强引用会阻止值的释放。
     *
     * 相比较强引用，**弱引用不持有值所有权，不保证引用关系一直有效，它仅仅保存一份指向数据的弱引用**。
     * 所以所引用它无法阻止所引用的内存值被释放，即**弱引用本身不对值的存在性做任何担保**，引用的值还存在就返回 Some，不存在就返回 None。
     *
     * `Weak` 就是一个弱引用，它与 Rc 非常相似，但是与 Rc 持有所有权不同，Weak 不持有所有权，它仅仅保存一份指向数据的弱引用。
     * `Weak` 虽然与 Rc 智能指针不同，但是有效 `Weak` 的生成总是需要 Rc 指针，通过值的 Rc 指针形式 `Rc::downgrade` 生成有效的 Weak 指针。
     *
     * 如果想要访问数据，需要通过 Weak 指针的 `upgrade` 方法实现，该方法返回一个类型为 `Option<Rc<T>>` 的值。
     *
     * **对比 Weak 和 Rc**
     * | Weak 弱引用，不计数                         | Rc 强引用，引用计数                    |
     * | ----------------------------------------- | ------------------------------------- |
     * | 不拥有所有权                               | 拥有访问值的能力（不可变引用访问）        |
     * | 不阻止值被释放(drop)                        | 所有权计数归零，才能 drop               |
     * | 引用的值存在返回 Some，不存在返回 None       | 引用的值必定存在                        |
     * | 通过 upgrade 取到 `Option<Rc<T>>`，然后再取值 | 通过 Deref 自动解引用，取值无需任何操作 |
     *
     * Weak 的功能非常弱，而这种弱恰恰适合以下的场景：
     * - 阻止 Rc 导致的循环引用，因为 Rc 智能指针的引用计数机制，可能会导致一个值对应的多个 Rc 计数无法归零
     * - 希望持有一个 Rc 类型的对象的临时引用，并希望这个引用不影响任何值的释放
     *
     * 在上面的 List 结构中，可以使用这种方式来解决循环引用问题：让前一个节点通过 Rc 来引用后一个节点，然后让后一个节点通过 Weak 来引用前一个节点。
     * Rc 强引用意味着数据不会被释放，Weak 需要使用 `upgrade` 后才能访问数据，意味着无法直接访问弱引用指向的数据，能避开循环引用的陷阱。
     *
     * ```rust
     * // 由于多出一种 Weak 类型，当前的List不满足使用，需要拓展字段。同时有可能需要修改前一个节点的值，所以用 `RefCell` 包裹
     * // 利用枚举创建一个空值类型，避免结构体没有空值的问题
     * #[derive(Debug)]
     * enum Node {
     *     // value, prev, next
     *     Cons(i32, RefCell<Weak<Node>>, RefCell<Rc<Node>>),
     *     Nil,
     * }
     *
     * impl Node {
     *     fn next(&self) -> Option<&RefCell<Rc<Node>>> {
     *         match self {
     *             Node::Cons(_, _, next) => Some(next),
     *             Node::Nil => None,
     *         }
     *     }
     *     fn prev(&self) -> Option<&RefCell<Weak<Node>>> {
     *         match self {
     *             Node::Cons(_, prev, _) => Some(prev),
     *             Node::Nil => None,
     *         }
     *     }
     * }
     *
     * //  使用 Rc 指向下一个节点，使用 Weak 指向上一个节点，避免循环引用陷阱
     * // - 创建 a，创建 b，利用 `Rc::clone(&b)` 让 a 的 next 指向 b，即 a 引用了 b
     * // - 利用 `Rc::downgrade(&a)` 让 b 的 prev 指向 a，即 b 弱引用了 a
     * let a = Rc::new(Node::Cons(
     *     1,
     *     RefCell::new(Weak::new()),
     *     RefCell::new(Rc::new(Node::Nil)),
     * ));
     * let b = Rc::new(Node::Cons(
     *     2,
     *     RefCell::new(Weak::new()),
     *     RefCell::new(Rc::new(Node::Nil)),
     * ));
     * println!(
     *     "Initial: a.strong_count = {}, a.weak_count = {}, b.strone_count = {}, b.weak_count = {}",
     *     Rc::strong_count(&a),
     *     Rc::weak_count(&a),
     *     Rc::strong_count(&b),
     *     Rc::weak_count(&b)
     * );
     * if let Some(next) = a.next() {
     *     *next.borrow_mut() = Rc::clone(&b); // a 的 next 指向 b，即 a 引用了 b
     * }
     * println!(
     *     "a.next: a.strong_count = {}, a.weak_count = {}, b.strone_count = {}, b.weak_count = {}",
     *     Rc::strong_count(&a),
     *     Rc::weak_count(&a),
     *     Rc::strong_count(&b),
     *     Rc::weak_count(&b)
     * );
     * if let Some(prev) = b.prev() {
     *     *prev.borrow_mut() = Rc::downgrade(&a); // Weak 有值的引用只有通过 Rc::downgrade 生成，Weak::new 是无值的弱引用
     * }
     * println!(
     *     "b.prev: a.strong_count = {}, a.weak_count = {}, b.strone_count = {}, b.weak_count = {}",
     *     Rc::strong_count(&a),
     *     Rc::weak_count(&a),
     *     Rc::strong_count(&b),
     *     Rc::weak_count(&b)
     * );
     *
     * println!("{:?}", a); // Weak 需要使用 upgrade 方法后才能访问真实数据，因此 Weak 会阻断循环引用的生成
     * ```
     *
     * Weak 需要使用 `upgrade` 方法后才能访问真实数据，因此 Weak 会阻断循环引用的生成。
     *
     * 除了树类型数据结构，还有所属关系的数据结构容易引起循环引用，比如杯子与所属人的关系，一个人可以有多个杯子，一个杯子属于一个人。
     * 在这种设计下，可以将人属性中的杯子属性设置为 Weak 或者把杯子中的所有人属性设置为 Weak。
     * 
     * ```rust
     * #[derive(Debug)]
     * struct Owner {
     *     id: i32,
     *     // RefCell<Vec<Weak<Cup>>> 比 Vec<RefCell<Weak<Cup>>> 更好的在于可以直接借用一整个vec，就可以修改vec内的元素。不需要单独借用再修改
     *     cups: RefCell<Vec<Weak<Cup>>>,
     * }
     *
     * #[derive(Debug)]
     * struct Cup {
     *     id: i32,
     *     owner: RefCell<Rc<Owner>>,
     * }
     *
     * // Rc::clone(&owner)
     * let owner = Rc::new(Owner {
     *     id: 0,
     *     cups: RefCell::new(vec![]),
     * });
     * let cup1 = Rc::new(Cup {
     *     id: 1,
     *     owner: RefCell::new(Rc::clone(&owner)),
     * });
     * let cup2 = Rc::new(Cup {
     *     id: 2,
     *     owner: RefCell::new(Rc::clone(&owner)),
     * });
     *
     * println!(
     *     "Initial: owner.strong_count = {}, owner.weak_count = {}, \ncup1.strong_count = {}, cup1.weak_count = {}, \ncup2.strong_count = {}, cup2.weak_count = {}",
     *     Rc::strong_count(&owner),
     *     Rc::weak_count(&owner),
     *     Rc::strong_count(&cup1),
     *     Rc::weak_count(&cup1),
     *     Rc::strong_count(&cup2),
     *     Rc::weak_count(&cup2),
     * );
     *
     * *owner.cups.borrow_mut() = vec![Rc::downgrade(&cup1), Rc::downgrade(&cup2)];
     *
     * println!("{:?}", owner);
     *
     * for cup in owner.cups.borrow().iter() {
     *     match cup.upgrade() {
     *         Some(x) => {
     *             println!("{x:?}")
     *         }
     *         None => {
     *             println!("当前引用失效")
     *         }
     *     }
     * }
     * ```
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

    // - 创建 a，创建 b，利用 `Rc::clone(&b)` 让 a 的 next 指向 b，即 a 引用了 b
    // - 利用 `Rc::clone(&a)` 让 b 的 next 指向 a，即 b 引用了 a
    let a = Rc::new(List::Cons(1, RefCell::new(Rc::new(List::Nil))));
    let b = Rc::new(List::Cons(2, RefCell::new(Rc::new(List::Nil))));
    println!(
        "Initial: a.strone_count = {}, b.strone_count = {}",
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
    if let Some(next) = b.tail() {
        *next.borrow_mut() = Rc::clone(&a); // b 的 next 指向 a，即 b 引用了 a
    }
    println!(
        "b.next: a.strone_count = {}, b.strone_count = {}",
        Rc::strong_count(&a),
        Rc::strong_count(&b)
    );

    // 成功创建了循环引用a-> b -> a -> b ····

    // 循环引用，造成8MB的主线程栈溢出
    // println!("{:?}", a.tail());

    // 由于多出一种 Weak 类型，当前的List不满足使用，需要拓展字段。同时有可能需要修改前一个节点的值，所以用 `RefCell` 包裹
    // 利用枚举创建一个空值类型，避免结构体没有空值的问题
    #[derive(Debug)]
    enum Node {
        // value, prev, next
        Cons(i32, RefCell<Weak<Node>>, RefCell<Rc<Node>>),
        Nil,
    }

    impl Node {
        fn next(&self) -> Option<&RefCell<Rc<Node>>> {
            match self {
                Node::Cons(_, _, next) => Some(next),
                Node::Nil => None,
            }
        }
        fn prev(&self) -> Option<&RefCell<Weak<Node>>> {
            match self {
                Node::Cons(_, prev, _) => Some(prev),
                Node::Nil => None,
            }
        }
    }

    //  使用 Rc 指向下一个节点，使用 Weak 指向上一个节点，避免循环引用陷阱
    // - 创建 a，创建 b，利用 `Rc::clone(&b)` 让 a 的 next 指向 b，即 a 引用了 b
    // - 利用 `Rc::downgrade(&a)` 让 b 的 prev 指向 a，即 b 弱引用了 a
    let a = Rc::new(Node::Cons(
        1,
        RefCell::new(Weak::new()),
        RefCell::new(Rc::new(Node::Nil)),
    ));
    let b = Rc::new(Node::Cons(
        2,
        RefCell::new(Weak::new()),
        RefCell::new(Rc::new(Node::Nil)),
    ));
    println!(
        "Initial: a.strong_count = {}, a.weak_count = {}, b.strone_count = {}, b.weak_count = {}",
        Rc::strong_count(&a),
        Rc::weak_count(&a),
        Rc::strong_count(&b),
        Rc::weak_count(&b)
    );
    if let Some(next) = a.next() {
        *next.borrow_mut() = Rc::clone(&b); // a 的 next 指向 b，即 a 引用了 b
    }
    println!(
        "a.next: a.strong_count = {}, a.weak_count = {}, b.strone_count = {}, b.weak_count = {}",
        Rc::strong_count(&a),
        Rc::weak_count(&a),
        Rc::strong_count(&b),
        Rc::weak_count(&b)
    );
    if let Some(prev) = b.prev() {
        *prev.borrow_mut() = Rc::downgrade(&a); // Weak 有值的引用只有通过 Rc::downgrade 生成，Weak::new 是无值的弱引用
    }
    println!(
        "b.prev: a.strong_count = {}, a.weak_count = {}, b.strone_count = {}, b.weak_count = {}",
        Rc::strong_count(&a),
        Rc::weak_count(&a),
        Rc::strong_count(&b),
        Rc::weak_count(&b)
    );

    println!("{:?}", a); // Weak 需要使用 upgrade 方法后才能访问真实数据，因此 Weak 会阻断循环引用的生成

    #[derive(Debug)]
    struct Owner {
        id: i32,
        // RefCell<Vec<Weak<Cup>>> 比 Vec<RefCell<Weak<Cup>>> 更好的在于可以直接借用一整个vec，就可以修改vec内的元素。不需要单独借用再修改
        cups: RefCell<Vec<Weak<Cup>>>,
    }

    #[derive(Debug)]
    struct Cup {
        id: i32,
        owner: RefCell<Rc<Owner>>,
    }

    // Rc::clone(&owner)
    let owner = Rc::new(Owner {
        id: 0,
        cups: RefCell::new(vec![]),
    });
    let cup1 = Rc::new(Cup {
        id: 1,
        owner: RefCell::new(Rc::clone(&owner)),
    });
    let cup2 = Rc::new(Cup {
        id: 2,
        owner: RefCell::new(Rc::clone(&owner)),
    });

    println!(
        "Initial: owner.strong_count = {}, owner.weak_count = {}, \ncup1.strong_count = {}, cup1.weak_count = {}, \ncup2.strong_count = {}, cup2.weak_count = {}",
        Rc::strong_count(&owner),
        Rc::weak_count(&owner),
        Rc::strong_count(&cup1),
        Rc::weak_count(&cup1),
        Rc::strong_count(&cup2),
        Rc::weak_count(&cup2),
    );

    *owner.cups.borrow_mut() = vec![Rc::downgrade(&cup1), Rc::downgrade(&cup2)];

    println!("{:?}", owner);

    for cup in owner.cups.borrow().iter() {
        match cup.upgrade() {
            Some(x) => {
                println!("{x:?}")
            }
            None => {
                println!("当前引用失效")
            }
        }
    }
}
