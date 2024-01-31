## Minigrep

一个完整的程序执行日志需要有正确和错误的区分，只用 `println!` 只能实现标准输出，而错误信息更适合输出到标准错误输出(stderr)，`println!` 并不适用。

使用 `cargo run > output.txt` 将日志输出到日志文件。

```txt
// output.txt
Problem parsing arguments: not enough arguments
```

### 标准错误输出 stderr

将错误信息重定向到 stderr 很简单，只需在打印错误的地方，将 println! 宏替换为 eprintln!，注意此时标准错误输出不会输出到日志文件中。

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = minigrep::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
```

错误输出，`eprintln!` 写入到标准错误输出中，默认还是输出在控制台中。

```shell
cargo run > output.txt
```

正常输出，`println!` 写入到标准输出中

```shell
cargo run -- ./public/poem.txt to > output.txt
```

```txt
The file content:
Searching for the
In file poem.txt
With text:
I'm nobody! Who are you?
Are you nobody, too?
Then there's a pair of us - don't tell!
They'd banish us, you know.

How dreary to be somebody!
How public, like a frog
To tell your name the livelong day
To an admiring bog!

=======================================
The search results:

Are you nobody, too?
How dreary to be somebody!
```

### Code

```rust
use ilearn::{run, Config};
use std::{env, error::Error, fs, process};

fn main() {
    // 通过类型注释，Rust编译器会将collect方法读取成指定类型
    let args: Vec<String> = env::args().collect();
    // 解构结构体，用unwrap取出Ok的内容，或者在闭包中拿到err错误信息
    let config: Config = Config::build(&args).unwrap_or_else(|err| {
        // 闭包读取err错误信息
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    /*
     *
     * if let 的使用让代码变得更简洁，可读性也更加好，原因是程序并不关注 run 返回的 Ok 值，只需要用 if let 去匹配是否存在错误即可。
     */
    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
```
