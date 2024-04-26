use ilearn::{run, Config};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    fmt::{Debug, Display},
    ops::{Add, Index},
};

fn main() {
    /*
     * ## Deref 解引用
     *
     * 在类型转换（二）通用类型转换中，有一个步骤是自动解引用，这里的自动解引用就和 Deref 特征相关：
     * 1. 编译器检查它是否可以直接调用 T::foo(value)，即检查类型是否具有foo方法，称之为**值方法调用**
     * 2. 如果值方法调用无法完成(例如方法类型错误或者类型没有对应函数的 Self 进行实现)，那么编译器会尝试**增加自动引用**，会尝试以下调用： `<&T>::foo(value)` 和 `<&mut T>::foo(value)`，称之为**引用方法调用**
     * 3. 如果值方法和引用方法两个方法不工作，编译器会试着**解引用 T** ，然后再进行尝试。这里使用了 `Deref` 特征 —— 若 `T: Deref<Target = U>` (T 可以被解引用为 U)，那么编译器会使用 U 类型进行尝试，称之为**解引用方法调用**
     * 4. 如果 T 不能被解引用，且 T 是一个定长类型(在编译期类型长度是已知的)，那么编译器也会尝试将 T 从**定长类型转为不定长类型**，例如将 [i32; 2] 转为 [i32]
     * 5. 如果以上方式均不成功，那编译器将报错
     *
     * ### 通过 `*` 获取引用背后的值
     * > Rust 会在方法调用和字段访问时自动应用解引用强制多态（deref coercions），在一些其他情况下，如在标准比较操作或赋值中，Rust 不会自动应用解引用：**在表达式中不能自动地执行隐式 Deref 解引用操作**。
     * > println! 实际上调用的就是Display特征的方法，所以println时存在自动解引用
     *
     * Deref 特征不仅可以自动解引用智能指针（引用），还可以解引用常规引用。
     *
     *
     * 常规引用是一个指针类型，**包含目标数据存储的内存地址**。对常规引用使用 `*` 操作符，就可以通过解引用的方式获取到内存地址对应的数据值：
     * ```rust
     * let x = 5;
     * let y = &5;
     * // println!("{}", x == y); 在标准比较或赋值中，rust不会自动应用解引用，因此不能直接比较
     * println!("{}, {}, {}", x, y, *y); // 可以自动解引用
     * ```
     * 
     * ### 智能指针解引用
     * 常规指针的解引用与大多数语言并无区别，但 Rust 的解引用功能更为丰富，Rust 将其提升到了一个新高度。
     * 
     * 考虑一下智能指针，它是一个结构体类型，如果直接对它进行解引用 `*myStruct`，显然编译器不知道该如何办，因此我们可以为智能指针结构体实现 Deref 特征。
     * 
     */

    let x = 5;
    let y = &5;
    // println!("{}", x == y); 在标准比较或赋值中，rust不会自动应用解引用，因此不能直接比较
    println!("{}, {}, {}", x, y, *y); // 可以自动解引用
}
