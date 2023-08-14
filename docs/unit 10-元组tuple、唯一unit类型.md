## 元组

Rust 的 tuple 类型可以存放 0 个、1 个或多个任意数据类型的数据，这些数据内容和顺序是固定的。使用 tup.N 的方式可以访问索引为 N 的元素。

- https://rust-book.junmajinlong.com/ch3/05_tuple_unit.html
- https://course.rs/basic/compound-type/tuple.html

```rs
let tup = (1, 2.3, 4);
let first = tup.0;
let (first, second, thirf) = tup;

```

注意，访问 tuple 元素的索引必须是编译期间就能确定的数值，而不能是变量。当 tup 只有一个元素时，不能省略逗号！

```rs
let tup = (1, 2, 3);
let tup = ("Hello", ); // 不能省略逗号，用于判断类型
```

## unit 唯一类型

不保存任何数据的 tuple 表示为()。在 Rust 中，它是特殊的，它有自己的类型：unit。unit 类型的写法为()，该类型也只有一个值，写法仍然是()。

```rs
//       类型  值
let unit: () = ()
```

unit 类型通常用在那些不关心返回值的函数中。在其他语言中，那些不写 return 语句或 return 不指定返回内容的的函数，一般表示不关心返回值。在 Rust 中可将这种需求写为 return ()。
