use rand::Rng;

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
     * ### 2. 避免同名引用
     * 不同的包、模块之间可能会存在重复的名称，怎么避免重名呢？有两种方法：
     * 1. 模块::函数
     * 2. as 别名引用
     *
     * `模块::函数`形式就是通过父模块来区分不同的子模块，比较适合路径较短的情况
     * ```rs
     * use std::fmt;
     * use std::io;
     *
     * fn function1() -> fmt::Result {}
     * fn function2() -> io::Result<()> {}
     * ```
     *
     * `as` 别名引用的强大之处在于为一个模块赋予新的名称：
     * ```rs
     * use std::fmt::Result;
     * use std::io::Result as IoResult;  // 使用 as 给予它一个全新的名称 IoResult
     *
     * fn function1() -> Result {}
     * fn function2() -> IoResult<()> {}
     * ```
     *
     * ### 3. 引入项再导出 re-exporting
     * 当外部的模块项 A 被引入到当前模块中时，它的可见性自动被设置为私有的，可以对它进行再导出以允许其它外部代码引用模块项 A，在 `src/lib.rs` 中，对 `front_of_house::hosting` 再导出：
     * ```rs
     * pub use crate::front_of_house::hosting; // 在 use 关键字前使用 pub 再导出
     *
     * pub fn eat_at_restaurant() {
     *     hosting::add_to_waitlist();
     *     hosting::add_to_waitlist();
     *     hosting::add_to_waitlist();
     * }
     * ```
     * 当希望将内部的实现细节隐藏起来或者按照某个目的组织代码时，可以使用 pub use 再导出。
     * 例如统一使用一个模块来提供对外的 API，那该模块就可以引入其它模块中的 API然后进行再导出，最终对于用户来说，所有的 API 都是由一个模块统一提供的。
     *
     * > 如果你有前端项目经验，会发现很多项目都会把内部的实现逻辑隐藏起来，然后通过 `index(.ts|.js|.tsx|.jsx)` 统一导出内部项。
     *
     * ### 4. 使用第三方包
     * 除了内置模块和自定义模块外，还可以使用其他开发者提供的包/模块，以下是引入第三方包的流程：
     * 1. 修改 Cargo.toml 文件，在 [dependencies] 区域添加一行：rand = "0.8.3"
     * 2. 等待相应的插件如 `rust-analyzer` 拉去完成后，就可以在需要的位置使用 `use` 导入该包
     *
     * ```rs
     * use rand::Rng;
     * fn main() {
     *     let secret_number = rand::thread_rng().gen_range(1..101);
     * }
     * ```
     * 注意：需要使用 use 引入第三方包 rand 中的 Rng 特征，因为调用的 gen_range 方法定义在该特征中，需要在当前作用域确定方法的来源。
     *
     * Rust 社区已经为我们贡献了大量高质量的第三方包，你可以在 crates.io 或者 lib.rs 中检索和使用，从目前来说查找包更推荐 lib.rs，搜索功能更强大，内容展示也更加合理，但是下载依赖包还是得用crates.io。
     *
     * ### 5. 简化导入方式
     *
     *  #### 1. 使用 {} 简化引入方式
     * 在相同的包/模块中导入不同的子模块（子项）时，可以使用 `{}` 合并导入
     * ```rs
     * use std::collections::{HashMap,BTreeMap,HashSet};
     * use std::{cmp::Ordering, io};
     * ```
     *
     * #### 2. self
     * 如果希望同时导入模块本身和子模块
     * ```rs
     * use std::io;
     * use std::io::Write;
     *
     * // 使用{}合并导入模块自身和子项
     * use std::io::{self, Write};
     * ```
     * 上面使用到了模块章节提到的 self 关键字，用来替代模块自身，结合上一节中的 self，可以得出它在模块中的两个用途：
     * - use self::xxx，表示加载当前模块中的 xxx。此时 self 可省略
     * - use xxx::{self, yyy}，表示，加载当前路径下模块 xxx 本身，以及模块 xxx 下的 yyy
     *
     * #### 3. * 引入模块下的所有项
     * 对于之前一行一行引入 std::collections 的方式，还可以使用 `*` 导入模块下的所有子项：
     * ```rs
     * use std::collections::*;
     * ```
     * 当使用 * 来引入的时候要格外小心，因为很难知道到底哪些被引入到了当前作用域中，有哪些会和自己程序中的名称冲突，在实际项目中，这种引用方式往往用于快速写测试代码。
     *
     *
     */

    let secret_number = rand::thread_rng().gen_range(1..10);
}
