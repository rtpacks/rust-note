use std::vec;

fn main() {
    /*
     * ## 字符串
     * 字符串是由字符组成的连续集合，Rust的字符是 Unicode 类型，因此每个字符占据 4 个字节内存空间，但是在字符串中不一样，字符串是 UTF-8 编码，也就是字符串中的字符所占的字节数是变化的(1 - 4)，这样有助于大幅降低字符串所占用的内存空间。
     *
     * Rust的字符串比较复杂，在语言层面上，只有str一种字符串类型，它通常是以引用类型出现 &str。但在标准库中，还有多种不同用途的字符串类型，其中使用最广的即是 String 类型。
     *
     * str类型是被硬编码进二进制文件的，在整个程序运行期间str类型的地址不会变化（存储在全局内存），也无法被修改。其实str是一个切片，&str 就是 str 的切片引用
     *
     * 有关字符串的操作方法可以详细查看：https://course.rs/basic/compound-type/string-slice.html#%E6%93%8D%E4%BD%9C%E5%AD%97%E7%AC%A6%E4%B8%B2。主要关注是否修改原字符串。
     *
     */

    let c = 'a';
    let e = "Hello World";

    let mut s = String::from("Hello");
    s.push('W');
    s.push_str("orld");
    println!("{}", s);

    let s = String::from("Hello World");
    let sp = s + e; // s所有权被转移了，生成新的字符串String
    println!("{}", sp);

    let s = "中国人";

    for c in s.chars() {
        println!("{}, {}", c, s);
    }
}
