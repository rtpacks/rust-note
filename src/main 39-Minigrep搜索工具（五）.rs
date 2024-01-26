use ilearn::{run, Config};
use std::{env, error::Error, fs, process};

fn main() {
    /*
     *
     * ## Minigrep
     * 实现忽略大小写功能
     *
     * 再次熟悉测试驱动开发模式：测试驱动开发模式(TDD, Test Driven Development)：
     * - 编写一个注定失败的测试，并且失败的原因和你指定的一样
     * - 编写一个成功的测试，与第三步配合
     * - 编写并优化你的功能逻辑代码（搜索函数功能），直到通过测试
     * - 这三个步骤将在我们的开发过程中不断循环，直到所有的代码都开发完成并成功通过所有测试。
     *
     * ### 1. 编写失败用例
     * 在 `search_fail` 函数中返回空vec，用于测试失败：
     * ```rust
     * // lib.rs
     *
     * #[cfg(test)]
     * mod tests {
     *     use super::*;
     *
     *    fn case_fail_result() {
     *         let query = "rust";
     *         let contents = "\
     * Rust:
     * safe, fast, productive.
     * Pick three.";
     *
     *         assert_eq!(
     *             vec!["Rust:"],
     *             search_case_insensitive_fail(query, contents)
     *         );
     *     }
     * }
     *
     * /// 增加生命周期提示，让编译器知道在函数调用期间这些引用变量是不会出现问题的
     * pub fn search_case_insensitive_fail<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
     *     vec![]
     * }
     * ```
     *
     * ### 2. 编写成功用例
     * ```rust
     *#[cfg(test)]
     * mod tests {
     *     use super::*;
     *
     *     #[test]
     *     fn case_right_result() {
     *         let query = "rust";
     *         let contents = "\
     * Rust:
     * safe, fast, productive.
     * Pick three.";
     *
     *         assert_eq!(
     *             vec!["Rust:"],
     *             search_case_insensitive_right(query, contents)
     *         );
     *     }
     *
     * pub fn search_case_insensitive_right<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
     *     let mut results = Vec::new();
     *     // 遍历迭代每一行
     *     for line in content.lines() {
     *         // 判断是否包含指定的query字符串
     *         if line.to_lowercase().contains(&query.to_lowercase()) {
     *             // 存储搜索内容
     *             results.push(line)
     *         }
     *     }
     *     results
     * }
     * ```
     *
     * ### 3. 优化代码结构
     *
     * 正确测试用例通过后，优化代码结构，方便外部调用，在run函数中，调用搜索函数并将搜索结构输出。
     * 
     * 这里需要则呢将该配置选项标识是否开启大小写敏感，env 包提供了相应的方法读取环境变量，is_ok 方法是 Result 提供的，用于检查是否有值，有就返回 true，没有则返回 false：
     * ```rust
     * pub struct Config {
     *     query: String,
     *     file_path: String,
     *     ignore_case: bool,
     * }
     *
     * // impl 为 Config 实现自定义的方法
     * impl Config {
     *     // 返回Result对象，
     *     pub fn build(args: &[String]) -> Result<Config, &'static str> {
     *         if args.len() < 3 {
     *             return Err("not enough arguments");
     *         }
     *
     *         let file_path = args[1].clone();
     *         let query = args[2].clone();
     *
     *         // Rust 的 env 包提供了相应的方法读取环境变量
     *         let ignore_case = env::var("IGNORE_CASE").is_ok();
     *
     *         Ok(Config {
     *             file_path,
     *             query,
     *             ignore_case,
     *         })
     *     }
     * }
     * ```
     *
     * ### 4. 调用功能代码
     *
     * 最后控制台执行整个程序
     * ```shell
     * $env:IGNORE_CASE=1;cargo run -- D:\workspace\Rust\rust-note\public\poem.txt Body
     * ```
     */

    // 通过类型注释，Rust编译器会将collect方法读取成指定类型
    let args: Vec<String> = env::args().collect();
    // 解构结构体，用unwrap取出Ok的内容，或者在闭包中拿到err错误信息
    let config: Config = Config::build(&args).unwrap_or_else(|err| {
        // 闭包读取err错误信息
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    /*
     *
     * if let 的使用让代码变得更简洁，可读性也更加好，原因是程序并不关注 run 返回的 Ok 值，只需要用 if let 去匹配是否存在错误即可。
     */
    if let Err(e) = run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}
