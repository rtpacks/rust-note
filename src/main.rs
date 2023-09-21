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
}
