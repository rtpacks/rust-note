// 餐厅前厅，用于吃饭
fn cleanTable() {}

mod front_of_house {

    fn clean() {
        crate::cleanTable();
        super::cleanTable();
    }

    // 招待客人
    pub mod hosting {
        pub fn add_to_waitlist() {}

        fn seat_at_table() {
            super::clean();
            self::add_to_waitlist();
        }
    }
    // 服务客人
    mod serving {
        fn take_order() {}
        fn serve_order() {}
        fn take_payment() {}
    }
}

pub fn eat_at_restaurant() {
    // 绝对路径
    crate::front_of_house::hosting::add_to_waitlist();
    // 相对路径
    front_of_house::hosting::add_to_waitlist();
}
