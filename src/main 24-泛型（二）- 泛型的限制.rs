use std::fmt::Debug;

fn main() {
    /*
     * ## 泛型
     * ### 泛型的限制
     *
     * ```rs
     * fn double<T>(i: T) -> T {}
     * ```
     * 在double函数的定义中，double期待的是对数值进行加法操作，但**泛型**却可以代表各种类型。
     * 因此，还需要对泛型T进行限制，否则在调用double函数时就允许传递字符串类型、Vec类型等值作为函数参数，这会产生错误。
     *
     * 事实上，在double的函数体内对泛型T的值i进行加法操作，只有实现了 Trait `std::ops::Add` 的类型才能使用+进行加法操作。
     * 因此要限制泛型T是那些实现了std::ops::Add的数据类型。
     *
     * **限制泛型**也叫做Trait绑定(**Trait Bound**)，其语法有两种：
     * - 在定义泛型类型T时，使用类似于T: Trait_Name这种语法进行限制
     * - 在返回值后面、大括号前面使用where关键字，如where T: Trait_Name
     * `T: trait_name` 这种形式中，`:` 不代表特征 Trait 的继承，而是表示对数据类型的限制，即T: Trait_Name表示将泛型T限制为那些实现了Trait_Name Trait的数据类型。
     *
     * 以下两种写法是等价的，但where关键字在复杂的定义中，可读性更好。
     * ```rs
     * fn f<T: Clone + Copy>(i: T) -> T{}
     *
     * fn f<T>(i: T) -> T
     *  where T: Clone + Copy {}
     * ```
     *
     * 复杂场景
     * ```rs
     * // 更复杂的示例：
     * fn query<M: Mapper + Serialize, R: Reducer + Serialize>(
     *     data: &DataSet, map: M, reduce: R) -> Results
     * {
     *     ...
     * }
     *
     * // 此时，下面写法更友好、可读性更高
     * fn query<M, R>(data: &DataSet, map: M, reduce: R) -> Results
     *     where M: Mapper + Serialize,
     *           R: Reducer + Serialize
     * {
     *     ...
     * }
     * ```
     *
     * 因此，在 `double` 函数的声明是：
     * T: std::ops::Add表示泛型T只能代表那些实现了std::ops::Add Trait的数据类型，比如各种数值类型都实现了Add Trait，因此T可以代表数值类型，而Vec类型没有实现Add Trait，因此T不能代表Vec类型。
     *
     */

    trait Moveable {
        fn run() {
            println!("running")
        }
    }

    fn playSimple<T: Moveable + Debug, K: Moveable + Debug>(p: T) {}

    fn playComplex<T, K>()
    where
        T: Moveable + Debug,
        K: Moveable + Debug,
    {
    }
}
