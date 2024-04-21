## 整数与枚举

在 Rust 中，从枚举到整数的转换很容易，但是反过来，就没那么容易，甚至部分实现还挺不安全, 例如使用 transmute。

在实际场景中，从整数到枚举的转换有时还是非常需要的，例如为了可读性，有一个枚举类型，然后需要从外面传入一个整数，用于控制后续的流程走向，此时就需要用整数去匹配相应的枚举。

### 手动匹配

为了实现这个需求，不要求**数字转换枚举**，可以利用**枚举容易转换数字**的特性进行匹配：

```rust
enum Status {
    INIT = 0,
    RUNNING = 1,
    SUCCESS = 2,
    ERROR = 3,
}

let status = 2u8;
let status_enum = match status {
    _ if status == Status::INIT as u8 => Some(Status::INIT),
    _ if status == Status::RUNNING as u8 => Some(Status::RUNNING),
    _ if status == Status::SUCCESS as u8 => Some(Status::SUCCESS),
    _ if status == Status::ERROR as u8 => Some(Status::ERROR),
    _ => None,
};

// 与上面的写法是一样的，只不过多了一个内部变量
let status_enum = match status {
    x if x == Status::INIT as u8 => Some(Status::INIT),
    x if x == Status::RUNNING as u8 => Some(Status::RUNNING),
    x if x == Status::SUCCESS as u8 => Some(Status::SUCCESS),
    x if x == Status::ERROR as u8 => Some(Status::ERROR),
    _ => None,
};
```

### 使用三方库

在手动匹配中，是没有实现**数字转换枚举**流程的，可以使用第三方库 `num-traits` 和 `num-derive` 来实现这个过程：

```rust
use num_derive::FromPrimitive;

// 使用第三方库实现
#[derive(FromPrimitive)]
enum Status2 {
    INIT = 1,
    RUNNING,
    SUCCESS,
    ERROR,
}

match FromPrimitive::from_u8(status) {
    Some(Status2::INIT) => println!("INIT"),
    _ => println!("NOT INIT"),
};
```

使用第三方库后，可以无需手动转换，使用 Optional 即可完成匹配。另外还可以使用一个较新的库: num_enums：

```rust
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Status3 {
    INIT = 1,
    RUNNING,
    SUCCESS,
    ERROR,
}

let num: u8 = Status3::INIT.try_into().expect("转换失败"); // 枚举转换为数字
let enum_item = Status3::try_from(2u8).expect("转换失败"); // 数字转换为枚举
```

### TryFrom 特征

如果不希望使用第三方库，自己也可以使用 TryFrom 实现转换逻辑。

```rust
// 使用TryFrom实现转换逻辑，将给定的数据结合给定的类型，使用TryFrom特征定义的逻辑进行转换
#[derive(Debug)]
enum Status4 {
    INIT = 1,
    RUNNING,
    SUCCESS,
    ERROR,
}
impl TryFrom<i32> for Status4 {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let result = match value {
            x if x == Status4::INIT as i32 => Ok(Status4::INIT),
            x if x == Status4::RUNNING as i32 => Ok(Status4::RUNNING),
            x if x == Status4::SUCCESS as i32 => Ok(Status4::SUCCESS),
            x if x == Status4::ERROR as i32 => Ok(Status4::ERROR),
            _ => Err(()),
        };
        result
    }
}

let enum_num = 2;
match enum_num.try_into() {
    Ok(Status4::INIT) => println!("INIT"),
    _ => println!("NOT INIT"),
}
```

为枚举实现 TryFrom 特征，i32 使用 try_into 方法，try_into 调用的是目标类型的 TryFrom 特征逻辑，再一次应证了 rust 强大的类型系统，它可以使用上下文信息，以进行转换。

**标注合适的类型 + try_into 方法 = 类型自由**

上面还有一个问题，需要为每个类型都定义一遍匹配分支，可以使用宏来解决这个问题：https://course.rs/advance/into-types/enum-int.html#tryfrom--%E5%AE%8F

### std::mem::transmute

这个方法原则上并不推荐，但是有其存在的意义，如果要使用，需要清晰的知道自己为什么使用，这属于 unsafe 代码。

阅读：https://course.rs/advance/into-types/enum-int.html#%E9%82%AA%E6%81%B6%E4%B9%8B%E7%8E%8B-stdmemtransmute

### 总结

枚举非常容易转化成数字，但是数字不容易转换成枚举。可以利用枚举容易转化成数字特性来实现枚举与数字的匹配。

### Code 
```rust
fn main() {
    enum Status {
        INIT = 0,
        RUNNING = 1,
        SUCCESS = 2,
        ERROR = 3,
    }

    let status = 2u8;
    let status_enum = match status {
        _ if status == Status::INIT as u8 => Some(Status::INIT),
        _ if status == Status::RUNNING as u8 => Some(Status::RUNNING),
        _ if status == Status::SUCCESS as u8 => Some(Status::SUCCESS),
        _ if status == Status::ERROR as u8 => Some(Status::ERROR),
        _ => None,
    };
    // 与上面的写法是一样的，只不过多了一个内部变量
    let status_enum = match status {
        x if x == Status::INIT as u8 => Some(Status::INIT),
        x if x == Status::RUNNING as u8 => Some(Status::RUNNING),
        x if x == Status::SUCCESS as u8 => Some(Status::SUCCESS),
        x if x == Status::ERROR as u8 => Some(Status::ERROR),
        _ => None,
    };

    // 使用第三方库实现
    #[derive(FromPrimitive)]
    enum Status2 {
        INIT = 1,
        RUNNING,
        SUCCESS,
        ERROR,
    }

    match FromPrimitive::from_u8(status) {
        Some(Status2::INIT) => println!("INIT"),
        _ => println!("NOT INIT"),
    };

    #[derive(TryFromPrimitive, IntoPrimitive)]
    #[repr(u8)]
    enum Status3 {
        INIT = 1,
        RUNNING,
        SUCCESS,
        ERROR,
    }
    // 枚举转换为数字
    let num: u8 = Status3::INIT.try_into().expect("转换失败");
    // 数字转换为枚举
    let enum_item = Status3::try_from(2u8).expect("转换失败");

    // 使用TryFrom实现转换逻辑，将给定的数据结合给定的类型，使用TryFrom特征定义的逻辑进行转换
    // 为枚举实现TryFrom特征，i32使用try_into方法，try_into调用的是目标类型的TryFrom特征逻辑，再一次应证了rust强大的类型系统，它可以使用上下文信息，以进行转换
    #[derive(Debug)]
    enum Status4 {
        INIT = 1,
        RUNNING,
        SUCCESS,
        ERROR,
    }
    impl TryFrom<i32> for Status4 {
        type Error = ();
        fn try_from(value: i32) -> Result<Self, Self::Error> {
            let result = match value {
                x if x == Status4::INIT as i32 => Ok(Status4::INIT),
                x if x == Status4::RUNNING as i32 => Ok(Status4::RUNNING),
                x if x == Status4::SUCCESS as i32 => Ok(Status4::SUCCESS),
                x if x == Status4::ERROR as i32 => Ok(Status4::ERROR),
                _ => Err(()),
            };
            result
        }
    }

    let enum_num = 2;
    match enum_num.try_into() {
        Ok(Status4::INIT) => println!("INIT"),
        _ => println!("NOT INIT"),
    }
}
```