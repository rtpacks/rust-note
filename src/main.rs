use core::panic;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, Read};

fn main() {
    /*
     * ## 使用 use 及受限可见性
     * 在同一个包内 rust 可以通过绝对路径 `crate::front_of_house::hosting` 或相对路径 `front_of_house::hosting::add_to_waitlist` 直接使用模块。
     * 如果需要在其他包中使用，需要使用 `use` 关键字导入模块或模块内的函数、结构体等内容，导入的路径也是绝对路径和相对路径两种：
     * - 绝对路径 `use crate::front_of_house::hosting;`
     * - 相对路径 `use front_of_house::hosting;`
     *
     * 导入之后就可以使用模块或模块内容：
     *
     *```sh
     * src
     *  │─ lib.rs
     *  │─ front_of_house.rs // 文件夹不完全管理模块的形式
     *  └─ front_of_house
     *      │─ hosting.rs
     *      └─ serving.rs
     * ```
     *
     */
}
