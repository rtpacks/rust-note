use core::fmt;
use std::{io, num, slice};

fn main() {
    /*
     *
     * ## unsafe：unsafe superpowers
     *
     * 五种超能力（unsafe superpowers）：
     * - 解引用裸指针
     * - 调用不安全的函数或方法
     * - 访问或修改可变静态变量
     * - 实现不安全 trait
     * - 访问 union 的字段
     *
     * ### 解引用裸指针
     *
     * 裸指针(raw pointer，又称原生指针) 在功能上跟引用类似，它需要显式地注明可变性。
     * 但是裸指针又和引用有所不同，裸指针有两种形式: `*const T` 和 `*mut T`，代表不可变和可变。
     * `*` 操作符常见的含义是用于解引用，但是在裸指针 `*const T` 和 `*mut` 中，`*` 只是类型名称的一部分，并没有解引用的含义。
     *
     * 截至目前，已经有三种类似指针的概念：**引用、智能指针和裸指针**。裸指针与引用、智能指针不同：
     * - 可以绕过 Rust 的借用规则，可以同时拥有一个数据的可变、不可变指针，甚至还能拥有多个可变的指针
     * - 不能保证指向的内存是合法的
     * - 可以是 null
     * - 没有实现任何自动的回收 (drop)
     *
     * 使用裸指针可以创建两个可变指针都指向同一个数据，如果使用安全的 Rust 是无法做到这一点的，因为违背了借用规则。
     * 因此虽然裸指针可以绕过借用规则，但是由此带来的数据竞争问题，需要程序员着重处理。
     *
     * 总之，裸指针跟 C 指针是非常像的，使用它需要以牺牲安全性为前提，但获得了更好的性能，也可以跟其它语言或硬件打交道。
     *
     * #### 基于引用创建裸指针
     * 需要注意：**基于引用创建裸指针是安全的行为，而解引用裸指针才是不安全的行为**。即基于引用创建裸指针时不需要 unsafe，解引用时才需要。
     *
     * ```rust
     * // 基于引用创建裸指针是安全的行为，解引用裸指针才是不安全的
     * let mut num = 3;
     * let num_ptr = &num as *const i32; // 创建裸指针是安全的
     * let num_mutptr1 = &mut num as *mut i32; // 创建可变的裸指针，与不可变裸指针存储是一样的地址，但语义上是区分的
     * let num_mutptr2 = &mut num as *mut i32; // 裸指针是可以创建多个可变的
     * unsafe {
     *     // *num_ptr = 4;
     *     *num_mutptr1 = 4;
     *     *num_mutptr2 = 4;
     * }
     * println!(
     *     "num = {}, num_ptr = {:p}, num_mutptr1 = {:p}, num_mutptr2 = {:p}",
     *     num, num_ptr, num_mutptr1, num_mutptr2
     * );
     * ```
     *
     * #### 基于智能指针创建裸指针
     * 与基于引用创建裸指针很类似，基于智能指针创建裸指针是安全的，解引用才是不安全的行为。
     * ```rust
     * // 基于智能指针创建裸指针
     * let mut num_box = Box::new(2);
     * let num_box_ptr = &*num_box as *const i32;
     * let num_box_mutptr = &mut *num_box as *mut i32;
     * unsafe {
     *     // *num_box_ptr = 4;
     *     *num_box_mutptr = 4;
     * }
     * println!(
     *     "num_box = {}, num_box_ptr = {:p}, num_box_mutptr = {:p}",
     *     num_box, num_box_ptr, num_box_mutptr
     * )
     * ```
     *
     * #### 基于内存地址创建裸指针
     * 基于一个内存地址来创建裸指针，可以想像这种行为是相当危险的。试图使用任意的内存地址往往是一种未定义的行为(undefined behavior)，因为该内存地址有可能存在值，也有可能没有。
     * 同时编译器也有可能会优化这段代码，会造成没有任何内存访问发生，甚至程序还可能发生段错误(segmentation fault)。
     *
     * 正常项目几乎不会基于内存地址创建裸指针的做法。
     *
     *
     * ### 调用 unsafe 函数或方法
     * unsafe 函数从外表上来看跟普通函数并无区别，唯一的区别就是它需要使用 unsafe fn 来进行定义。
     * 这种定义方式是为了告诉调用者：当调用此函数时需要注意它的相关需求，因为 Rust 无法担保调用者在使用该函数时能满足它所需的一切需求。
     *
     * 在编写 unsafe 函数时，有一点需要注意：
     * unsafe 函数体中无需使用 unsafe 语句块，unsafe 函数自身就是一个 unsafe 语句块，但一个函数包含了 unsafe 代码不代表需要将整个函数都定义为 unsafe fn。
     *
     * ```rust
     * unsafe fn gen_unsafe() {
     *     // 基于智能指针创建裸指针
     *     let mut num_box = Box::new(2);
     *     let num_box_ptr = &*num_box as *const i32;
     *     let num_box_mutptr = &mut *num_box as *mut i32;
     *     // *num_box_ptr = 4;
     *     *num_box_mutptr = 4; // unsafe函数中无需unsafe语句块
     *
     *     println!(
     *         "gen_unsafe: num_box = {}, num_box_ptr = {:p}, num_box_mutptr = {:p}",
     *         num_box, num_box_ptr, num_box_mutptr
     *     );
     * }
     * unsafe { gen_unsafe() }
     * ```
     *
     * ### 安全抽象包裹 unsafe 代码
     *
     * 一个函数包含了 unsafe 代码不代表需要将整个函数都定义为 unsafe fn。事实上，在标准库中有大量的安全函数，它们内部都包含了 unsafe 代码块，例如 split_at_mut。
     *
     * 需求：将一个数组分成两个切片，且每一个切片都要求是可变的。类似这种需求在安全 Rust 中是很难实现的，因为要对同一个数组做两个可变借用，这不符合借用规则。
     * ```rust
     * fn split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
     *     let len = slice.len();
     *     assert!(mid <= len);
     *
     *     (&mut slice[..mid], &mut slice[mid..]) // 出现多个可变借用，不符合 rust 的借用规则，编译失败
     * }
     * ```
     *
     * 使用 unsafe 绕过借用规则
     * ```rust
     * // 安全抽象包裹 unsafe 代码，即将一个unsafe语句块放在安全的rust中
     * fn split_at_mut(_slice: &mut [i32], point: usize) -> (&mut [i32], &mut [i32]) {
     *     let len = _slice.len();
     *     assert!(point < len);
     *
     *     let ptr = _slice.as_mut_ptr();
     *     // (&mut _slice[..point], &mut _slice[point..]) 出现多个可变借用，不符合 rust 的借用规则，编译失败
     *     unsafe {
     *         // 从可变裸指针获取可变引用
     *         (
     *             slice::from_raw_parts_mut(ptr, point),
     *             slice::from_raw_parts_mut(ptr.add(point), len - point),
     *         )
     *     }
     * }
     * let mut arr = [1, 2, 3, 4];
     * println!("{:?}", split_at_mut(&mut arr, 2));
     * ```
     *
     * 有几点需要注意：
     * as_mut_ptr 会返回指向 slice 首地址的裸指针 `*mut i32`
     * slice::from_raw_parts_mut 方法通过指针和长度来创建一个新的切片，是一个unsafe方法。简单来说，该切片的初始地址是 ptr，长度为 point
     * ptr.add(point) 可以获取第二个切片的初始地址，是一个unsafe方法。由于切片中的元素是 i32 类型，每个元素都占用了 4 个字节的内存大小，因此不能简单的用 `ptr + mid` 来作为初始地址，而应该使用 `ptr + 4 * mid`，但是这种使用方式并不安全，因此 .add 方法是最佳选择
     *
     * ```rust
     *  // 安全抽象包裹 unsafe 代码，即将一个unsafe语句块放在安全的rust中
     *  fn split_at_mut(_slice: &mut [i32], point: usize) -> (&mut [i32], &mut [i32]) {
     *      let len = _slice.len();
     *      assert!(point < len);
     *
     *      let ptr = _slice.as_mut_ptr();
     *      // (&mut _slice[..point], &mut _slice[point..]) 出现多个可变借用，不符合 rust 的借用规则，编译失败
     *      unsafe {
     *          // 从可变裸指针获取可变引用
     *          (
     *              slice::from_raw_parts_mut(ptr, point), // from_raw_parts_mut 通过指针和长度来创建一个新的切片，是一个unsafe方法
     *              slice::from_raw_parts_mut(ptr.add(point), len - point), // ptr.add(point) 可以获取第二个切片的初始地址，是一个unsafe方法
     *          )
     *      }
     *  }
     *  let mut arr = [1, 2, 3, 4];
     *  println!("{:?}", split_at_mut(&mut arr, 2));
     * ```
     *
     * ### FFI 外部函数接口
     * FFI（Foreign Function Interface）外部函数接口是用来与其它语言进行交互的接口设计，但并不是所有语言都称为 FFI。例如在 Java 中称之为 JNI（Java Native Interface）。
     *
     * FFI 之所以存在是现实中很多代码库都是由不同语言编写的，如果需要使用某个库，但它是由其它语言编写的，往往只有几个选择：
     * - 对该库进行重写或者移植
     * - 独立的服务调用（HTTP，gRPC）
     * - 使用 FFI
     *
     * 在大部分情况下，重写或移植程序需要花费大量的时间和精力，独立的服务调用可能不满足时延，此时 FFI 就是最佳选择。
     * 并且，在将其他语言的代码重构为 Rust 时，先将相关代码引入到 Rust 项目中，然后逐步重构，是一个非常不错的渐进式过程。
     *
     * 涉及到不同语言的交互，无法确定这个行为是否安全，因此 rust 的 FFI 需要 unsafe 的支持才能绕过编译器的审查，达到正常编译的目的。
     *
     * ```rust
     * extern "C" {
     *     fn abs(input: i32) -> i32;
     * }
     *
     * unsafe {
     *     println!("Absolute value of -3 according to C: {}", abs(-3));
     * }
     * ```
     * C 语言的代码定义在了 extern 代码块中， 而 extern 必须使用 unsafe 才能进行进行调用，原因在于其它语言的代码并不会强制执行 Rust 的规则，因此 Rust 无法对这些代码进行检查，最终还是要靠开发者自己来保证代码的正确性和程序的安全性。
     *
     * > 阅读：
     * > - https://rustwiki.org/zh-CN/book/ch19-01-unsafe-rust.html#使用-extern-函数调用外部代码
     *
     * #### ABI
     * **应用二进制接口 ABI (Application Binary Interface) 定义了如何在汇编层面来调用该函数**。
     *
     * 在 extern "C" 代码块列出想要调用的外部函数的签名。其中 "C" 定义了外部函数所使用的 ABI。在所有 ABI 中，C 语言的是最常见的。
     *
     *
     * #### 其它语言调用 Rust 函数
     * FFI 支持 rust 调用其他语言，也支持其他语言调用 rust。方法是使用 extern 来创建一个接口，其它语言可以通过该接口来调用相关的 Rust 函数。
     *
     * 供其他语言调用的 FFI 语法与调用其他语言的 FFI 有所不同，调用其他语言使用 extern 语句块，供其他语言调用是在函数定义时加上 extern 关键字。
     * 除了加上 extern 关键字外，还需要加上 `#[no_mangle]` 注解，它的作用是告诉 Rust 编译器不要乱改函数的名称。
     *
     * > Mangling：rust 编译时可能需要修改函数的名称，目的是为了让名称包含更多的信息，这样其它的编译部分就能从该名称获取相应的信息，这种修改会导致函数名变得相当不可读，并且使原函数名称失效。
     *
     * ```rust
     * #[no_mangle]
     * pub extern "C" fn call_from_c() {
     *     println!("Just called a Rust function from C!");
     * }
     * ```
     *
     * ### 访问或修改可变静态变量
     * 在之前的全局变量章节中有介绍。
     *
     * ### unsafe 特征
     * 之所以会有 unsafe 的特征，是因为该特征至少有一个方法包含有编译器无法验证的内容。unsafe 的特征并不常见，已接触的只有 Send。
     * unsafe 特征需要使用 unsafe impl 实现方法，unsafe impl 通知编译器，程序相应的正确性由程序员保证。
     *
     * 阅读：https://course.rs/advance/unsafe/superpowers.html#实现-unsafe-特征
     *
     * ### 访问 union 中的字段
     * union 主要用于跟 C 代码进行交互，访问 union 的字段是不安全的，因为 Rust 无法保证当前存储在 union 实例中的数据类型。
     *
     * ```rust
     * #[repr(C)]
     * union MyUnion {
     *     f1: u32,
     *     f2: f32,
     * }
     * ```
     * union 的使用方式与结构体很相似，但是 union 的所有字段都共享同一个存储空间，意味着往 union 的某个字段写入值，会导致其它字段的值会被覆盖。
     *
     * ### 实用工具库
     * unsafe 和 FFI 在 Rust 的使用场景中是相当常见，因此社区已经开发出一些实用的工具，可以改善相应的开发体验。这一部分可以在开发中尝试不同的工具。
     *
     * #### rust-bindgen 和 cbindgen
     * 对于 FFI 调用来说，保证接口的正确性是非常重要的，这两个库可以帮我们自动生成相应的接口。
     * 其中 rust-bindgen 用于生成在 Rust 中访问 C 的代码，而 cbindgen 则相反，用于生成在 C 中访问 Rust 的代码。
     *
     * #### cxx
     * 如果需要跟 C++ 代码交互，则推荐使用 cxx，它提供了双向的调用，最大的优点就是安全，无需使用 unsafe 语句块。
     *
     * #### Miri
     * miri 可以生成 Rust 的中间层表示 MIR，它可以帮助检查常见的未定义行为(UB = Undefined Behavior)，例如
     * - 内存越界检查和内存释放后再使用(use-after-free)
     * - 使用未初始化的数据
     * - 数据竞争
     * - 内存对齐问题
     *
     * 可以通过 rustup component add miri 来安装它，并通过 cargo miri 来使用，同时还可以使用 cargo miri test 来运行测试代码。
     * 但需要注意的是，它只能帮助识别被执行代码路径的风险，那些未被执行到的代码是没办法被识别的。
     *
     * #### Prusti
     * prusti 需要自己来构建一个证明，然后通过它证明代码中的不变量是正确被使用的，当在安全代码中使用不安全的不变量时，就会非常有用。
     * 阅读：https://viperproject.github.io/prusti-dev/user-guide/
     *
     * #### Clippy
     * 官方的 clippy 检查器提供了有限的 unsafe 支持，虽然不多但是至少有一定帮助。例如 missing_safety_docs 检查可以帮助检查哪些 unsafe 函数遗漏了文档。
     * 需要注意的是：Rust 编译器并不会默认开启所有检查，可以调用 rustc -W help 来看看最新的信息。
     *
     * #### 模糊测试(fuzz testing)
     * 在 Rust Fuzz Book 中列出了一些 Rust 可以使用的模糊测试方法。同时还可以使用 rutenspitz 这个过程宏来测试有状态的代码，例如数据结构。
     *
     * ### 总结
     * unsafe 只应该用于仅限的五种场景，其它场景应该坚决的使用安全的代码。
     * 总之，能不使用 unsafe 一定不要使用，就算使用也要控制好边界，让范围尽可能的小，只有真的需要 unsafe 的代码才应该包含其中, 而不是将无关代码也纳入进来。
     * 
     * ### 进一步学习
     * - https://blog.logrocket.com/unsafe-rust-how-and-when-not-to-use-it/
     *
     *
     *
     *
     *
     *
     */

    // 基于引用创建裸指针是安全的行为，解引用裸指针才是不安全的
    let mut num = 3;
    let num_ptr = &num as *const i32; // 创建裸指针是安全的
    let num_mutptr1 = &mut num as *mut i32; // 创建可变的裸指针，与不可变裸指针存储是一样的地址，但语义上是区分的
    let num_mutptr2 = &mut num as *mut i32; // 裸指针是可以创建多个可变的
    unsafe {
        // *num_ptr = 4;
        *num_mutptr1 = 4;
        *num_mutptr2 = 4;
    }
    println!(
        "num = {}, num_ptr = {:p}, num_mutptr1 = {:p}, num_mutptr2 = {:p}",
        num, num_ptr, num_mutptr1, num_mutptr2
    );

    // 基于智能指针创建裸指针
    let mut num_box = Box::new(2);
    let num_box_ptr = &*num_box as *const i32;
    let num_box_mutptr = &mut *num_box as *mut i32;
    unsafe {
        // *num_box_ptr = 4;
        *num_box_mutptr = 4;
    }
    println!(
        "num_box = {}, num_box_ptr = {:p}, num_box_mutptr = {:p}",
        num_box, num_box_ptr, num_box_mutptr
    );

    unsafe fn gen_unsafe() {
        // 基于智能指针创建裸指针
        let mut num_box = Box::new(2);
        let num_box_ptr = &*num_box as *const i32;
        let num_box_mutptr = &mut *num_box as *mut i32;
        // *num_box_ptr = 4;
        *num_box_mutptr = 4;

        println!(
            "gen_unsafe: num_box = {}, num_box_ptr = {:p}, num_box_mutptr = {:p}",
            num_box, num_box_ptr, num_box_mutptr
        );
    }
    unsafe { gen_unsafe() }

    // 安全抽象包裹 unsafe 代码，即将一个unsafe语句块放在安全的rust中
    fn split_at_mut(_slice: &mut [i32], point: usize) -> (&mut [i32], &mut [i32]) {
        let len = _slice.len();
        assert!(point < len);

        let ptr = _slice.as_mut_ptr();
        // (&mut _slice[..point], &mut _slice[point..]) 出现多个可变借用，不符合 rust 的借用规则，编译失败
        unsafe {
            // 从可变裸指针获取可变引用
            (
                slice::from_raw_parts_mut(ptr, point), // from_raw_parts_mut 通过指针和长度来创建一个新的切片，是一个unsafe方法
                slice::from_raw_parts_mut(ptr.add(point), len - point), // ptr.add(point) 可以获取第二个切片的初始地址，是一个unsafe方法
            )
        }
    }
    let mut arr = [1, 2, 3, 4];
    println!("{:?}", split_at_mut(&mut arr, 2));

    // FFI
    extern "C" {
        fn abs(input: i32) -> i32;
    }
    unsafe { println!("{}", abs(-3)) }
}
