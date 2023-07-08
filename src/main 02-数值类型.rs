use num::complex::Complex;

fn main() {
    /*
     * 基础数据类型
     *
     * ## 数值类型
     * https://course.rs/basic/base-type/numbers.html
     *
     * 整形类型，默认为i32，即有符号32位。i有符号，u无符号，(i|u)size视架构定。实现std::cmp::Eq特征，即完全比较。
     * i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize
     *
     * 浮点类型，默认为f64，即双精度，实现的是std::cmp::PartialEq特征，即部分比较。
     *
     * ## 浮点数陷阱
     * 0.1 + 0.2 != 0.3 在低精度条件下成立，高精度条件下不成立！这是由于二进制无法精确表达0.2
     * f32条件下 0.1 + 0.2: 3e99999a，0.3: 3e99999a
     * f64条件下 0.1 + 0.2: 3fd3333333333334，0.3: 3fd3333333333333
     *
     * ## NaN
     * 数学上未定义的数值类型，例如对负数取平方根 -42.1.sqrt()，所有跟 NaN 交互的操作，都会返回一个 NaN，而且 NaN 不能用来比较。
     * NaN != Nan，不能直接比较，而需要使用 number.is_nan()方法确定。
     *   let x = (-42.0_f32).sqrt(); assert_eq!(x, x);
     *
     * 相同类型的数字才能进行运算！才能赋值！
     * let v: u16 = 38_u8 as u16;
     *
     * ## Range 序列
     * 快速生成指定范围的类数值类型，如 1..=5，'a'..='z'，序列只允许用于数字或字符类型。
     */
    let a = 1;

    if (-42f32).sqrt() == (-42f32).sqrt() {
        // println!("Nan可以直接比较，即可以使用NaN==NaN");
    } else {
        // println!(
        //     "NaN不可以直接比较，即不可以直接使用NaN==NaN比较，需要使用 number.is_nan() 测试比较"
        // );
        // println!("{}", -1_f64.sqrt() == f64::NAN);
    }

    // let a = 12_i32 + 14_i64; error
    let a = 12_i32 + 14_i64 as i32;
    println!("{}", a);

    // 序列
    for n in 1..=5 {
        // println!("{}", n);
    }

    for c in 'a'..='z' {
        // println!("{}", c)
    }

    // num库
    let a = Complex { re: 1, im: 1 };

    let b = Complex::new(1, 1);

    let result = a + b;

    println!("{} + {}i", result.re, result.im);
}
