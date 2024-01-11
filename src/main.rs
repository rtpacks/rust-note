use std::{env, fs};

fn main() {
    /*
     *
     * ## Minigrep
     * https://course.rs/basic-practice/base-features.html
     *
     * 在cargo的命令行运行中，`--` 是给调用的程序使用的，而 `--` 前的参数是给cargo使用的。Rust的命令行参数可以通过 `env::args()` 方法来获取：
     *
     * 首先通过 use 引入标准库中的 env 包，然后 env::args 方法会读取并分析传入的命令行参数，最终通过 collect 方法输出一个集合类型 Vector。
     * > env::args 读取到的参数集合中第一个是程序的可执行的路径名
     * ```rust
     * use std::env;
     * fn main() {
     *     let args: Vec<String> = env::args().collect();
     *     dbg!(args);
     * }
     * ```
     * **用户的输入不可信**，例如用户输入非 Unicode 字符时，当前程序会直接崩溃。
     * 原因是当传入的命令行参数包含非 Unicode 字符时， `std::env::args 会直接崩溃。
     * 如果有这种特殊需求，建议大家使用 std::env::args_os，该方法产生的数组将包含 OsString 类型，而不是之前的 String 类型，前者对于非 Unicode 字符会有更好的处理。
     *
     * 两个选择：
     * 1. 用户爱输入啥输入啥，崩溃了知道自己错了
     * 2. args_os 会引入额外的跨平台复杂性
     *
     * ### 读取文件
     * 在 `minigrep` 程序中，不建议使用 `args[1]` `args[2]` 形式来使用，需要用到变量来存储读取的文件路径和带搜索的字符串。
     * 
     * ```rust
     * use std::fs;
     * fn main {
     *      let args: Vec<String> = env::args().collect();
     *      let file_path = args[1].clone();
     *      let contents = fs::read_to_string(file_path).expect("Should have been able to read the file.");
     *      println!("The contents: \n${contents}");
     * }
     * ```
     * 
     * ```shell
     * cargo run -- D:\workspace\Rust\rust-note\README.md
     * cargo run -- D:\workspace\Rust\rust-note\public\poem.txt
     * ```
     */

    println!("Hello, world!");

    // 通过类型注释，Rust编译器会将collect方法读取成指定类型
    let args: Vec<String> = env::args().collect();

    // 实现Debug特征，使用 `{:?}` 输出
    // cargo run -- Hello, Minigrep
    println!("{:?}", args); // => ["projectpath", "Hello", "Minigrep"]

    let contents = fs::read_to_string(args[1].clone()).expect("Should have been able to read the file.");

    println!("The contents: \n{contents}");
}
