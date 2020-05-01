use tco;

#[tco::rewrite]
fn fac_with_acc(n: u128, acc: u128) -> u128 {
    if n > 1 {
        fac_with_acc(n - 1, acc * n)
    } else {
        120
    }
}

fn main(){
    assert_eq!(fac_with_acc(5,1),120);
}