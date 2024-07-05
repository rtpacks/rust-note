use std::{marker::PhantomPinned, pin::Pin};

fn main() {
    /*
     *
     * ## async 异步编程：Pin 和 Unpin
     *
     * 在之前的章节(unit 58-结构体的自引用)中介绍过，自引用结构体是一个很特殊的类型，由于所有权和引用同时存在，导致不能像正常类型一样使用。
     *
     * ```rust
     * struct SelfRef<'a> {
     *     value: String,
     *     pointer_to_value: &'a str, // 该引用指向上面的value
     * }
     *
     * let s = String::from("Hello World");
     * let selfRef = SelfRef {
     *      value: s,
     *      pointer_to_value: &s
     * }
     * ```
     *
     * 由于转移所有权和使用借用同时发生，不符合借用规则，最后编译报错，更多信息需要了解 unit 58-结构体的自引用。
     *
     * 用裸指针实现自引用结构体，并尝试移动自引用结构体：
     * ```rust
     * #[derive(Debug)]
     * struct Test {
     *     a: String,
     *     b: *const String, // 改成指针
     * }
     *
     * impl Test {
     *     fn new(txt: &str) -> Self {
     *         Test {
     *             a: String::from(txt),
     *             b: std::ptr::null(),
     *         }
     *     }
     *     fn init(&mut self) {
     *         let self_ref: *const String = &self.a;
     *         self.b = self_ref;
     *     }
     *     fn a(&self) -> &str {
     *         &self.a
     *     }
     *     fn b(&self) -> &String {
     *         unsafe { &*(self.b) }
     *     }
     * }
     * ```
     *
     * 移动自引用结构体：
     * ```rust
     * let mut test1 = Test::new("test1");
     * test1.init();
     * let mut test2 = Test::new("test2");
     * test2.init();
     *
     * println!("a: {}, b: {}", test1.a(), test1.b());
     * // 使用swap()函数交换两者，这里发生了 move(移动)
     * std::mem::swap(&mut test1, &mut test2);
     * test1.a = "I've totally changed now!".to_string();
     * println!("a: {}, b: {}", test2.a(), test2.b());
     * ```
     *
     * 这里生成两个 Test 结构体实例，然后交换（发生移动）了内存空间，此时两者的 b 属性所存储裸指针的指向就不再是自身的 a 数据。
     * 如果释放其中一个实例，并尝试访问另外一个实例 b 属性指向的数据，就会发生经典的内存错误，访问一个未定义的数据，属于未定义的行为。
     *
     * 这种会发生意外副作用的 move(移动) 就属于不安全的移动，这种不安全的移动行为应该被禁止。
     *
     * 什么是 move 移动？根据官方定义：**所有权转移的这个过程就是 move(移动)**，具体来讲，是一个值内存地址的移动，对象是值。move(移动) 分为安全移动和不安全移动两种。
     *
     * 怎么判断在内存中移动是否安全？判断在内存中移动数据是否会导致意外的副作用，比如上例两个 Test 实例交换后可能导致意外的副作用。
     *
     * 从**是否可以在内存中安全的被移动**的角度，rust 的类型分类两类，`Unpin` 和 `!Unpin`，简单理解，移动安全与非移动安全。
     * > trait 特征前的 `!` 代表没有实现某个特征的意思，`!Unpin` 说明类型没有实现 Unpin 特征。
     *
     * `Unpin` 与 `!Unpin` 区分：
     * - `Unpin` 表示类型**可以在内存中安全地移动**，即能安全的改变地址不会带来意外的错误。绝大多数标准库类型都实现了 Unpin。
     * - `!Unpin` 表示类型**在内存中移动可能会发生意外的副作用**，比如裸指针实现的自引用结构体，改变结构体地址后，存储的裸指针还是访问原地址，存在未定义行为的风险。
     *
     * 这些定义与名称会比较绕，`Unpin` 表示**不需要被固定就可以安全移动的类型**，`!Unpin` 表示没有实现 `Unpin` 特征的类型，也就是**在内存移动中可能发生副作用的类型**。
     * 为什么不用 Pin 和 Unpin 两个名词呢？这是因为 rust 将 `Pin` 作为表示 “固定动作” 的智能指针（结构体），**`Pin` 表示固定一个值的地址，使非安全移动的类型无法被移动**。
     *
     * Pin 可以接收实现 `Unpin` 或 `!Unpin` 特征的类型：
     * ```rust
     * pub struct Pin<P> {
     *     pointer: P,
     * }
     * ```
     *
     * - 如果 Pin 的是 Unpin 类型，则还是可以被移动走的。因为实现 Unpin 就表示移动是安全的
     * - 如果 Pin 的是 !Unpin 类型，则无法被移动走。因为 !Unpin 就表示移动是不安全的
     *
     * 如果将 Unpin 与之前的 Send/Sync 进行对比，会发现它们都很像：
     * - 都是标记特征( marker trait )，该特征未定义任何行为，非常适用于标记
     * - 都可以通过 `!` 语法去除实现
     * - 绝大多数情况都是自动实现, 无需额外关注
     *
     * 记住 Pin 规则：
     * - Pin 固定 `!Unpin` 特征的类型关键就在禁止其被修改移动，被固定也就无法被修改移动，也就是无法在 Safe Rust 的情况下拿到 `Pin<!Unpin>` 类型的可变引用，确保内存安全。
     * - Pin 做的事情就是制定一个编译器规则，在使用自引用类型的时候给予帮助/提示，并禁止在 Safe Rust 中编写异常的代码，如禁止移动实现了 `!Unpin` 特征的数据类型。如果非要在 unsafe 中做可能发生异常的动作，那么 Pin 将没有任何作用。
     *
     * 什么时候使用 Pin？如果不希望某个被引用的内容发生改变，就可以使用 Pin。
     * 如 Test 结构体是一个自引用结构体，如果移动会发生意外的副作用，所以它的移动应该是被禁止的，使用 Pin 固定地址，禁止其移动。
     *
     * Pin 只会固定 `!Unpin` 类型，`Unpin` 类型不受影响。
     * Pin 固定实现 `Unpin` 特征的类型时，因为 Unpin 不受影响，所以在 **Safe Rust** 的环境下，`Pin<impl Unpin>` 有两种方式能够安全的拿到可变引用：
     * - 通过 `Pin::get_mut()` 安全的获取可变引用
     * - 实现 Unpin 的类型，再实现 `DerefMut` 特征，就可以直接解引用拿到可变引用。其实这个也是通过 `get_mut` 获取的。
     *
     * ```rust
     *
     * ```
     *
     * // TODO
     *
     * Pin 固定实现 `!Unpin` 特征的类型时，有两种方式：
     * - 使用 PhantomPinned，PhantomPinned 实现了 `!Unpin`，只要一个类型属性是 `!Unpin`，这个结构体就默认成为实现 `!Unpin` 的类型
     * - 手动实现 impl !Unpin for Test {}
     *
     * ```rust
     * #[derive(Debug)]
     * struct Test {
     *     a: String,
     *     b: *const String,
     *     _pin: PhantomPinned, // 用 PhantomPinned 标识该类型是一个实现 `!Unpin` 的类型，只要一个类型属性是 `!Unpin`，这个结构体就默认成为实现 `!Unpin` 的类型
     * }
     * ```
     *
     * TODO 使用 Pin 禁止自引用结构体在 Safe Rust 中移动
     *
     *
     * ### 为什么 Pin 可以解决非安全移动问题
     * 非安全移动的类型会发生问题的原因就在于被移动了。回顾 Pin 的作用，**`Pin` 表示固定一个值的地址**，它可以接收实现 `Unpin` 或 `!Unpin` 特征的类型。
     * 也就是 Pin 固定值、不让值移动的行为，从概念的角度上看，已经解决了非安全移动的问题。接下来简单的看一下实现方式是如何解决这个问题的。
     *
     * Pin 的作用就是保证在 Safe Rust 中被其包裹的指针所指向的值在内存中的位置不变，Pin 防止其包裹指针所指向的内容不会变的实现很简单，即不能获取其包裹指针的可变引用。
     *
     * 所以 Pin 做的事情就是制定一个编译器规则，在使用自引用类型的时候给予帮助/提示，并禁止在 Safe Rust 中编写异常的代码，如禁止移动实现了 `!Unpin` 特征的数据类型。
     * 如果非要在 unsafe 中修改，那么 Pin 没有任何作用。
     *
     *
     *
     * TODO Pin 是怎么在 Safe Rust 中防止外部拿到可变引用的
     * 使用Pin后，并不意味着就不需要使用 unsafe 操作，Pin 只是将由于 `!Unpin` 在内存移动可能引发副作用的移动限制了，不会移动就不发生 `!Unpin` 的副作用。
     * 至于在原内存空间修改值，还是需要 unsafe 操作获取被 Pin 包括的值，此时的 unsafe 代码是安全的，因为修改内部的值不会导致整体的内存地址发生变化
     *
     * ### Future 为何需要 Pin
     * Future 为何需要 Pin？Future 可以是一个自引用结构体，Pin 可以解决自引用结构体的问题，自然也可以解决 Future 的自引用问题。
     *
     * TODO 为什么 Future 是一个自引用结构体？
     *
     *
     * ### 参考阅读
     * - https://folyd.com/blog/rust-pin-unpin/
     * - https://folyd.com/blog/rust-pin-advanced/#pin-shi-xian-de-trait
     *
     * 归档
     * - https://web.archive.org/web/20240627082751/https://folyd.com/blog/rust-pin-unpin/
     * - https://web.archive.org/web/20240627082717/https://folyd.com/blog/rust-pin-advanced/#pin-shi-xian-de-trait
     *
     */

    // struct SelfRef<'a> {
    //     value: String,
    //     pointer_to_value: &'a str, // 该引用指向上面的value
    // }
    // 由于转移所有权和使用借用同时存在，不符合借用规则，编译报错
    // let s = String::from("Hello World");
    // let selfRef = SelfRef {
    //     value: s,
    //     pointer_to_value: &s,
    // };

    // 改成存储裸指针的实现
    // #[derive(Debug)]
    // struct Test {
    //     a: String,
    //     b: *const String, // 改成指针
    // }

    // impl Test {
    //     fn new(txt: &str) -> Self {
    //         Test {
    //             a: String::from(txt),
    //             b: std::ptr::null(),
    //         }
    //     }
    //     fn init(&mut self) {
    //         let self_ref: *const String = &self.a;
    //         self.b = self_ref;
    //     }
    //     fn a(&self) -> &str {
    //         &self.a
    //     }
    //     fn b(&self) -> &String {
    //         unsafe { &*(self.b) }
    //     }
    // }
    // let mut test1 = Test::new("test1");
    // test1.init();
    // let mut test2 = Test::new("test2");
    // test2.init();
    // println!("a: {}, b: {}", test1.a(), test1.b());
    // // 使用swap()函数交换两者，这里发生了 move(移动)
    // std::mem::swap(&mut test1, &mut test2);
    // test1.a = "I've totally changed now!".to_string();
    // println!("a: {}, b: {}", test2.a(), test2.b());

    // Pin 智能指针与 Unpin 特征
    let mut pin_num = Box::pin(1);
    pin_num = Box::pin(1);

    struct A {
        a: i32,
    }
    let mut a_ins = Box::pin(A { a: 1 });


    #[derive(Debug)]
    struct Test {
        a: String,
        b: *const String,
        _pin: PhantomPinned,
    }

    impl Test {
        fn new(txt: &str) -> Pin<Box<Self>> {
            let t = Test {
                a: String::from(txt),
                b: std::ptr::null(),
                _pin: PhantomPinned,
            };
            let mut boxed = Box::pin(t);
            let self_ptr: *const String = &boxed.as_ref().a;
            unsafe { boxed.as_mut().get_unchecked_mut().b = self_ptr };

            boxed
        }

        fn a<'a>(self: Pin<&'a Self>) -> &'a str {
            &self.get_ref().a
        }

        fn b<'a>(self: Pin<&'a Self>) -> &'a String {
            unsafe { &*(self.b) }
        }
    }

    let mut test1 = Test::new("test1");
    let mut test2 = Test::new("test2");

    println!("a: {}, b: {}", test1.as_ref().a(), test1.as_ref().b());
    // 使用swap()函数交换两者，这里发生了 move(移动)
    // std::mem::swap(&mut *test1, &mut *test2);
    // std::mem::swap(test1.as_mut().get_mut(), test2.as_mut().get_mut());
    println!("a: {}, b: {}", test2.as_ref().a(), test2.as_ref().b());
}
