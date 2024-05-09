use ilearn::{run, Config};

fn main() {
    /*
     * ## Drop 释放资源
     *
     * 在 Rust 中可以指定在一个变量超出作用域时，执行一段特定的代码。
     * 这段特定的代码可以由编译器自动插入，这样无需在每一个使用该变量的地方，都写一段代码来手动进行收尾工作和资源释放。
     * 指定这样一段收尾工作靠的就是 Drop 特征。
     *
     * 一个作用域结束时，除返回值外的变量/值外，变量和值有两种行为：
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
     * 一个作用域结束时，除返回值外的变量/值外，变量和值有两种行为：
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
     * `Drop::drop(&mut self)` 可以释放资源，但根据所有权模型和生命周期，由于 `&mut self` 没有转移变量所有权，手动调用会存在许多问题。
     *
     * 如果希望手动释放资源，**需要转移变量所有权**，让变量不能再使用，即让所有权模型和生命周期正常工作，rust 提供的手动释放函数 `mem::drop()` 函数非常简单：
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
     * > https://doc.rust-lang.org/std/mem/fn.drop.html
     * >
     * > https://github.com/sunface/rust-course/pull/1254
     * >
     * > 事实上，能被显式调用的 `drop(_x)` 函数只是个空函数，在拿走目标值的所有权后没有任何操作。
     * > 而由于其持有目标值的所有权，在 `drop(_x)` 函数结束之际，编译器会执行 `_x` 真正的析构函数，从而完成释放资源的操作。
     * > 换句话说，`drop(_x)` 函数只是帮助目标值的所有者提前离开了作用域。
     *
     * ### Drop::drop(&mut self) 的 &mut self
     * `Drop` 特征是rust自动清理的来源，它的职责是**执行任何必要的清理逻辑，而不是处理内存释放细节**。
     *
     * 为什么 `Drop()::drop(&mut self)` 的接收者是 `&mut self`，`&self` `self` 作为接收者有什么缺陷？
     *
     * > https://www.zhihu.com/question/612370614
     *
     * 1. 不可能是 `&self`
     * 要清理结构体内部的数据，必须能具有变量所有权或可变引用才能改变结构体数据，因此只读引用不合适。
     *
     * 2. `self` 不适合
     * 在上面的提到过：**堆上的变量和值，作用域内声明的被销毁，作用域外声明的保留**。
     *
     * `self` 接收者会转移变量的所有权，即相当于在作用域内声明了变量，在函数栈退出时就会被释放：
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
     * 从示例中可以预测到，将 `self: Self` 当作接收者，触发流程：
     * - 第一次是外部函数的函数栈退出，调用了析构函数
     * - 第二次开始是析构函数 `Drop::drop` 的函数栈退出，又调用了析构函数，形成死循环调用
     *
     * 因此 `self: Self` 是不适合作为析构函数的接收者，而接收者为 `&mut self`，可以在清理时方便的修改实例内部的信息。
     * 
     * > https://github.com/rtpacks/rust-note/blob/main/docs/unit%2047-%E7%B1%BB%E5%9E%8B%E8%BD%AC%E6%8D%A2%EF%BC%88%E4%BA%8C%EF%BC%89%E9%80%9A%E7%94%A8%E7%B1%BB%E5%9E%8B%E8%BD%AC%E6%8D%A2.md#%E7%82%B9%E6%93%8D%E4%BD%9C%E7%AC%A6
     * > 
     * > `Drop::drop(&mut self)` 是由 `x.drop()` 进行了隐式转换得来的。
     *
     * ### 互斥的 Copy 和 Drop
     * > https://github.com/sunface/rust-course/discussions/749#discussioncomment-3121717
     * 
     * Drop特征除了**不允许手动调用析构函数 `Drop::drop(&mut self)`**的限制外，还有一个限制是**一个类型不能同时实现Copy特征和Drop特征**。
     *
     * 这是因为实现了 Copy 特征的类型会被编译器隐式的复制，因此非常难以预测析构函数执行的时间和频率，因此这些实现了 Copy 特征的类型无法拥有析构函数。
     * 但从根本上理解，**一个类型不能同时实现 Copy 特征和 Drop 特征**，更重要的是**内存安全（资源正确释放）**方面的考虑。
     * 
     * copy 可以理解为栈内存的简单复制，通常意义上的**浅拷贝（trivial copy）**。
     * 
     * 简单举例：有一个结构体只包含一个指针，这个指针指向分配出来的堆内存，类似智能指针。
     * 它实现了 Drop，作用是释放堆上的内存。编译器的工作是在栈上分配类或者结构体，在离开作用域时自动插入析构函数。
     * Copy 特征会在变量赋值时把这个指针复制一遍，这时候就有两个结构体在栈上。
     * 结构体离开作用域会调用 drop，这个时候有两个结构体就会调用两遍析构，但是结构体管理的实际资源（堆上的一段内存）只有一个，此时资源就被释放两遍。
     * 这是一种内存错误。
     * 
     * 解决方法就是：
     * 1. 不仅仅复制栈上的结构，我复制这个结构体的时候把资源也复制一份。也就是clone trait。
     * 2. 使用智能指针，给资源做一个引用计数，结构体作为引用计数和资源的控制结构。每次出作用域的时候，就检查一遍引用计数，判断此时是否可以释放。
     * 3. 禁止复制，也就是 move 语义，资源的控制结构只存在一个，这个控制结构拥有所有权。move语义的赋值其实也是仅仅复制栈上的结构，但是编译器帮我记住现有栈上有效的结构到底是哪一个。要是用错了，就报错。
     * 4. 有 gc 的，就用 gc 来释放。
     * 
     * 因此，Copy和Drop互斥的最大原因是是在内存安全方面的考虑，而不仅仅因为 Copy 会复制资源。
     *
     * ### Drop 使用场景
     * 对于 Drop 而言，主要有两个功能：
     * - 回收内存资源
     * - 执行一些收尾工作
     *
     * 在绝大多数情况下无需手动 drop 回收内存资源，因为 Rust 会自动完成这些工作，它甚至会对复杂类型的每个字段都单独的调用 drop 进行回收！
     *
     * 但是确实有极少数情况，需要程序员手动回收资源的，例如文件描述符、网络 socket 等，当这些值超出作用域不再使用时，就需要进行关闭以释放相关的资源，在这些情况下，就需要使用者手动解决 Drop 的问题。
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
