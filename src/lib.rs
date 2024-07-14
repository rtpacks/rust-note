use std::{env, error::Error, fs};

use crate::front_of_house::hosting;
use front_of_house::serving;

mod front_of_house;

mod back_of_house;

fn cleanTable() {}

/**
 * # Example
 * eat_at_restaurant
 * ```rs
 * let a = 1;
 * println!("{}", a);
 * ```
 */
pub fn eat_at_restaurant() {
    // 绝对路径使用模块（方法）
    crate::front_of_house::hosting::add_to_waitlist();
    // 相对路径使用模块（方法）
    front_of_house::hosting::add_to_waitlist();
    // use绝对路径导入并使用模块（方法）
    hosting::add_to_waitlist();
    // use相对路径导入并使用模块（方法）
    serving::take_payment();
}

pub mod compute {
    /// `add_one` 将指定值加1
    ///
    /// # Examples11
    ///
    /// ```rust
    /// let arg = 3;
    /// let answer = ilearn::compute::add_one(arg);
    ///
    /// assert_eq!(6, answer);
    /// ```
    pub fn add_one(x: i32) -> i32 {
        let a = 1;
        x + a
    }

    /// should_panic 可以测试发生 panic 的测试用例
    /// ```rust,should_panic
    /// let arg = 1;
    /// let answer = ilearn::compute::add_two(arg);
    /// ```
    pub fn add_two(x: i32) -> i32 {
        if x == 1 {
            panic!("x 不能等于 1");
        }
        let a = 2;
        x + a
    }

    /// 在代码块中使用 # 开头的行在文档测试中生效，但会在生成文档时忽略
    /// ```rust,should_panic
    /// let arg = 1;
    /// let answer = ilearn::compute::add_three(arg);
    /// # let answer = ilearn::compute::add_three(arg);
    /// # println!("{}", answer);
    /// ```
    pub fn add_three(x: i32) -> i32 {
        if x == 2 {
            panic!("x 不能等于 2");
        }
        let a = 3;
        x + a
    }
}

/// 直接指定跳转标准库：`add_one` 返回一个[`Option`]类型
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

/**
 * 定义配置数据结构体
 */
pub struct Config {
    query: String,
    file_path: String,
    ignore_case: bool,
}

/**
 * impl 为 Config 实现自定义的方法
 */
impl Config {
    // 返回Result对象，
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let file_path = args[1].clone();
        let query = args[2].clone();

        // Rust 的 env 包提供了相应的方法读取环境变量
        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            file_path,
            query,
            ignore_case,
        })
    }
}

/**
 * Box<dyn Error> 动态特征对象，只要实现了某个特征就可以进行类型转换
 */
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let content =
        fs::read_to_string(config.file_path).expect("Should have been able to read the file.");

    println!("The file content: \n{content}\n");
    println!("=======================================");
    println!("The search results: \n");

    let results = if config.ignore_case {
        search_case_insensitive_right(&config.query, &content)
    } else {
        search_right(&config.query, &content)
    };

    for line in results {
        println!("{line}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn right_result() {
        let query = "rust";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["Rust:"], search_right(query, contents));
    }

    #[test]
    fn case_fail_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(
            vec!["safe, fast, productive."],
            search_case_insensitive_fail(query, contents)
        );
    }

    #[test]
    fn case_right_result() {
        let query = "rust";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(
            vec!["Rust:"],
            search_case_insensitive_right(query, contents)
        );
    }
}

/// 增加生命周期提示，让编译器知道在函数调用期间这些引用变量是不会出现问题的
pub fn search<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
    vec![]
}

pub fn search_right<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    // 遍历迭代每一行
    for line in content.lines() {
        // 判断是否包含指定的query字符串
        if line.contains(query) {
            // 存储搜索内容
            results.push(line)
        }
    }
    results
}

/**
 * 失败用例
 */
pub fn search_case_insensitive_fail<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
    vec![]
}

/**
 * 成功的用例
 */
pub fn search_case_insensitive_right<'a>(query: &'a str, content: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    // 遍历迭代每一行
    for line in content.lines() {
        // 判断是否包含指定的query字符串
        if line.to_lowercase().contains(&query.to_lowercase()) {
            // 存储搜索内容
            results.push(line)
        }
    }
    results
}

pub mod threadpool;
