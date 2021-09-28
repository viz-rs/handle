//! A handle trait for asynchronous context pipeline.
//!
//! Maintain context in multiple handlers.
//!
//! Examples
//!
//! ```
//! use handle::Handle;
//! use futures::executor::block_on;
//! use std::{future::Future, pin::Pin, sync::Arc};
//!
//! type Result = anyhow::Result<()>;
//! type BoxFuture<'a, T = Result> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
//!
//! struct Context {
//!     index: usize,
//!     middleware: Vec<Box<dyn for<'a> Handle<'a, Context, Output = Result>>>,
//! }
//!
//! impl Context {
//!     async fn next(&mut self) -> Result {
//!         if let Some(m) = self.middleware.pop() {
//!             m.call(self).await
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! async fn a(cx: &mut Context) -> Result {
//!     let size = cx.middleware.len();
//!     let repeat = "-".repeat(2 * size);
//!
//!     println!("exec Fn a --{}>> {:>2}", repeat, cx.index);
//!
//!     cx.index += 1;
//!     let fut = cx.next().await;
//!     cx.index += 1;
//!
//!     println!("exec Fn a --{}<< {:>2}", repeat, cx.index);
//!
//!     fut
//! }
//!
//! #[derive(Clone)]
//! struct A {
//!     index: usize,
//! }
//!
//! impl<'a> Handle<'a, Context> for A {
//!     type Output = Result;
//!
//!     fn call(
//!         &'a self,
//!         cx: &'a mut Context
//!     ) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
//!         Box::pin(async move {
//!             let size = cx.middleware.len();
//!             let repeat = "-".repeat(2 * size);
//!
//!             println!("exec St A --{}>> {:>2}", repeat, cx.index);
//!
//!             cx.index += self.index;
//!             let fut = cx.next().await;
//!             cx.index -= self.index;
//!
//!             println!("exec St A --{}<< {:>2}", repeat, cx.index);
//!
//!             fut
//!         })
//!     }
//! }
//!
//! #[async_std::main]
//! async fn main() -> Result {
//!     let mut cx = Context {
//!         index: 0,
//!         middleware: vec![Box::new(a), Box::new(A { index: 2 })],
//!     };
//!
//!     let result = cx.next().await;
//!     assert!(result.is_ok());
//!     assert_eq!(result.unwrap(), ());
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, rustdoc::missing_doc_code_examples, unreachable_pub)]

use std::{future::Future, pin::Pin};

/// A handle trait for asynchronous context pipeline.
pub trait Handle<'a, Context>
where
    Self: Send + Sync + 'static,
{
    /// Returns `Output`
    type Output;

    /// Invokes the handler within the given `Context` and then returns `Output`
    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;
}

impl<'a, Context, Output, F, Fut> Handle<'a, Context> for F
where
    F: Send + Sync + 'static + Fn(&'a mut Context) -> Fut,
    Fut: Future<Output = Output> + Send + 'a,
    Context: 'a,
{
    type Output = Output;

    #[inline]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin((self)(cx))
    }
}

#[cfg(test)]
mod tests {
    use crate::Handle;
    use anyhow::Error;
    use futures::{executor::block_on, future::BoxFuture};
    use std::{future::Future, pin::Pin, sync::Arc};

    type Result = anyhow::Result<()>;
    type Middleware = dyn for<'a> Handle<'a, Context, Output = Result>;

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

    fn e(cx: &mut Context) -> impl Future<Output = Result> + '_ {
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

    impl<'a> Handle<'a, Context> for A {
        type Output = Result;

        fn call(
            &'a self,
            cx: &'a mut Context,
        ) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
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

    impl<'a> Handle<'a, Context> for B {
        type Output = Result;

        fn call(
            &'a self,
            cx: &'a mut Context,
        ) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
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

    impl<'a> Handle<'a, Context> for C {
        type Output = Result;

        fn call(
            &'a self,
            cx: &'a mut Context,
        ) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
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

    #[test]
    fn futures_rt() {
        assert!(block_on(async move {
            let mut cx = Context {
                index: 0,
                middleware: Vec::new(),
            };

            let mut v: Vec<Box<Middleware>> = vec![];
            v.push(Box::new(f));
            v.push(Box::new(e));
            v.push(Box::new(d));
            v.push(Box::new(c));
            v.push(Box::new(b));
            v.push(Box::new(a));
            v.push(Box::new(A { index: 1 }));
            v.push(Box::new(B { index: 2 }));
            v.push(Box::new(C { index: 3 }));
            v.reverse();
            assert_eq!(v.len(), 9);

            let mut v: Vec<Arc<Middleware>> = vec![];

            // Handled it!
            // A Closure cant use `cx.next()`.
            v.push(Arc::new(|cx: &mut Context| {
                assert_eq!(cx.index, 12);

                println!("We handled it!");

                async move {
                    // assert_eq!(cx.index, 12); // Error
                    Ok(())
                }
            }));
            v.push(Arc::new(C { index: 3 }));
            v.push(Arc::new(B { index: 2 }));
            v.push(Arc::new(A { index: 1 }));
            v.push(Arc::new(f));
            v.push(Arc::new(e));
            v.push(Arc::new(d));
            v.push(Arc::new(c));
            v.push(Arc::new(b));
            v.push(Arc::new(a));

            cx.middleware = v.clone();
            println!("mw 0: {}", v.len());

            let result = cx.next().await;
            assert_eq!(result?, ());

            println!("mw 1: {}", v.len());

            cx.middleware = v.clone();

            let result = cx.next().await;
            assert_eq!(result?, ());

            println!("mw 2: {}", v.len());

            cx.middleware = v.clone();

            let result = cx.next().await;
            assert_eq!(result?, ());

            Ok::<_, Error>(())
        })
        .is_ok());
    }

    #[async_std::test]
    async fn async_std_rt() -> Result {
        let mut cx = Context {
            index: 0,
            middleware: Vec::new(),
        };

        let mut v: Vec<Arc<Middleware>> = vec![];
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

        cx.middleware = v.clone();
        println!("mw 0: {}", v.len());

        let result = cx.next().await;
        assert_eq!(result?, ());

        println!("mw 1: {}", v.len());

        cx.middleware = v.clone();

        let result = cx.next().await;
        assert_eq!(result?, ());

        println!("mw 2: {}", v.len());

        cx.middleware = v.clone();

        let result = cx.next().await;
        assert_eq!(result?, ());

        Ok(())
    }
}
