use ilearn::{run, Config};
use std::{
    array::IntoIter, collections::HashMap, convert::TryInto, env, error::Error, fmt::Display, fs,
    mem::size_of, process,
};

fn main() {
    /*
     * ## 类型转换
     * Rust 是类型安全的语言，因此在 Rust 中做类型转换不是一件简单的事。
     *
     * 首先看一段错误的实例，a 和 b 拥有不同的类型，**Rust 不允许两种不同的类型进行比较**：
     * ```rust
     * let a: i32 = 10;
     * let b: u16 = 100;
     *
     * if a < b { println!("a < b 执行"); } 代码报错，因为a和b是不同的类型，不能直接用于比较
     * ```
     * 解决方法很简单，把数据类型改成同一种就可以通过编译。针对数值类型（包括字符ASCII），rust提供了两种**专属数值类型的转换方式 `as` 和 `TryInto`**
     *
     * ### as 转换
     *
     * as 转换非常方便，但使用时需要小心。因为每个数值类型能表达的数据范围不同，如果把范围较大的类型转换成较小的类型，可能会造成数据溢出的错误。
     *
     * 因此**尽量把范围较小的类型转换成较大的类型**，来避免这些问题的发生。数值类型可以通过 `i8::MAX` 的形式查看该类型能表达的最大值。
     *
     * #### 数值与数值
     * ```rust
     * let a: i32 = 10;
     * let b: u16 = 100;
     * let transformed_a = a as u16;
     * let transformed_b = b as i32;
     * println!("{transformed_a}, {transformed_b}");
     * ```
     *
     * #### 数值与字符
     * 转换字符，只有 `u8` 类型的数据（ASCII范围）才能转换成字符，但字符可以转换成包含u8范围的数值类型，如i16，u16等
     * ```rust
     * let a = 'a';
     * let transformed_a = a as u8;
     * let c: u8 = 99;
     * let transformed_c = c as char;
     * println!("{transformed_a}, {transformed_c}");
     * ```
     *
     * #### 数值与内存指针
     *
     * rust 指针可通过 `as_mut_ptr` 获取，它的形式与 `C` 一样，数组的指针也是第一个元素的指针。
     *
     * 指针常用 usize（4 bytes(32 bit) 或 8 bytes(64 bit)）表示。
     *
     * > 数组指针偏移，第N+1元素地址 = 第 N 元素地址 + 元素内存大小
     *
     * ```rust
     * let mut v = vec![0; 3];
     * let p1: *mut i32 = v.as_mut_ptr();
     * let first_address = p1 as usize;
     * let second_address = first_address + size_of::<i32>(); // 数组指针偏移，第N+1元素地址 = 第 N 元素地址 + 元素大小
     * let p2 = second_address as *mut i32;
     * unsafe {
     *     *p2 += 1;
     * }
     * println!(
     *     "v = {:?}, second_address = {:?} p2 = {:?}",
     *     v, second_address, p2
     * );
     * ```
     *
     * #### 强制类型转换的边角知识
     * - 转换不具有传递性 就算 e as U1 as U2 是合法的，也不能说明 e as U2 是合法的（e 不能直接转换成 U2），因为as关键字执行的转换可能涉及到具体的转换逻辑和额外的步骤，这些逻辑在不同的转换路径中可能不适用。
     *
     * ### TryInto 特征
     * 在一些场景中，使用 as 关键字会有比较大的限制，比如转换失败直接panic。如果希望在类型转换上拥有完全的控制而不依赖内置的转换，例如处理转换错误，那么可以使用 TryInto 特征。
     *
     * **如果你要使用一个特征的方法，那么你需要引入该特征到当前的作用域中**，如果希望使用 try_into 方法，需要引入对应的 TryInto 特征。
     * > 在1.75.0，TryInto 已通过 `std::prelude` 预导入，无需手动 use 手动导入。
     *
     * try_into 方法会尝试进行一次转换，并返回一个 Result，此时就可以对其进行相应的错误处理（使用时需要给结果标注转换后的类型）。
     * 
     * ```rust
     * use std::convert::TryInto;
     * let a = 8 as i32;
     * let b: u8 = a.try_into().expect("转换失败");
     * let c: u8 = match a.try_into() {
     *     Ok(v) => v,
     *     Err(e) => panic!("expr"),
     * };
     * println!("{b}, {c}");
     * ```
     * > 巩固rust认识
     * > rust编译的过程，会根据给定的类型生成对应的代码，给变量标注类型和给表达式指定类型的效果是一样的：
     * > ```rust
     * > let a = 8 as u8;
     * > let b: u8 = 8;
     * > ```
     *
     */

    let a: i32 = 10;
    let b: u16 = 100;
    // if a < b { println!("a < b 执行了") } 代码报错，因为 a 和 b是不同的类型，rust不允许两种不同的类型进行比较
    println!("{:?}", i8::MIN);
    println!("{:?}", u8::MIN == 0);

    // 数值与数值
    let transformed_a = a as u16;
    let transformed_b = b as i32;
    println!("{transformed_a}, {transformed_b}");

    // 数值与字符
    let a = 'a';
    let transformed_a = a as u32;
    let c: u8 = 99;
    let transformed_c = c as char;
    println!("{transformed_a}, {transformed_c}");

    // 数值与内存指针
    let mut v = vec![0; 3];
    let p1: *mut i32 = v.as_mut_ptr();
    let first_address = p1 as usize;
    let second_address = first_address + size_of::<i32>();
    let p2 = second_address as *mut i32;
    unsafe {
        *p2 += 1;
    }
    println!(
        "v = {:?}, second_address = {:?} p2 = {:?}",
        v, second_address, p2
    );

    let a = 8 as i32;
    let b_1: u8 = a.try_into().unwrap();
    let b: u8 = a.try_into().expect("转换失败");
    let c: u8 = match a.try_into() {
        Ok(v) => v,
        Err(e) => panic!("expr"),
    };
    println!("{b}, {c}");
}
