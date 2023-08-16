use std::vec;

fn main() {
    /*
     * ## 结构体（二）
     * 通过结构体，我们可以将相关联的数据片段联系起来并命名它们，这样可以使得代码更加清晰。在 impl 块中，你可以定义与你的类型相关联的函数，而方法是一种相关联的函数，让你指定结构体的实例所具有的行为。
     *
     * Struct就像面向对象的类一样，Rust允许为Struct定义实例方法和关联函数，实例方法可被所有实例对象访问调用，关联函数类似于其他语言的类函数或静态方法。
     * - 实例方法第一个参数是self（的各种形式），实例方法是所有实例对象可访问、调用的方法，由实例调用。
     * - 关联函数是指第一个参数不是self(的各种形式)但和Struct有关联关系的函数，由结构体直接调用，称为关联函数(associate functions)，如String::from("")。
     *
     * 什么是self呢？self表示调用方法时的Struct实例对象(如person.speak()时，self就是person，是具体的实例)，相当于其他语言中的this。
     * self的类型是Self（大写），在结构体内部使用Self代替结构体本身，即Self是结构体的别名，比如Person结构体实例方法的第一个参数self，它的类型就是结构体自身Self，也就是Person，这样在编写代码时无需重复指定类型。
     *
     * 上面说到，实例方法的第一个参数是self，self的类型是结构体类型Self，结合所有权可以得到下面三种形式
     * - `fn f(self: Self)`：当obj.f()时，转移obj的所有权，调用f方法之后，obj将无效
     * - `fn f(self: &Self)`：当obj.f()时，借用而非转移obj的只读权，方法内部不可修改obj的属性，调用f方法之后，obj依然可用
     * - `fn f(self: &mut Self)`：当obj.f()时，借用obj的可写权，方法内部可修改obj的属性，调用f方法之后，obj依然可用
     *
     * 可以使用以下写法再次简化表达形式
     * - `fn f(self)`：当obj.f()时，转移obj的所有权，调用f方法之后，obj将无效
     * - `fn f(&self)`：当obj.f()时，借用而非转移obj的只读权，方法内部不可修改obj的属性，调用f方法之后，obj依然可用
     * - `fn f(&mut self)`：当obj.f()时，借用obj的可写权，方法内部可修改obj的属性，调用f方法之后，obj依然可用
     *
     * ### 拓展
     * 与字段同名的方法将被定义为只返回字段中的值，而不做其他事情。这样的方法被称为 getters，Rust 并不像其他一些语言那样为结构字段自动实现它们。Getters 很有用，因为你可以把字段变成私有的，但方法是公共的，这样就可以把对字段的只读访问作为该类型公共 API 的一部分。
     *
     * 实际上，实例方法也属于关联函数。当作关联函数调用时，需要手动传递一个self。
     * ```rs
     * Person::plus_age(&mut p);
     * ```
     */

    #[derive(Debug)]
    struct Person {
        age: i32,
        country: String,
    };

    impl Person {
        fn not_frequently_used(self) {}

        fn speak(self: &Self, str: &str) {
            println!("{}, {}", str, self.age);
        }

        fn age(&self) -> i32 {
            // 方法名可以与字段名称重复，age() 和 age rust编译器会自动判断调用的是方法还是属性
            self.age
        }

        fn plus_age(&mut self) {
            self.age = self.age + 1;
        }

        fn from(country: String) -> Self {
            // or return Person
            Person { age: 0, country }
        }
    }

    let mut p = Person::from(String::from("China"));
    p.age = 19;

    p.plus_age();

    println!("{}", p.age);
    println!("{}", p.age());

    p.speak(&format!("I from {} and I {} old!", p.country, p.age));
    println!("I from {} and I {} old!", p.country, p.age);

    // 当作关联函数调用
    println!("{}", p.age);
    Person::plus_age(&mut p);
    println!("{}", p.age);

    println!("{:#?}", p);
}
