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
     *
     * ### 代码改进
     * - 单一且庞大的函数。main 函数当前执行两个任务：解析命令行参数和读取文件，需要将大的函数拆分成更小的功能单元。
     * - 配置变量散乱在各处。独立的变量越多，越是难以维护，需要将这些用于配置的变量整合到一个结构体中。
     * - 细化错误提示。文件不存在、无权限等等都是可能的错误，一条大一统的消息无法给予用户更多的提示。
     * - 使用错误而不是异常。需要增加合适的错误处理代码，来给予使用者给详细友善的提示。
     *
     * #### 分离 main 函数
     * 关于如何处理庞大的 main 函数，Rust 社区给出了统一的指导方案:
     * - 将程序分割为 main.rs 和 lib.rs，并将程序的逻辑代码移动到后者内
     * - 对部分非常基础的功能，严格来说不算是逻辑代码的一部分，可以放在 main.rs 中
     *
     * 重新梳理后可以得出 main 函数应该包含的功能:
     * - 解析命令行参数
     * - 初始化其它配置
     * - 调用 lib.rs 中的 run 函数，以启动逻辑代码的运行
     * - 如果 run 返回一个错误，需要对该错误进行处理
     *
     * 这个方案有一个很优雅的名字: 关注点分离 (Separation of Concerns)。简而言之，main.rs 负责启动程序，lib.rs 负责逻辑代码的运行。
     * 从测试的角度而言，这种分离也非常合理： lib.rs 中的主体逻辑代码可以得到简单且充分的测试，至于 main.rs ？确实没办法针对其编写额外的测试代码，但由于它的代码也很少，容易保证它的正确性。
     *
     */

    // 通过类型注释，Rust编译器会将collect方法读取成指定类型
    let args: Vec<String> = env::args().collect();

    // 实现Debug特征，使用 `{:?}` 输出
    // cargo run -- Hello, Minigrep
    println!("{:?}", args); // => ["projectpath", "Hello", "Minigrep"]

    let contents =
        fs::read_to_string(args[1].clone()).expect("Should have been able to read the file.");

    println!("The contents: \n{contents}");
}
