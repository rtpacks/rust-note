use num::complex::Complex;

fn main() {
    /*
     * ## 字符 char
     * Unicode编码的值都是合法的Rust字符，如英文，中文，emoji等。Unicode编码是四个字节，所以Rust的字符也是四个字节大小。可以使用标准库 std::mem::size_of_val() 函数来获取。
     * 所以相比较于其他语言，如c中的字符表示ASCII，是八位即一字节的字符。Rust中的字符改变时32位，用来存储Unicode的。
     *
     * ## 布尔值 bool
     * Rust中布尔值占一个字节大小。
     *
     * ## 单元类型 ()
     * 单元类型就是 `()`，唯一的值也是 `()` ，
     * 单元类型是一个很简单的定义类型，它可以是一个函数的返回值，标识函数返回值不为空，不占用内存。
     *
     * 一个类型需要多少多少bit来存储是用对数来算的, bool需要1bit是因为有2种值true和false, ()这种类型只有一种值, 那么log2(0)=0, 也就是0bit来存储.
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
