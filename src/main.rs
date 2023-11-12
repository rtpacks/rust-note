use core::panic;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, Read};

fn main() {
    /*
     * ## 使用 use 及受限可见性
     *
     * 在同一个包内 rust 可以通过绝对路径 `crate::front_of_house::hosting` 或相对路径 `front_of_house::hosting::add_to_waitlist` 直接使用模块。
     *
     * ### 1. 基本引入方式
     * 如果需要在其他包/模块中使用，还可以使用 `use` 关键字导入模块或模块内的函数、结构体等内容，导入的路径也是绝对路径和相对路径两种：
     * - 绝对路径 `use crate::front_of_house::hosting;`
     * - 相对路径 `use front_of_house::hosting;`
     *
     * 导入之后就可以使用模块或模块内容：
     *
     * 目录结构
     *```sh
     * src
     *  │─ lib.rs
     *  │─ front_of_house.rs // 文件夹不完全管理模块的形式
     *  └─ front_of_house
     *      │─ hosting.rs
     *      └─ serving.rs
     * ```
     *
     * 在 `src/lib.rs` 中导入 `front_of_house` 的子模块 `hosting` 和 `serving`，并调用模块的方法。
     * ```rs
     * use crate::front_of_house::hosting; // 绝对路径导入模块的方式
     * use front_of_house::serving; // 相对路径导入模型的方式
     *
     * mod front_of_house; // 加载子模块（类似占位符）
     *
     * pub fn cleanTable() {
     *     // 绝对路径使用模块（方法）
     *     crate::front_of_house::hosting::add_to_waitlist();
     *     // 相对路径使用模块（方法）
     *     front_of_house::hosting::add_to_waitlist();
     *     // use绝对路径导入并使用模块（方法）
     *     hosting::add_to_waitlist();
     *     // use相对路径导入并使用模块（方法）
     *     serving::take_payment();
     * }
     * ```
     *
     * 以上的代码注意区分加载子模块 `mod front_of_house;` 和导入模块 `use front_of_house::serving;` 的区别，导入子模块只是为了方便管理将子模块抽离成一个文件，导入模块是和当前模块没有关系的，只是需要用到其他模块的内容才进行引用。
     *
     * #### 引入模块还是函数
     * 从使用简洁性来说，引入函数自然是更甚一筹，但是在某些时候，引入模块会更好：
     * - 需要引入同一个模块的多个函数
     * - 作用域中存在同名函数
     * 严格来说，对于引用方式并没有需要遵守的惯例，建议优先使用最细粒度(引入函数、结构体等)的引用方式，如果引起了某种麻烦(例如前面两种情况)，再使用引入模块的方式。
     *
     */
}
