# TCO 

TCO is a tail-call optimization library. It's a proof-of-concept attribute macro you can slap onto item functions to optimize them if they are in tail-calling format.

## Limitations

It's not very smart.

It doesn't actually verify the function is a tail calling function.

It only works on free functions (eg. `fn foo(bar: Bar) -> u32` not in an impl block).

It _can_ have problems with passing a non-copy argument.

It's untested with references.

It doesn't support mutual recursion.

It supports only basic patterns in the function argument (no tuple destructuring).

It can't turn a non-tail calling into a tail calling function.

## Help wanted

This is just a proof-of-concept. I'd love help fleshing it out. 

## Alternatives

 * [tramp](https://docs.rs/tramp/0.3.0/tramp/)
 * [async-recursion](https://docs.rs/async-recursion/0.3.1/async_recursion/)

## Examples

Enough talk, examples!

### Sync Example: Factorial
```rust
#[tco::rewrite]
fn fac_with_acc(n: u128, acc: u128) -> u128 {
    if n > 1 {
        fac_with_acc(n - 1, acc * n)
    } else {
        acc
    }
}
```

expands to 

```rust
fn fac_with_acc(n: u128, acc: u128) -> u128 {
    let mut n = n;
    let mut acc = acc;
    '__tco_loop: loop {
        return {
            if n > 1 {
                {
                    let __tco_0 = (n - 1, acc * n);
                    n = __tco_0.0;
                    acc = __tco_0.1;
                    continue '__tco_loop;
                }
            } else {
                acc
            }
        };
    }
}
```

### Async Example: Factorial 

```rust
#[tco::rewrite]
async fn fac_with_acc(n: u128, acc: u128) -> u128 {
    if n > 1 {
        fac_with_acc(n - 1, acc * n).await
    } else {
        acc
    }
}

``` 

expands to 

```rust
async fn fac_with_acc(n: u128, acc: u128) -> u128 {
    let mut n = n;
    let mut acc = acc;
    '__tco_loop: loop {
        return {
            if n > 1 {
                {
                    let __tco_0 = (n - 1, acc * n);
                    n = __tco_0.0;
                    acc = __tco_0.1;
                    continue '__tco_loop;
                }
            } else {
                acc
            }
        };
    }
}
```

without the tco::rewrite attribute, you instead get the folliwing error:

```
error[E0733]: recursion in an `async fn` requires boxing
 --> $DIR/await_no_tco.rs:6:46
  |
6 | async fn fac_with_acc(n: u128, acc: u128) -> u128 {
  |                                              ^^^^ recursive `async fn`
  |
  = note: a recursive `async fn` must be rewritten to return a boxed `dyn Future`
```
