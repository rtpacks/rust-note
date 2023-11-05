use core::panic;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, Read};

fn main() {
    /*
     * ## 单元 Module
     * 一个 `src/lib.rs` 或 `src/main.rs` 就是一个包Crate，一个包可能是多种功能的集合体。为了项目工程（Package）的组织维护，需要对包进行拆分。
     *
     * 模块Module（mod）是**rust代码的构成单元**，是代码拆分的单位（文件/文件夹）。
     * 使用模块可以将包中的代码按照功能性进行重组，而不需要将所有代码都写在 `src/lib.rs` 或者 `src/main.rs`，最终实现更好的可读性及易用性。
     * 同时，使用mod还可以非常灵活地去控制代码的可见性，进一步强化 Rust 的安全性。
     *
     * 以lib的包（lib crate）为例，该包（crate）的入口在 `src/lib.rs`，也是包的根。在 `src/lib.rs`` 里定义模块（mod）非常简单：
     * ```rs
     * mod mymod {
     *     fn test() {
     *         println!("test");
     *     }
     * }
     * ```
     * 但实际项目中不可能将所有的模块（mod）都放在lib.rs文件（包的根），而是会将代码按功能等拆分为多个模块（mod）。
     *
     * ### 1. 模块拆分
     *
     * **一般来说，一个文件都会被视为一个mod，而且mod可以嵌套定义。嵌套定义的mod既可以写在同一个文件里，也可以通过文件夹的形式来实现。**
     *
     * 以餐馆为例，使用 `cargo new --lib restaurant` 创建一个餐馆，注意这里创建的是一个库类型的 Package，然后将以下代码放入 src/lib.rs 中：
     * ```rs
     * // 餐厅前厅，用于吃饭
     * mod front_of_house {
     *     // 招待客人
     *     mod hosting {
     *         fn add_to_waitlist() {}
     *         fn seat_at_table() {}
     *     }
     *     // 服务客人
     *     mod serving {
     *         fn take_order() {}
     *         fn serve_order() {}
     *     }
     * }
     * ```
     * 以上的代码创建了三个模块，有几点需要注意的：
     *
     * - 使用 mod 关键字来创建新模块，后面紧跟着模块名称
     * - 模块可以嵌套，这里嵌套的原因是招待客人和服务都发生在前厅
     * - 模块中可以定义各种 Rust 类型，例如函数、结构体、枚举、特征等
     * - 所有模块均定义在同一个文件中
     * 类似上述代码中所做的，使用模块就能将功能相关的代码组织到一起，然后通过一个模块名称来说明这些代码为何被组织在一起。
     *
     *
     *
     * 以lib类型的crate为例，该crate的入口在src/lib.rs，也是crate的根。
     */
}
