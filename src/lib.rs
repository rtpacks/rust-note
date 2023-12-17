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
        let a = 3;
        x + a
    }

    /// ```rust,should_panic
    /// let arg = 1;
    /// let answer = ilearn::compute::add_two(arg);
    /// ```
    pub fn add_two(x: i32) -> i32 {
        if x == 1 {
            panic!("x 不能等于 1");
        }
        let a = 3;
        x + a
    }
}
