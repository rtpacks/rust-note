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
}
