## 字符 char

Unicode 编码的值都是合法的 Rust 字符，如英文，中文，emoji 等。Unicode 编码是四个字节，所以 Rust 的字符也是四个字节大小。可以使用标准库 d::mem::size_of_val() 函数来获取。
所以相比较于其他语言，如 c 中的字符表示 ASCII，是八位即一字节的字符。Rust 中的字符改变时 32 位，用来存储 Unicode 的。

## 布尔值 bool

Rust 中布尔值占一个字节大小。

## 单元类型 ()

单元类型就是 `()`，唯一的值也是 `()` ，
单元类型是一个很简单的定义类型，它可以是一个函数的返回值，标识函数返回值不为空，不占用内存。

一个类型需要多少多少 bit 来存储是用对数来算的, bool 需要 1bit 是因为有 2 种值 true 和 false, ()这种类型只有一种值, 那么 log2(0)=0, 也就是 0bit 来存储.
