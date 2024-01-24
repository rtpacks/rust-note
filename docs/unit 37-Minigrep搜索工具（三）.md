## Minigrep

阅读连接；https://course.rs/basic-practice/refactoring.html#%E5%A2%9E%E5%8A%A0%E6%A8%A1%E5%9D%97%E5%8C%96%E5%92%8C%E9%94%99%E8%AF%AF%E5%A4%84%E7%90%86

### 代码改进
- 单一且庞大的函数。main 函数当前执行两个任务：解析命令行参数和读取文件，需要将大的函数拆分成更小的功能单元。
- 配置变量散乱在各处。独立的变量越多，越是难以维护，需要将这些用于配置的变量整合到一个结构体中。
- 细化错误提示。文件不存在、无权限等等都是可能的错误，一条大一统的消息无法给予用户更多的提示。
- 使用错误而不是异常。需要增加合适的错误处理代码，来给予使用者给详细友善的提示。

#### 分离 main 函数
关于如何处理庞大的 main 函数，Rust 社区给出了统一的指导方案:
- 将程序分割为 main.rs 和 lib.rs，并将程序的逻辑代码移动到后者内
- 对部分非常基础的功能，严格来说不算是逻辑代码的一部分，可以放在 main.rs 中

重新梳理后可以得出 main 函数应该包含的功能:
- 解析命令行参数
- 初始化其它配置（生成配置对象）
- 调用 lib.rs 中的 run 函数，以启动逻辑代码的运行（创建run函数，并使用 Box\<dyn std::error::Error\> 抛出错误让外部程序处理）
- 如果 run 返回一个错误，需要对该错误进行处理（main函数处理整体的逻辑逻辑，包括处理异常）

这个方案有一个很优雅的名字: 关注点分离 (Separation of Concerns)。简而言之，main.rs 负责启动程序，lib.rs 负责逻辑代码的运行。
从测试的角度而言，这种分离也非常合理： lib.rs 中的主体逻辑代码可以得到简单且充分的测试，至于 main.rs ？确实没办法针对其编写额外的测试代码，但由于它的代码也很少，容易保证它的正确性。

当前实现步骤：将配置对象和run函数重构至lib.rs，在main.rs中调用并处理错误逻辑


```rust
use std::{env, error::Error, fs, process};
use ilearn::{Config, run};

fn main() {
    // 通过类型注释，Rust编译器会将collect方法读取成指定类型
    let args: Vec<String> = env::args().collect();
    // 解构结构体，用unwrap取出Ok的内容，或者在闭包中拿到err错误信息
    let config = Config::build(&args).unwrap_or_else(|err| {
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
```

```rust
// lib.rs
/**
 * 定义配置数据结构体
 */
pub struct Config {
    query: String,
    file_path: String,
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

        Ok(Config { file_path, query })
    }
}

/**
 * Box<dyn Error> 动态特征对象，只要实现了某个特征就可以进行类型转换
 */
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let content =
        fs::read_to_string(config.file_path).expect("Should have been able to read the file.");

    println!("The file content: \n{content}");

    Ok(())
}
```