use std::hint::black_box;

pub fn gen_item_key(idx: usize) -> String {
    black_box(format!("item-{idx}"))
}

pub fn gen_item_value(val: u32) -> String {
    black_box(format!("value-{val}"))
}
