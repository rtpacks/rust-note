use rand::Rng;

fn main() {
    /*
     * ## 注释和文档
     *
     * 在 Rust 中，注释分为三类：
     * - 代码注释，用于说明某一块代码的功能，读者往往是同一个项目的协作开发者
     * - 文档注释，支持 Markdown，对项目描述、公共 API 等用户关心的功能进行介绍，同时还能提供示例代码，目标读者往往是想要了解你项目的人
     * - 包和模块注释，严格来说这也是文档注释中的一种，它主要用于说明当前包和模块的功能，方便用户迅速了解一个项目
     *
     * ### 1. 代码注释
     * 代码注释和其他语言类似，具体分为两种行注释和块注释：
     * - 行注释：可以放在某一行代码的上方，也可以放在当前代码行的后方 `//`
     * - 块注释：当注释行数较多时，可以使用块注释 ```/\*\*\/```
     *
     * ### 2. 文档注释
     * 当查看一个 crates.io 上的包提供的文档来浏览相关的功能特性、使用方式，这种文档就是通过文档注释实现的。
     * Rust 提供了 cargo doc 的命令，可以用于把这些文档注释转换成 HTML 网页文件，最终展示给用户浏览，这样用户就知道这个包是做什么的以及该如何使用。
     *
     * 与代码注释相同，文档注释也分为行注释和块注释：
     * - 文档行注释，用三个斜杠描述 ///
     * - 文档块注释，用 /\*\* \*\/ 描述
     *
     * 文档注释需要注意几点：
     * 1. 文档注释需要位于 `lib` 类型的包中，例如 `src/lib.rs` 中
     * 2. 文档注释可以使用 `markdown` 语法！例如 `# Examples` 的标题，以及代码块高亮
     * 3. 被注释的对象需要使用 `pub` 对外可见，记住：文档注释是给用户看的，内部实现细节不应该被暴露出去
     *
     * ### 3. 包和模块级别的注释
     * 除了函数、结构体等 Rust 项的注释，还可以给包和模块添加注释，需要注意的是，这些注释要添加到包、模块的最上方！
     * 包模块注释，可以让用户从整体的角度理解包的用途，让用户在看的时候心中有数。
     *
     * 与之前的任何注释一样，包级别的注释也分为两种：
     * - 行注释 \/\/!
     * 块注释 /\*! ... \*\/
     *
     * ### 4. 查看文档 cargo doc
     * 运行 `cargo doc` 可以直接生成 `HTML` 文件，放在 `target/doc` 目录下。为了方便，可以使用 `cargo doc --open` 命令，可以在生成文档后，自动在浏览器中打开网页。
     *
     * #### 常用文档标题
     * 除了 `# Examples`，一些常用的标题可以在项目中酌情使用，这些标题更多的是一种惯例：
     * - Panics：函数可能会出现的异常状况，这样调用函数的人就可以提前规避
     * - Errors：描述可能出现的错误及什么情况会导致错误，有助于调用者针对不同的错误采取不同的处理方式
     * - Safety：如果函数使用 unsafe 代码，那么调用者就需要注意一些使用条件，以确保 unsafe 代码块的正常工作
     *
     * ### 5. 文档测试(Doc Test)
     * 文档注释和包模块注释除了支持生成文档外，还支持文档测试，也就是文档注释和包模块注释不仅可以生成文档，还可以作为单元测试的用例运行，使用 cargo test 运行测试。
     *
     * 注意：文档注释和包模块注释都需要在 `src/lib.rs` 即库类型的 crate 中使用。
     *
     * 在注释中尽量使用 **完整路径** 来调用函数，因为测试是在另外一个独立的线程中运行的，在 lib.rs 中加入：
     *
     * ```rs
     * pub mod compute {
     *     /// `add_one` 将指定值加1
     *     ///
     *     /// # Examples11
     *     ///
     *     /// ```rust
     *     /// let arg = 3;
     *     /// let answer = ilearn::compute::add_one(arg);
     *     ///
     *     /// assert_eq!(6, answer);
     *     /// ```
     *     pub fn add_one(x: i32) -> i32 {
     *         let a = 3;
     *         x + a
     *     }
     * }
     * ```
     * 可以看到，文档中的测试用例被完美运行，而且输出中也明确提示了 Doc-tests world_hello，意味着这些测试的名字叫 Doc test 文档测试。
     * 在测试过程中，可能调用函数的函数发生panic，导致测试用例无法继续执行，如果想要通过这种测试，可以添加 should_panic，通过 should_panic，告诉 Rust 这个用例可能会导致 panic，这样测试用例就能顺利通过：
     * ```rust
     * /// ```rust,should_panic
     * /// let arg = 1;
     * /// let answer = ilearn::compute::add_two(arg);
     * /// ```
     *  pub fn add_two(x: i32) -> i32 {
     *      if x == 1 {
     *          panic!("x 不能等于 1");
     *      }
     *      let a = 3;
     *      x + a
     *  }
     * ```
     *
     * #### 保留测试，隐藏文档
     * 希望保留文档测试的功能，但是又要将某些测试用例的内容从文档中隐藏起来，使用 `#` 就能达到效果。
     * 使用 # 开头的行会在文档中被隐藏起来，但是依然会在文档测试中运行：
     * ```rust
     * pub mod compute {
     *     /// `add_one` 将指定值加1
     *     ///
     *     /// # Examples11
     *     ///
     *     /// ```rust
     *     /// let arg = 3;
     *     /// let answer = ilearn::compute::add_one(arg);
     *     /// # let answer2 = ilearn::compute::add_one(5);
     *     /// assert_eq!(6, answer);
     *     /// ```
     *     pub fn add_one(x: i32) -> i32 {
     *         let a = 3;
     *         x + a
     *     }
     * }
     * ```
     *
     * ### 6. 文档注释的代码跳转
     * 注释可以生成文档，可以编写测试用例，rust的注释还支持代码跳转，使用 `[\`\`]` 表示可跳转，跳转支持标准库、指定完整路径、指定类型多种方式：
     * ```rust
     * /// 直接指定跳转标准库：`add_one` 返回一个[`Option`]类型
     * /// 使用完整路径跳转：[`crate::MySpecialFormatter`]
     * /// 使用完整路径跳转：[`crate::MySpecialFormatter`]
     * /// 跳转到结构体  [`Foo`](struct@Foo)
     * /// 跳转到同名函数 [`Foo`](fn@Foo)
     * /// 跳转到同名宏 [`foo!`]
     * pub fn add_one(x: i32) -> Option<i32> {
     *     Some(x + 1)
     * }
     * pub struct MySpecialFormatter;
     * pub struct Bar;
     * pub struct Foo {}
     * pub fn Foo() {}
     *
     * #[macro_export]
     * macro_rules! foo {
     *   () => {}
     * }
     * ```
     *
     * ### 7. 总结
     * 注释分为三类：代码注释，文档注释，包和模块注释，这三类各自又能分为行注释和块注释。其中文档注释和包模块注释能够使用 `cargo doc` 生成文档便于使用者阅读。
     * 同时，文档注释和包模块注释支持文档测试，也就是文档注释和包模块注释不仅可以生成文档，还可以作为单元测试的用例运行，使用 cargo test 运行测试。
     *
     * ```rs
     * pub mod compute {
     *     /// `add_one` 将指定值加1
     *     ///
     *     /// # Examples11
     *     ///
     *     /// ```rust
     *     /// let arg = 3;
     *     /// let answer = ilearn::compute::add_one(arg);
     *     ///
     *     /// assert_eq!(6, answer);
     *     /// ```
     *     pub fn add_one(x: i32) -> i32 {
     *         let a = 3;
     *         x + a
     *     }
     * }
     * ```
     */

    /// `add_one` 将指定值加1
    ///
    /// # Examples11
    ///
    /// ```rust
    /// let arg = 3;
    /// let answer = crate::compute::add_one(arg);
    ///
    /// assert_eq!(6, answer);
    /// ```
    mod compute {
        fn add_one(x: i32) -> i32 {
            let a = 3;
            x + a
        }
    }

    /// 直接指定跳转标准库：`add_one` 返回一个[`Option`]类型
    /// 使用完整路径跳转：[`ilearn::MySpecialFormatter`]
    /// 使用完整路径跳转：[`crate::MySpecialFormatter`]
    /// 跳转到结构体  [`Foo`](struct@Foo)
    /// 跳转到同名函数 [`Foo`](fn@Foo)
    /// 跳转到同名宏 [`foo!`]
    pub fn add_one(x: i32) -> Option<i32> {
        Some(x + 1)
    }
    pub struct MySpecialFormatter;
    pub struct Bar;
    pub struct Foo {}
    pub fn Foo() {}

    #[macro_export]
    macro_rules! foo {
        () => {};
    }
}
