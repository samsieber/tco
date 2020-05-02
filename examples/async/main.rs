#![allow(unused)]

use std::future::Future;

#[tco::rewrite]
async fn fac_with_acc(n: u128, acc: u128) -> u128 {
    if n > 1 {
        fac_with_acc(n - 1, acc * n).await
    } else {
        acc
    }
}

pub fn main() {
    assert_eq!(futures::executor::block_on(fac_with_acc(5, 1)), 120);
}
