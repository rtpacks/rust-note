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
