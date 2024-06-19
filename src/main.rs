use core::fmt;
use std::{io, num};

fn main() {
    /*
     *
     * ## 转换和边界异常处理
     *
     * Result 和 Option 在业务程序中很常见，并且通常还需要自定义错误类型以便快速定位问题。
     *
     * ### 组合器
     *
     * 因为 Result 和 Option 类型很常见，但使用真实值时需要取出再判断，或者 Result 和 Option 互相转换，这些操作都显得比较琐碎，所以 rust 提供了一些组合器简化这些操作。
     * 这些操作的功能与 JavaScript 的与或非 `|, ||, &, &&, !` 的功能类似，可以返回某个值。
     *
     *
     * 组合器不同于组合模式，组合器更多的是用于对返回结果的类型进行变换：例如使用 ok_or 将一个 Option 类型转换成 Result 类型。
     * > 组合模式：将对象组合成树形结构以表示“部分整体”的层次结构。组合模式使得用户对单个对象和组合对象的使用具有一致性。–GoF \<\<设计模式\>\>
     *
     * ```rust
     * let id: Option<i32> = Some(1);
     * let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
     * println!("{}", id.unwrap());
     * ```
     *
     * #### or 和 and
     * 这两个方法会对两个 Option 表达式或两个 Result 表达式做逻辑组合，最终返回 Option 或 Result。
     *
     * - or()，表达式按照顺序求值，若任何一个表达式的结果是 Some 或 Ok，则该值会立刻返回
     * - and()，若两个表达式的结果都是 Some 或 Ok，则第二个表达式中的值被返回。若任何一个的结果是 None 或 Err ，则立刻返回。
     *
     * ```rust
     * // or 和 and
     * let id1: Option<i32> = Some(1);
     * let id2: Option<i32> = None;
     * println!(
     *     "id1 or id2 = {:?}, id1 and id2 = {:?}, id2 or id1 = {:?}, id2 and id1 = {:?}",
     *     id1.or(id2),
     *     id1.and(id2),
     *     id2.or(id1),
     *     id2.and(id1)
     * )
     * ```
     * 除了 or 和 and 之外，Rust 还提供了异或 xor ，但是它只能应用在 Option 上，不能应用在 Result 上，因为不能对一个值和错误进行异或操。
     *
     * #### or_else() 和 and_then()
     * 它们跟 or() 和 and() 类似，唯一的区别在于，它们的第二个表达式是一个返回 Option 或 Result 的闭包。
     * ```rust
     * // or_else 或 and_then
     * let id1: Option<i32> = Some(1);
     * let id2 = || None;
     * let id3 = |_| Some(1);
     *
     * // 注意 impl 不能作为直接作为普通变量的类型
     * // let id3: impl Fn() -> Option<i32> = || None; error `impl Trait` is only allowed in arguments and return types
     * println!(
     *     "id1.or_else(id2) = {:?}, id1.and_then(id3) = {:?}",
     *     id1.or_else(id2),
     *     id1.and_then(id3)
     * );
     * ```
     *
     * #### filter
     * filter 用于对 Option 进行条件过滤：
     * ```rust
     * // filter 用于对 Option 进行条件过滤
     * let id1 = Some(1);
     * let is_even = |x: &i32| x % 2 == 0;
     * println!("id1.filter(is_event) = {:?}", id1.filter(is_even));
     * ```
     *
     * #### map, map_err, map_or, map_or_else
     * map 可以将 Some 中的值映射为另一个 Some，Ok 类似：
     *
     * ```rust
     * let id1 = Some(1);
     * let mapFn = |x: i32| -> i32 {
     *     if x > 2 {
     *         2
     *     } else {
     *         1
     *     }
     * };
     * println!("id1.map(mapFn) = {:?}", id1.map(mapFn))
     * ```
     *
     * 如果需要对 Result 的 Err 的信息进行修改，就需要使用 map_err。
     *
     * map_or 和 map_or_else 在 map 的基础上添加了一个默认值，区别是 map_or 给定指定类型的默认值，map_or_else 通过闭包提供默认值。
     *
     * #### ok_or ok_or_else
     * ok_or 和 ok_or_else 都是将 Option 转换为 Result 的组合器，两者都接受一个 Err 默认参数，ok_or 直接给定类型参数，ok_or_else 通过闭包给定默认参数。
     *
     * ```rust
     * let id: Option<i32> = Some(1);
     * let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
     * println!("{}", id.unwrap());
     * ```
     *
     * ### 自定义错误类型
     * 虽然标准库定义了大量的错误类型，但光使用这些错误类型往往是不够的。在业务场景中，往往会自定义相应的错误(异常)类型。
     * 比如返回示例中常见的 JSON：`{ "code": 50020, "msg": "fail" }`。
     *
     * 为了更好的定义错误类型，标准库中定义了一些可复用的特征，比如 `std::error::Error` 特征：
     * ```rust
     * use std::fmt::{Debug, Display};
     * pub trait Error: Debug + Display {
     *     fn source(&self) -> Option<&(Error + 'static)> { ... }
     * }
     * ```
     *
     * 当自定义类型实现该特征后，该类型就可以作为 Err 来使用。
     * 实际上，rust 自定义错误类型非常简单，可以不实现 `std::error::Error`，只需要实现 Debug 和 Display 特征即可，这也就是说 source 方法是可选的。
     * 同时 Debug 特征往往无需手动实现，可以通过 derive 来派生，即可以选择自定义 Debug，也可以选择 derive Debug 特征。
     *
     * ```rust
     * // 自定义错误
     * struct AppError {
     *     code: i32,
     *     msg: String,
     * }
     * // 为自定义错误实现 Display 特征
     * impl fmt::Display for AppError {
     *     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
     *         write!(
     *             f,
     *             "Display: AppError {{ code: {}, message: {} }}, try again!",
     *             self.code, self.msg
     *         )
     *     }
     * }
     * // 为自定义错误实现 Debug 特征，也可以通过派生 Debug 实现
     * impl fmt::Debug for AppError {
     *     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
     *         write!(
     *             f,
     *             "Debug: AppError {{ code: {}, message: {} }}",
     *             self.code, self.msg
     *         )
     *     }
     * }
     *
     * fn produce_error() -> Result<(), AppError> {
     *     Err(AppError {
     *         code: 404,
     *         msg: String::from("Page not found"),
     *     })
     * }
     * match produce_error() {
     *     Err(err) => eprintln!("{}", err),
     *     _ => println!("No error"),
     * }
     * ```
     *
     * 事实上，实现 Debug 和 Display 特征并不是作为 Err 使用的必要条件，把这两个特征实现和相应使用去除也不会报错。为自定义类型实现这两个特征的原因有二:
     * - 错误打印输出后，才能有实际用处，而打印输出就需要实现这两个特征
     * - 可以将自定义错误转换成 Box<dyn std::error:Error> 特征对象，用来做归一化不同错误类型
     *
     *
     * ### From 特征，错误转换
     * 在一个函数运行中可能会产生不同的错误，如何将这些错误统一成自定义类型的错误呢？rust 提供 `std::convert::From` 特征解决转换问题。
     * 在生成 String 时经常使用到的 String::from 就是 `std::convert::From` 提供的功能。
     *
     * > `std::convert::From` 已在 std::prelude 中，无需手动导入。
     *
     * 在错误的转换中，还有一点特别重要，`?` 可以自动将错误进行隐式的强制转换.
     *
     * 为自定义错误实现 From 特征，将多种错误类型转换成自定义错误类型：
     *
     * ```rust
     * // 为自定义错误实现 From 特征，将 io::Error 转换成自定义错误
     * impl From<io::Error> for AppError {
     *     fn from(error: io::Error) -> Self {
     *         AppError {
     *             kind: String::from("io"),
     *             msg: String::from(error.to_string()),
     *         }
     *     }
     * }
     * // 为自定义错误实现 From 特征，将 num::ParseIntError 转换成自定义错误
     * impl From<num::ParseIntError> for AppError {
     *     fn from(error: num::ParseIntError) -> Self {
     *         AppError {
     *             kind: String::from("parse"),
     *             msg: String::from(error.to_string()),
     *         }
     *     }
     * }
     *
     * fn gen_error() -> Result<(), AppError> {
     *     // 函数标注返回AppError，有一点特别重要，? 可以将错误进行隐式的强制转换，将ParseIntError转换成 AppError
     *     let num: i32 = String::from("").parse()?;
     *     Ok(())
     * }
     * match gen_error() {
     *     Err(e) => eprintln!("{}", e),
     *     _ => println!("No error"),
     * }
     * ```
     *
     *
     */

    //  Option 转换成 Result
    let id: Option<i32> = Some(1);
    let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
    println!("{}", id.unwrap());

    // or 和 and
    let id1: Option<i32> = Some(1);
    let id2: Option<i32> = None;
    println!(
        "id1.or(id2) = {:?}, id1.and(id2) = {:?}, id2.or(id1) = {:?}, id2.and(id1) = {:?}",
        id1.or(id2),
        id1.and(id2),
        id2.or(id1),
        id2.and(id1)
    );

    // or_else 或 and_then
    let id1: Option<i32> = Some(1);
    let id2 = || None;
    let id3 = |_| Some(1);

    // 注意 impl 不能作为直接作为普通变量的类型
    // let id3: impl Fn() -> Option<i32> = || None; error `impl Trait` is only allowed in arguments and return types
    println!(
        "id1.or_else(id2) = {:?}, id1.and_then(id3) = {:?}",
        id1.or_else(id2),
        id1.and_then(id3)
    );

    // filter 用于对 Option 进行条件过滤
    let id1 = Some(1);
    let is_even = |x: &i32| x % 2 == 0;
    println!("id1.filter(is_event) = {:?}", id1.filter(is_even));

    // map 可以将 Some 中的值映射为另一个 Some，Ok 类似
    let id1 = Some(1);
    let mapFn = |x: i32| -> i32 {
        if x > 2 {
            2
        } else {
            1
        }
    };
    println!("id1.map(mapFn) = {:?}", id1.map(mapFn));

    // 自定义错误
    struct AppError {
        kind: String,
        msg: String,
    }
    impl fmt::Display for AppError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Display: AppError {{ kind: {}, message: {} }}, try again!",
                self.kind, self.msg
            )
        }
    }
    impl fmt::Debug for AppError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Debug: AppError {{ kind: {}, message: {} }}",
                self.kind, self.msg
            )
        }
    }

    fn produce_error() -> Result<(), AppError> {
        Err(AppError {
            kind: String::from(""),
            msg: String::from("Page not found"),
        })
    }
    match produce_error() {
        Err(err) => eprintln!("{}", err),
        _ => println!("No error"),
    }

    // 为自定义错误实现 From 特征，将 io::Error 转换成自定义错误
    impl From<io::Error> for AppError {
        fn from(error: io::Error) -> Self {
            AppError {
                kind: String::from("io"),
                msg: String::from(error.to_string()),
            }
        }
    }
    // 为自定义错误实现 From 特征，将 num::ParseIntError 转换成自定义错误
    impl From<num::ParseIntError> for AppError {
        fn from(error: num::ParseIntError) -> Self {
            AppError {
                kind: String::from("parse"),
                msg: String::from(error.to_string()),
            }
        }
    }

    fn gen_error() -> Result<(), AppError> {
        // 函数标注返回AppError，有一点特别重要，? 可以将错误进行隐式的强制转换，将ParseIntError转换成 AppError
        let num: i32 = String::from("").parse()?;
        Ok(())
    }
    match gen_error() {
        Err(e) => eprintln!("{}", e),
        _ => println!("No error"),
    }
}
