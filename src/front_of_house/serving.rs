use super::hosting;
// use crate::front_of_house::hosting;

fn take_order() {
    crate::front_of_house::hosting::add_to_waitlist();
    hosting::add_to_waitlist();
}

fn serve_order() {}

pub fn take_payment() {}
