use core::fmt;
use std::{io, num};

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
     * 在编写 unsafe 函数时，有一点需要注意：unsafe 函数体中无需使用 unsafe 语句块，unsafe 函数自身就是一个 unsafe 语句块。
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
}
