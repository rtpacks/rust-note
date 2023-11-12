fn take_order() {
    crate::front_of_house::hosting::add_to_waitlist();
    super::hosting::add_to_waitlist();
}
fn serve_order() {}
fn take_payment() {}
