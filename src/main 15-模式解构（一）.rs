use core::panic;

fn main() {
    /*
     * ## 模式解构赋值
     *
     * 模式匹配时可用于解构赋值，可解构的类型包括struct、enum、tuple、slice等等。
     *
     * 解构赋值时，可使用_作为某个变量的占位符，使用..作为剩余所有变量的占位符(使用..时不能产生歧义，
     * 例如 (..,x,..) 是有歧义的，解构需要明确的模式匹配)。当解构的类型包含了命名字段时，可使用fieldname简化fieldname: fieldname的书写。
     *
     * ### struct
     *
     * 解构Struct时，会将待解构的struct各个字段和Pattern中的各个字段进行匹配，并为找到的字段变量进行赋值。
     * 当Pattern中的字段名和字段变量同名时，可简写。例如P{name: name, age: age}和P{name, age}是等价的Pattern，赋值后原有变量名失效。
     * struct/对象的解构形式上，Rust与JavaScript有一点区别，Rust需要在解构的位置声明类型，`let Point{x, y} = p;`，而JavaScript则是直接解构 `const {x, y}  = p`。
     *
     *
     * ```rs
     * struct Point2 {
     *   x: i32,
     *   y: i32,
     * }
     *
     * struct Point3 {
     *   x: i32,
     *   y: i32,
     *   z: i32,
     * }
     *
     * fn main(){
     *   let p = Point2{x: 0, y: 7};
     *
     *   // 等价于 let Point2{x: x, y: y} = p;
     *   let Point2{x, y} = p;
     *   println!("x: {}, y: {}", x, y);
     *   // 解构时可修改字段变量名:
     *   let Point2 { x: a, y: b } = p;
     *   // 此时，变量a和b将被赋值，原有变量名失效
     *   println!("a: {a}, b: {b}");
     *
     *   let ori = Point{x: 0, y: 0, z: 0};
     *   match ori{
     *     // 使用..忽略解构后剩余的字段
     *     Point3 {x, ..} => println!("{}", x),
     *   }
     * }
     * ```
     *
     * ### 解构enum
     *
     * Rust的模式匹配非常强大，在模式匹配章节中提到过，Rust支持 `字面量`, `变量`, `or`, `范围` 四种类型单个或组合的pattern。
     *
     * ```rs
     * enum IPAddr {
     *   IPAddr4(u8,u8,u8,u8),
     *   IPAddr6(String),
     * }
     *
     * fn main(){
     *   let ipv4 = IPAddr::IPAddr4(127,0,0,1);
     *   match ipv4 {
     *     // 丢弃解构后的第四个值
     *     IPAddr::IPAddr4(a,b,c,_) => println!("{},{},{}", a,b,c),
     *     IPAddr::IPAddr6(s) => println!("{}", s),
     *   }
     * }
     * ```
     *
     * ### 解构元组
     *
     * tuple 的元素可以是任意类型，解构位置（pattern）需要指定类型。
     * ```rs
     * let ((feet, inches), Point {x, y}) = ((3, 1), Point { x: 3, y: -1 });
     * ```
     *
     * ### @ 绑定变量名
     *
     * 当解构后进行模式匹配时，如果某个值没有对应的变量名，可以使用`@`手动绑定一个变量名，一般用在数组，元组上。（结构体有自己的属性名称）
     * ```rs
     * #![allow(unused)]
     * fn main() {
     *     struct S(i32, i32);
     *     match S(1, 2) {
     *     // 如果匹配1成功，将其赋值给变量z
     *     // 如果匹配2成功，也将其赋值给变量z
     *         S(z @ 1, _) | S(_, z @ 2) => assert_eq!(z, 1),
     *         _ => panic!(),
     *     }
     *
     *     let p = [1, 2, 4];
     *     match p {
     *         // pattern
     *         [1, ret @ .., z @ 4] => println!("{:?}, {z}", ret), // [2], 4
     *         _ => println!("no result"),
     *     }
     * }
     * ```
     */

    #[derive(Debug)]
    struct Point2 {
        x: i32,
        y: i32,
    }

    struct Point3 {
        x: i32,
        y: i32,
        z: i32,
    }

    let p = Point2 { x: 0, y: 7 };

    // 等价于 let Point2{x: x, y: y} = p;
    let Point2 { x, y } = p;
    println!("x: {}, y: {}", x, y);
    // 解构时可修改字段变量名:
    let Point2 { x: a, y: b } = p;
    // 此时，变量a和b将被赋值，原有变量名失效
    println!("a: {a}, b: {b}");

    let ori = Point3 { x: 0, y: 0, z: 0 };
    match ori {
        // 使用..忽略解构后剩余的字段
        Point3 { x, .. } => println!("{}", x),
    }

    // 解构enum
    enum IP {
        V4(u8, u8, u8, u8),
        V6(String),
    }

    let ip = IP::V6(String::from("world"));
    match ip {
        // 变量类型的模式匹配
        IP::V6(s) => println!("{s}"),
        _ => panic!("发生错误"),
    }

    // 模式解构数组
    let p = [1, 2, 4];
    match p {
        // pattern
        [1, ret @ .., z @ 4] => println!("{:?}, {z}", ret),
        _ => println!("no result"),
    }
}
