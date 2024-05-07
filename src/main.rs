use ilearn::{run, Config};

fn main() {
    /*
     * ## Drop 释放资源
     *
     * 在 Rust 中可以指定在一个变量超出作用域时，执行一段特定的代码。
     * 这段特定的代码可以由编译器自动插入，这样无需在每一个使用该变量的地方，都写一段代码来手动进行收尾工作和资源释放。
     * 指定这样一段收尾工作靠的就是 Drop 特征。
     *
     * 一个函数作用域结束时(函数栈退出)，除返回值外的变量/值外，变量和值有两种行为：
     * - 栈上的变量和值，由于函数栈的退出，这些栈上的变量和值都会被销毁
     * - 堆上的变量和值，作用域内声明的被销毁，作用域外声明的保留
     *
     * ```rust
     *  #[derive(Debug)]
     *  struct MyBox<T>(T);
     *  impl<T> MyBox<T> {
     *      fn new(v: T) -> MyBox<T> {
     *          MyBox(v)
     *      }
     *  }
     *
     *  fn display(s: &mut MyBox<i32>) {
     *      let n = 1; // 作用域内的栈上变量和值都被销毁
     *      println!("{:#?}", s); // 作用域外的堆上变量和值被保留
     *      let v = String::from("Hello World"); // 作用域内声明的堆上变量和值，包括所有权都会被销毁
     *  }
     *  let mut v = MyBox::new(1);
     *  display(&mut v);
     *  println!("{v:#?}"); // 验证离开display函数后，变量所有权未变，值没有被销毁
     * ```
     *
     * 由于函数栈的退出，栈上的变量（包括外部传入的参数，具有所有权）存储在函数栈上，它被销毁是一定的，为什么作用域内声明的堆上变量被销毁，而作用域外被保留？
     *
     * 原因就在于 Drop 特征，Drop 特征为这些变量插入了一段收尾工作的代码：
     * ```rust
     * // 验证drop特征自动插入一段收尾工作（销毁）的代码
     * struct HasDrop1;
     * struct HasDrop2;
     * impl Drop for HasDrop1 {
     *     fn drop(&mut self) {
     *         println!("HasDrop1 dropping")
     *     }
     * }
     * impl Drop for HasDrop2 {
     *     fn drop(&mut self) {
     *         println!("HasDrop2 dropping")
     *     }
     * }
     *
     * fn display_drop() {
     *     let x = HasDrop1 {}; // 变量后被销毁
     *     let y = HasDrop2; // 变量先被销毁
     *     HasDrop1{}; // 直接销毁
     *     println!("display_drop over");
     * }
     *
     * display_drop();
     *
     * println!("main over");
     * ```
     *
     * 输出结果：
     * ```shell
     * display_drop over
     * HasDrop1 dropping
     * HasDrop2 dropping
     * HasDrop1 dropping
     * main over
     * ```
     *
     * display_drop 函数结束时正常自动销毁了 `HasDrop1` 和 `HasDrop2`，但注意**销毁变量**的时机是**函数结束**时，顺序是先销毁 HasDrop2，后销毁 HasDrop1，如果值没有变量接收则立即销毁。
     *
     * 因此**函数内变量的销毁顺序是创建逆序的，先创建后销毁**。如果是结构体中含有结构体（复杂类型），它销毁顺序呢？
     * ```rust
     * struct HasDrop3 {
     *      bar: HasDrop2,
     *      foo: HasDrop1,
     * }
     * impl Drop for HasDrop3 {
     *      fn drop(&mut self) {
     *          println!("HasDrop3 dropping");
     *      }
     * }
     *
     * let z = HasDrop3 { foo: HasDrop1, bar: HasDrop2 }; // 与struct定义顺序不同
     *
     * println!("main over");
     * ```
     *
     * 输出结果：
     * ```shell
     * main over
     * HasDrop3 dropping
     * HasDrop2 dropping
     * HasDrop1 dropping
     * ```
     *
     * 从输出的顺序可以知道：**结构体内部的销毁顺序是结构体属性定义的顺序**，与创建结构体的字段顺序无关。
     *
     * 总结：
     * 一个函数作用域结束时(函数栈退出)，除返回值外的变量/值外，变量和值有两种行为：
     * - 栈上的变量和值，由于函数栈的退出，这些栈上的变量和值都会被销毁
     * - 堆上的变量和值，作用域内声明的被销毁，作用域外声明的保留
     *
     * 堆资源回收的顺序：
     * - **变量级别，按照逆序的方式**，如果 X 在 Y 之前创建，那么 X 在 Y 之后被 drop
     * - **结构体内部，按照顺序的方式**，结构体 X 中的字段按照定义中的顺序依次 drop
     *
     * ### 没有实现 Drop 的结构体
     * 实际上，**Rust 自动为几乎所有的类型实现了 Drop 特征**，因此即使不手动为结构体实现 Drop，它依然会调用默认实现的 drop 函数，同时再调用每个字段的 drop 方法。
     *
     * 移除 `HasDrop3` 的 Drop 实现，并再次调用：
     * ```rust
     * struct HasDrop3 {
     *      bar: HasDrop2,
     *      foo: HasDrop1,
     * }
     *
     * let z = HasDrop3 { foo: HasDrop1, bar: HasDrop2 }; // 与struct定义顺序不同
     *
     * println!("main over");
     * ```
     *
     * 输出结果：
     * ```shell
     * main over
     * HasDrop2 dropping
     * HasDrop1 dropping
     * ```
     *
     * ### 手动销毁
     * 
     * 手动调用时，所有权通过参数传入 `drop()`，然后在 `drop()` 方法结束时(离开作用域)，调用 `Drop()::drop()` 释放掉形参。
     * 也就是说，并不是mem::drop()导致的释放，而是在mem::drop()结束时自动释放。
     * 就是把所有权带走不传出来，假如实现了 Copy trait 就不起作用了
     *
     *
     * drop之所以是&mut self，是为了在清理时可以方便的修改实例内部的信息。
     *
     *
     */

    #[derive(Debug)]
    struct MyBox<T>(T);
    impl<T> MyBox<T> {
        fn new(v: T) -> MyBox<T> {
            MyBox(v)
        }
    }

    fn display(s: &mut MyBox<i32>) {
        let n = 1; // 作用域内的栈上变量和值都被销毁
        println!("{:#?}", s); // 作用域外的堆上变量和值被保留
        let v = String::from("Hello World"); // 作用域内声明的堆上变量和值，包括所有权都会被销毁
    }
    let mut v = MyBox::new(1);
    display(&mut v);
    println!("{v:#?}"); // 验证离开display函数后，变量所有权未变，值没有被销毁

    // 验证drop特征自动插入一段收尾工作（销毁）的代码
    struct HasDrop1;
    struct HasDrop2;
    impl Drop for HasDrop1 {
        fn drop(&mut self) {
            println!("HasDrop1 dropping")
        }
    }
    impl Drop for HasDrop2 {
        fn drop(&mut self) {
            println!("HasDrop2 dropping")
        }
    }

    fn display_drop() {
        let x = HasDrop1 {}; // 变量后被销毁
        let y = HasDrop2; // 变量先被销毁
        HasDrop1 {}; // 直接销毁

        println!("display_drop over");
    }

    display_drop();

    struct HasDrop3 {
        bar: HasDrop2,
        foo: HasDrop1,
    }
    impl Drop for HasDrop3 {
        fn drop(&mut self) {
            println!("HasDrop3 dropping");
        }
    }
    // 结构体内部的销毁
    let z = HasDrop3 {
        foo: HasDrop1,
        bar: HasDrop2,
    };
    println!("main over");
}
