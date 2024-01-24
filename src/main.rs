use ilearn::{run, Config};
use std::{env, error::Error, fs, process};

fn main() {
    /*
     *
     * ## Minigrep
     * 测试用例编写
     *
     * 测试驱动开发模式(TDD, Test Driven Development)：
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
     *     #[test]
     *     fn fail_result() {
     *         let query = "duct";
     *         let contents = "\
     * Rust:
     * safe, fast, productive.
     * Pick three.";
     *
     *         assert_eq!(vec!["safe, fast, productive."], search_fail(query, contents));
     *     }
     * }
     *
     * /// 增加生命周期提示，让编译器知道在函数调用期间这些引用变量是不会出现问题的
     * pub fn search_fail<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
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
     *     fn right_result() {
     *         let query = "duct";
     *         let contents = "\
     * Rust:
     * safe, fast, productive.
     * Pick three.";
     *
     *         assert_eq!(
     *             vec!["safe, fast, productive."],
     *             search_right(query, contents)
     *         );
     *     }
     * }
     *
     * pub fn search_right<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
     *     let mut results = Vec::new();
     *     // 遍历迭代每一行
     *     for line in content.lines() {
     *         // 判断是否包含指定的query字符串
     *         if line.contains(query) {
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
     * 正确测试用例通过后，优化代码结构，方便外部调用，在run函数中，调用搜索函数并将搜索结构输出：
     * ```rust
     * pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
     *     let content =
     *         fs::read_to_string(config.file_path).expect("Should have been able to read the file.");
     *
     *     println!("The file content: \n{content}\n");
     *     println!("=======================================");
     *     println!("The search results: \n");
     *
     *     for line in search_right(&config.query, &content) {
     *         println!("{line}");
     *     }
     *
     *     Ok(())
     * }
     * ```
     * 
     * ### 4. 调用功能代码
     * 
     * 最后控制台执行整个程序
     * ```shell
     * cargo run -- D:\workspace\Rust\rust-note\public\poem.txt  body
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
