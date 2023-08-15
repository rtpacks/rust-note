use std::vec;

fn main() {
    /*
     * ## 结构体
     * Struct是Rust中非常重要的一种数据类型，它可以容纳各种类型的数据，并且在存放数据的基本功能上之外还提供一些其他功能，比如可以为Struct类型定义方法。
     * 实际上，Struct类型类似于面向对象的类，Struct的实例则类似于对象。Struct的实例和面向对象中的对象都可以看作是使用key-value模式的hash结构去存储数据，同时附带一些其他功能。
     * 
     * 由于Struct的复杂性，Rust不能直接打印Struct，需要使用 #[derive(Debug)] 对结构体进行标记。使用 {:?} 或 {:#?} 格式化输出。
     *
     * ### 定义Struct
     * 使用struct关键字定义Struct类型。具名Struct(named Struct)表示有字段名称的Struct。Struct的字段(Field)也可以称为Struct的属性(Attribute)。
     * - 结构体尽量不使用引用类型的属性，否则需要管理生命周期 `'`
     * - 当要构造的Struct实例的字段值来自于变量，且这个变量名和字段名相同，则可以简写该字段。
     * - 使用 `..` 可以快速生成Struct实例，注意原有实例字段可能会被转移所有权！如果部分字段被转移，该部分字段和原有实例都不允许访问。如果某部分字段实现Copy Trait，则允许继续使用。
     *
     * ```rs
     * #[drive(Debug)]
     * struct Person {
     *      age: i32;
     *      name: String;
     * }
     * let age = 18;
     *
     * let p = Person { age, name: String::from("zhangsan") };
     * let p2 = Person { ..p }; // name所有权被转移
     *
     * println!("{:?}", p2); // 打印结构体
     * println!("{:?}", p.age); // right，允许继续使用
     * println!("{:?}", p); // error name字段被转移所有权，不允许访问
     * println!("{:?}", p.name); // error 被转移所有权，不允许访问
     * ```
     * 
     * 结构体内存分布可以查看：https://course.rs/basic/compound-type/struct.html#%E7%BB%93%E6%9E%84%E4%BD%93%E7%9A%84%E5%86%85%E5%AD%98%E6%8E%92%E5%88%97
     * 
     * 结构体所有权可以查看：https://course.rs/basic/compound-type/struct.html#%E7%BB%93%E6%9E%84%E4%BD%93%E6%95%B0%E6%8D%AE%E7%9A%84%E6%89%80%E6%9C%89%E6%9D%83
     *
     * ### 其他结构体
     * tuple struct，除了named struct外，Rust还支持没有字段名的struct结构体，称为元组结构体(tuple struct)。
     * ```rs
     * struct Color(i32, i32, i32);
     * struct Point(i32, i32, i32);
     * let black = Color(0, 0, 0);
     * let origin = Point(0, 0, 0);
     * ```
     * 虽然Color和Point的属性相同，但是Color和Point是不同的类型，是不同的结构体。在其他方面，元组结构体实例类似于元组：可以将其解构，也可以使用.后跟索引来访问单独的值，等等。
     * 
     * unit-like-struct，类单元结构体(unit-like struct)是没有任何字段的空struct。
     * ```rs
     * struct St;
     * ```
     */

    #[derive(Debug)]
    struct Person {
        age: i32,
    };

    let p = Person { age: 1 };

    println!("{:?}", p);
}
