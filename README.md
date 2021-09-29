<h1 align="center">Handle</h1>

<div align="center">
  <p><strong>A handle trait for asynchronous context pipeline.</strong></p>
</div>

<div align="center">
  <img src="https://img.shields.io/badge/-safety!-success?style=flat-square" alt="Safety!" />
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/handle">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/handle">
    <img src="https://img.shields.io/crates/v/handle.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/handle">
    <img src="https://img.shields.io/crates/d/handle.svg?style=flat-square"
      alt="Download" /></a>
  <!-- Twitter -->
  <a href="https://twitter.com/_fundon">
    <img src="https://img.shields.io/badge/twitter-@__fundon-blue.svg?style=flat-square" alt="Twitter: @_fundon" /></a>
</div>

## Example

```rust
use handle::{Handle, BoxFuture};
use futures::executor::block_on;
use std::{future::Future, pin::Pin, sync::Arc};
use anyhow::Error;

type Result = anyhow::Result<()>;
type Middleware = dyn for<'a> Handle<'a, Context, Result>;

struct Context {
    index: usize,
    middleware: Vec<Arc<Middleware>>,
}

impl Context {
    async fn next(&mut self) -> Result {
        if let Some(m) = self.middleware.pop() {
            m.call(self).await
        } else {
            Ok(())
        }
    }
}

async fn a(cx: &mut Context) -> Result {
    let size = cx.middleware.len();
    let repeat = "-".repeat(2 * size);

    println!("exec Fn a --{}>> {:>2}", repeat, cx.index);

    assert_eq!(cx.index, 0);
    cx.index += 1;
    assert_eq!(cx.index, 1);

    let fut = cx.next().await;

    assert_eq!(cx.index, 1);
    cx.index -= 1;
    assert_eq!(cx.index, 0);

    println!("exec Fn a --{}<< {:>2}", repeat, cx.index);

    fut
}

fn b<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result> {
    let size = cx.middleware.len();
    let repeat = "-".repeat(2 * size);

    println!("exec Fn b --{}>> {:>2}", repeat, cx.index);

    assert_eq!(cx.index, 1);
    cx.index += 1;
    assert_eq!(cx.index, 2);

    Box::pin(async move {
        let fut = cx.next().await;

        assert_eq!(cx.index, 2);
        cx.index -= 1;
        assert_eq!(cx.index, 1);

        println!("exec Fn b --{}<< {:>2}", repeat, cx.index);

        fut
    })
}

fn c(cx: &mut Context) -> BoxFuture<'_, Result> {
    let size = cx.middleware.len();
    let repeat = "-".repeat(2 * size);

    println!("exec Fn c --{}>> {:>2}", repeat, cx.index);

    assert_eq!(cx.index, 2);
    cx.index += 1;
    assert_eq!(cx.index, 3);

    Box::pin(async move {
        let fut = cx.next().await;

        assert_eq!(cx.index, 3);
        cx.index -= 1;
        assert_eq!(cx.index, 2);

        println!("exec Fn c --{}<< {:>2}", repeat, cx.index);

        fut
    })
}

fn d<'a>(cx: &'a mut Context) -> impl Future<Output = Result> + 'a {
    let size = cx.middleware.len();
    let repeat = "-".repeat(2 * size);

    println!("exec Fn d --{}>> {:>2}", repeat, cx.index);

    assert_eq!(cx.index, 3);
    cx.index += 1;
    assert_eq!(cx.index, 4);

    async move {
        let fut = cx.next().await;

        assert_eq!(cx.index, 4);
        cx.index -= 1;
        assert_eq!(cx.index, 3);

        println!("exec Fn d --{}<< {:>2}", repeat, cx.index);

        fut
    }
}

fn e(cx: Pin<&mut Context>) -> impl Future<Output = Result> + '_ {
    let size = cx.middleware.len();
    let repeat = "-".repeat(2 * size);

    println!("exec Fn e --{}>> {:>2}", repeat, cx.index);

    assert_eq!(cx.index, 4);
    cx.index += 1;
    assert_eq!(cx.index, 5);

    async move {
        let fut = cx.next().await;

        assert_eq!(cx.index, 5);
        cx.index -= 1;
        assert_eq!(cx.index, 4);

        println!("exec Fn e --{}<< {:>2}", repeat, cx.index);

        fut
    }
}

async fn f(cx: &mut Context) -> Result {
    let size = cx.middleware.len();
    let repeat = "-".repeat(2 * size);

    println!("exec Fn f --{}>> {:>2}", repeat, cx.index);

    assert_eq!(cx.index, 5);
    cx.index += 1;
    assert_eq!(cx.index, 6);

    let fut = cx.next().await;

    assert_eq!(cx.index, 6);
    cx.index -= 1;
    assert_eq!(cx.index, 5);

    println!("exec Fn f --{}<< {:>2}", repeat, cx.index);

    fut
}

#[derive(Clone)]
struct A {
    index: usize,
}

impl<'a> Handle<'a, Context, Result> for A {
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        Box::pin(async move {
            let size = cx.middleware.len();
            let repeat = "-".repeat(2 * size);

            println!("exec St A --{}>> {:>2}", repeat, cx.index);

            assert_eq!(cx.index, 6);
            cx.index += self.index; // + 1
            assert_eq!(cx.index, 7);

            let fut = cx.next().await;

            assert_eq!(cx.index, 7);
            cx.index -= self.index; // - 1
            assert_eq!(cx.index, 6);

            println!("exec St A --{}<< {:>2}", repeat, cx.index);

            fut
        })
    }
}

struct B {
    index: usize,
}

impl<'a> Handle<'a, Context, Result> for B {
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        Box::pin(async move {
            let size = cx.middleware.len();
            let repeat = "-".repeat(2 * size);

            println!("exec St B --{}>> {:>2}", repeat, cx.index);

            assert_eq!(cx.index, 7);
            cx.index += self.index; // + 2
            assert_eq!(cx.index, 9);

            let fut = cx.next().await;

            assert_eq!(cx.index, 9);
            cx.index -= self.index; // - 2
            assert_eq!(cx.index, 7);

            println!("exec St B --{}<< {:>2}", repeat, cx.index);

            fut
        })
    }
}

struct C {
    index: usize,
}

impl<'a> Handle<'a, Context, Result> for C {
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        Box::pin(async move {
            let size = cx.middleware.len();
            let repeat = "-".repeat(2 * size);

            println!("exec St C --{}>> {:>2}", repeat, cx.index);

            assert_eq!(cx.index, 9);
            cx.index += self.index; // + 3
            assert_eq!(cx.index, 12);

            let fut = cx.next().await;

            assert_eq!(cx.index, 12);
            cx.index -= self.index; // - 3
            assert_eq!(cx.index, 9);

            println!("exec St C --{}<< {:>2}", repeat, cx.index);

            fut
        })
    }
}

fn main() {
    assert!(block_on(async move {
        let mut cx = Context {
            index: 0,
            middleware: Vec::new(),
        };

        let mut v: Vec<Box<Middleware>> = vec![
            Box::new(a),
            Box::new(b),
            Box::new(c),
            Box::new(d),
            Box::new(e),
            Box::new(f),
        ];

        let mut v: Vec<Arc<Middleware>> = vec![
            // Handled it!
            // A Closure cant use `cx.next()` in async block.
            Arc::new(|cx: &mut Context| {
                assert_eq!(cx.index, 12);

                println!("We handled it!");

                async move {
                    // assert_eq!(cx.index, 12); // compiled panic!
                    Ok(())
                }
            }),
            Arc::new(C { index: 3 }),
            Arc::new(B { index: 2 }),
            Arc::new(A { index: 1 }),
            Arc::new(f),
            Arc::new(e),
            Arc::new(d),
            Arc::new(c),
            Arc::new(b),
            Arc::new(a),
        ];

        cx.middleware = v.clone();
        println!("mw 0: {}", v.len());

        let result = cx.next().await?;
        assert_eq!(result, ());

        v.clear();

        v.insert(0, Arc::new(a));
        v.insert(0, Arc::new(b));
        v.insert(0, Arc::new(c));
        v.insert(0, Arc::new(d));
        v.insert(0, Arc::new(e));
        v.insert(0, Arc::new(f));
        v.insert(0, Arc::new(A { index: 1 }));
        v.insert(0, Arc::new(B { index: 2 }));
        v.insert(0, Arc::new(C { index: 3 }));
        // Handled it!
        async fn handler(cx: &mut Context) -> Result {
            assert_eq!(cx.index, 12);

            println!("We handled it!");

            Ok(())
        }
        v.insert(0, Arc::new(handler));

        println!("mw 1: {}", v.len());

        let result = cx.next().await?;
        assert_eq!(result, ());

        Ok::<_, Error>()
    }).is_ok());
}
```

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
