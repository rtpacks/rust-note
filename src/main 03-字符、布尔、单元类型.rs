use num::complex::Complex;

fn main() {
    /*
     * ## 字符 char
     * Unicode编码的值都是合法的Rust字符，如英文，中文，emoji等。Unicode编码是四个字节，所以Rust的字符也是四个字节大小。可以使用标准库 std::mem::size_of_val() 函数来获取。
     *
     * ## 布尔值 bool
     * Rust中布尔值占一个字节大小。
     *
     * ## 单元类型 ()
     * 单元类型就是 `()`，唯一的值也是 `()` ，
     * 单元类型是一个很简单的定义类型，它可以是一个函数的返回值，标识函数返回值不为空，不占用内存。
     */
    println!("unit3");
    let a = '中';
    let b = 'b';
    println!(
        "{}, {}",
        std::mem::size_of_val(&a),
        std::mem::size_of_val(&b)
    );

    // bool
    println!("{}, {}", 1 == 1, 1 != 1);
}
