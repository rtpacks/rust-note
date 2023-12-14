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

/**
```
let a = 1;
println!("{}", a);
```
*/
pub fn test() {}
