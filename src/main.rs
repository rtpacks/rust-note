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
     * > 析构函数 destructor：一个用来**清理实例**的通用编程概念，与构造函数对应
     *
     * Drop 特征是编译器自动插入变量的收尾工作代码，编译器通过 `Drop::drop(&mut self)` 释放资源。
     * 但 Drop 特征有一个特殊限制：**不允许手动调用析构函数 `Drop::drop(&mut self)`**。
     *
     * 即实现 Drop 特征的结构体，编译器可以自动插入收尾工作代码（释放资源 Drop::drop），但不允许手动调用 `Drop::drop(&mut self)` 释放资源。
     *
     * ```rust
     * #[derive(Debug)]
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
     * let mut z = HasDrop3 { foo: HasDrop1, bar: HasDrop2 };
     *
     * z.drop(); 错误代码，不允许直接调用析构函数，等于下一行
     * Drop::drop(&mut z); 错误代码，不允许直接调用析构函数，属于上一行的类型转换
     * ```
     *
     * 以上报错是**不允许手动调用析构函数 `Drop::drop(&mut self)`**引发的，它受到 rust 的所有权模型的限制：
     *
     * 如果允许手动调用 `Drop::drop(&mut self)`，`Drop::drop(&mut self)` 的接收者是 `&mut self`。
     * 因为 `&mut self` 没有转移变量所有权，所以在手动调用 `Drop::drop` 释放变量后，编译器根据生命周期检查，发现变量的**所有权未丢失**，这样就会造成两个严重问题：
     *
     * **1. 可能访问错误数据**
     *
     * 因为手动调用 `Drop::drop` 释放变量后，编译器根据生命周期检查发现变量所有权未丢失，变量可以正常使用，所以再次访问时，会访问到错误的数据。
     * ```rust
     * let mut z = HasDrop3 { foo: HasDrop1, bar: HasDrop2 };
     * z.drop();
     * println!("Running!:{:#?}", z); 访问错误的数据
     * ```
     *
     * **2. 二次析构（释放）**
     *
     * 同样，编译器发现所有权未丢失，变量可以再次使用，且变量实现了Drop特征，因此在函数结束时，根据Drop特征自动释放变量，造成二次析构。
     * ```rust
     * fn display() {
     *      let mut z = HasDrop3 { foo: HasDrop1, bar: HasDrop2 };
     *      z.drop();
     * }
     * display()
     * ```
     *
     * #### 通过 drop 释放
     * `Drop::drop(&mut self)` 是能够释放资源的，但根据所有权模型和生命周期，`&mut self`由于没有转移变量所有权，手动调用会存在许多问题。
     *
     * 以上的案例都在说明，如果希望释放资源，**需要转移变量所有权**，让变量不能再使用，即让所有权模型和生命周期正常工作，rust 提供的手动释放函数 `mem::drop()` 函数非常简单：
     * ```rust
     * pub fn drop<T>(_x: T) {}
     * ```
     *
     * 所有权通过参数传入 `drop()`，然后在 `drop()` 方法结束时(离开作用域)，调用 `Drop()::drop()` 释放掉形参（rust为几乎所有的类型都实现了Drop特征），保证堆上的资源被释放。
     * 也就是说，并不是 `mem::drop()` 导致的释放，而是在 `mem::drop()` 结束时编译器根据 Drop 特征自动插入的收尾工作代码（自动释放）。
     *
     * 它的核心目的就是把所有权带进来，而不传出来。这样就保证 `mem::drop` 函数正常释放变量，并且 `mem::drop` 函数外该变量不能再使用。
     *
     * 手动做一个drop函数：
     * ```rust
     *     let mut z = HasDrop3 {
     *     foo: HasDrop1,
     *     bar: HasDrop2,
     * };
     * fn dropHeap<T>(_v: T) {}
     * dropHeap(z);
     *
     * println!("{:#?}", z); 错误代码，所有权被 dropHeap 函数转移，不能再使用变量
     * ```
     *
     * ### Drop::drop(&mut self) 的 &mut self
     * `Drop` 特征是rust自动清理的来源，它的职责是**执行任何必要的清理逻辑，而不是处理内存释放细节**。
     *
     * 为什么 `Drop()::drop(&mut self)` 的接收者是 `&mut self`，`&self` `self` 作为接收者有什么缺陷？
     *
     * > https://www.zhihu.com/question/612370614
     *
     * 1. 不可能是 `&self`
     * 要清理结构体内部的数据，必须能具有变量所有权或可变引用才能改变结构体数据，因此只读引用不适合。
     *
     * 2. `self` 不适合
     * 在上面的提到过：**堆上的变量和值，作用域内声明的被销毁，作用域外声明的保留**。
     * 
     * `self` 接收者会转移变量的所有权，即相当于在函数作用域内声明了变量，在函数栈退出时就会被释放：
     * ```rust
     * struct CustomStruct;
     * impl Drop for CustomStruct {
     *     fn drop(self) {
     *         println!("drop");
     *         // 这里由于函数栈的退出，当前的 `self` drop，又调用了析构，因此会无限打印"drop"
     *     }
     * }
     * ```
     * 
     * 从示例中可以预测到，如果将 `self: Self` 当作接收者，第一次是外部函数的函数栈退出，调用了析构函数，第二次开始是析构函数的函数栈退出，调用了析构函数，形成死循环调用，因此 `self: Self` 是不适合的。
     *
     * drop 之所以是&mut self，是为了在清理时可以方便的修改实例内部的信息。假如实现了 Copy trait 就不起作用了
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

    // Drop::drop 不允许手动调用
    let mut z = HasDrop3 {
        foo: HasDrop1,
        bar: HasDrop2,
    };
    // z.drop(); 错误代码，不允许直接调用析构函数，等于下一行
    // Drop::drop(&mut z); 错误代码，不允许直接调用析构函数，属于上一行的类型转换

    drop(z);

    let mut z = HasDrop3 {
        foo: HasDrop1,
        bar: HasDrop2,
    };
    fn dropHeap<T>(_v: T) {}
    dropHeap(z);
    // println!("{:#?}", z); 错误代码，所有权被转移，不能再使用变量
}
