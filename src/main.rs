use rand::Rng;

fn main() {
    /*
     * ## 格式化与输出
     * ### 1. print!，println!，format!，eprint!，eprintln!
     * - print! 将格式化文本输出到标准输出，不带换行符
     * - println! 同上，但是在行的末尾添加换行符
     * - format! 将格式化文本输出到 String 字符串
     * - eprint!，eprintln! 仅应该被用于输出错误信息和进度信息，其它场景都应该使用 print! 系列
     *
     * ### 2. `{}` 与 `{:?}`
     * 与 `{}` 类似，`{:?}` 也是占位符：
     * - {} 适用于实现了 `std::fmt::Display` 特征的类型，用来以更优雅、更友好的方式格式化文本，例如展示给用户
     * - `{:?}` 适用于实现了 `std::fmt::Debug` 特征的类型，用于调试场景
     * - 其实两者的选择很简单，当你在写代码需要调试时，使用 `{:?}`，剩下的场景，选择 `{}`。
     *
     * #### 1. Debug 特征
     *
     * 大多数 Rust 类型都实现了 Debug 特征或者支持派生该特征，对于数值、字符串、数组，可以直接使用 {:?} 进行输出，但是对于结构体，需要派生Debug特征后，才能进行输出，总之很简单。
     * ```rust
     * let i = 3.1415926;
     * let s = String::from("hello");
     * let v = vec![1, 2, 3];
     * println!("{:?}, {:?}, {:?}", i, s, v, );
     * ```
     * #### 2. Display 特征
     * 与大部分类型实现了 Debug 不同，实现了 Display 特征的 Rust 类型并没有那么多，往往需要自定义想要的格式化方式，没有实现 Display 特征就直接使用 `{}` 格式化输出，代码不会通过编译：
     * ```rust
     * let s = String::from("hello");
     * let v = vec![1, 2, 3];
     * println!("{}, {}, {}, {}", s, v); // 不会通过编译
     * ```
     * 如果希望打印复杂类型，可以有其他方法：
     * - 使用 {:?} 或 {:#?}，{:#?} 与 {:?} 几乎一样，唯一的区别在于 `{:#?}` 能更优美地输出内容
     * - 为自定义类型实现 Display 特征
     * - 使用 newtype 为外部类型实现 Display 特征
     *
     * ##### 自定义类型实现 Display 特征
     * 如果需要被格式化输出的类型是定义在当前作用域中的，那么可以为其直接实现 Display 特征。只要实现 Display 特征中的 fmt 方法，即可为自定义结构体 Person 添加自定义输出
     * ```rust
     * use std::fmt;
     *
     * struct Person {
     *     name: String,
     *     age: u8,
     * }
     * impl fmt::Display for Person {
     *     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
     *         write!(f, "姓名{}，年龄{}", self.name, self.age)
     *     }
     * }
     * fn main() {
     *     let p = Person {
     *         name: "sunface".to_string(),
     *         age: 18,
     *     };
     *     println!("{}", p);
     * }
     * ```
     *
     * ##### 为外部类型实现 Display 特征
     * 在 Rust 中，**无法直接为外部类型实现外部特征** ，但是可以使用 newtype 解决此问题，将一个当前作用域的新类型包裹想要格式化输出的外部类型，最后只要为新类型实现 Display 特征，即可进行格式化输出：
     * ```rust
     * struct Array(Vec<i32>);
     * use std::fmt;
     * impl fmt::Display for Array {
     *     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
     *         write!(f, "数组是：{:?}", self.0)
     *     }
     * }
     * fn main() {
     *     let arr = Array(vec![1, 2, 3]);
     *     println!("{}", arr);
     * }
     * ```
     */

    #[derive(Debug)]
    struct Person {
        name: String,
        age: u8,
    }

    let i = 3.1415926;
    let s = String::from("hello");
    let v = vec![1, 2, 3];
    let p = Person {
        name: "sunface".to_string(),
        age: 18,
    };
    println!("{:?}, {:?}, {:?}, {:?}", i, s, v, p);
}
