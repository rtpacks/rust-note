## Rust Note

参考书籍 | Reference Books

- https://doc.rust-lang.org/stable/book/
- https://course.rs/
- https://kaisery.github.io/trpl-zh-cn/
- https://rustwiki.org/zh-CN/book/
- https://rustwiki.org/zh-CN/rust-by-example/
- https://rustwiki.org/zh-CN/reference/

### Iteration
rust 的迭代器并不是一个迭代器处理完集合中的所有数据后再传递给下一个迭代器处理，它的设计更像是中间件，即不同方法的组合。这与 JavaScript 不相同。

对于一条数据，当前迭代器的逻辑处理完成后，就会给到下一个迭代器处理。并不是等收集所有数据，在一个迭代器中处理完成这些数据后再给到下一个迭代器。