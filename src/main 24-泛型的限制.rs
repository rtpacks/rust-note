use std::fmt::Debug;

fn main() {
    /*
     * ## 泛型的限制
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
     * 观察指定变量数据类型的写法i: i32和限制泛型的写法T: Trait_Name，由此可知，Trait其实是泛型的数据类型，Trait限制了泛型所能代表的类型，正如数据类型限制了变量所能存放的数据格式。
     * 有时候需要对泛型做多重限制，这时使用+即可。例如T: Add<Output=T>+Copy+Clone，表示限制泛型T只能代表那些同时实现了Add、Copy、Clone这三种Trait的数据类型。
     *
     * 之所以要做多重限制，是因为有时候限制少了，泛型所能代表的类型不够精确或者缺失某种功能。比如，只限制泛型T是实现了std::ops::Add Trait的类型还不够，还要限制它实现了Copy Trait以便函数体内的参数i被转移所有权时会自动进行Copy，但Copy Trait是Clone Trait的子Trait，即Copy依赖于Clone，因此限制泛型T实现Copy的同时，还要限制泛型T同时实现Clone Trait。
     *
     * 简而言之，要对泛型做限制，一方面是函数体内需要某种Trait提供的功能(比如函数体内要对i执行加法操作，需要的是std::ops::Add的功能)，另一方面是要让泛型T所能代表的数据类型足够精确化(如果不做任何限制，泛型将能代表任意数据类型)。
     *
     *
     * ### 总结
     * 观察变量数据类型的写法 `i: i32` 和限制泛型的写法 `T: Trait` 可以得到一个结果，Trait就是泛型的数据类型，泛型就是代表“已知的数据类型”的一个变量（和类型），正如数据类型限制了变量所能存放的数据格式（默认携带了类型）。
     *
     * 因此在rust编译期间，会根据泛型所有的情况生成不同的代码，用于零成本抽象。
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
