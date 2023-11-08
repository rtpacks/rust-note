// 餐厅前厅，用于吃饭

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
