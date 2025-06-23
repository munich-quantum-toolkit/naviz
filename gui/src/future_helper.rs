//! Helper-functions for executing [Future]s.
//!
//! See [FutureHelper::new] and its associated functions.

use std::error::Error;
use std::future::Future;
use std::sync::mpsc::Sender;

#[cfg(not(target_arch = "wasm32"))]
use futures::executor::ThreadPool;

/// A future that is [Send] except on wasm and has output `O`.
/// Used for passing into methods of [FutureHelper].
///
/// Non-WASM-version: Requires [Send].
#[cfg(not(target_arch = "wasm32"))]
pub trait SendFuture<O>: Future<Output = O> + Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<O, T: Future<Output = O> + Send> SendFuture<O> for T {}
/// A future that is [Send] except on wasm and has output `O`.
/// Used for passing into methods of [FutureHelper].
///
/// WASM-version: Does not require [Send].
#[cfg(target_arch = "wasm32")]
pub trait SendFuture<O>: Future<Output = O> {}
#[cfg(target_arch = "wasm32")]
impl<O, T: Future<Output = O>> SendFuture<O> for T {}

/// A struct which has some functions to help with executing futures
/// on both web and native.
pub struct FutureHelper {
    #[cfg(not(target_arch = "wasm32"))]
    executor: ThreadPool,
}

#[allow(dead_code)] // Some of these functions may not be used now, but could be used later
impl FutureHelper {
    /// Create a new [FutureHelper].
    /// Will allocate a thread-pool of one thread on native.
    /// Will use `wasm_bindgen_futures` on web.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(Self {})
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let pool = ThreadPool::builder().pool_size(1).create()?;
            Ok(Self { executor: pool })
        }
    }

    /// Execute a future asynchronously
    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute<F: SendFuture<()> + 'static>(&self, f: F) {
        self.executor.spawn_ok(f);
    }

    /// Execute a future asynchronously
    #[cfg(target_arch = "wasm32")]
    pub fn execute<F: SendFuture<()> + 'static>(&self, f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }

    /// Execute a future asynchronously
    /// and send its result to the passed channel
    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute_to<F: SendFuture<O> + 'static, O: 'static + Send>(&self, f: F, r: Sender<O>) {
        self.execute(async move {
            let _ = r.send(f.await);
        });
    }

    /// Execute a future asynchronously
    /// and send its result to the passed channel
    #[cfg(target_arch = "wasm32")]
    pub fn execute_to<F: SendFuture<O> + 'static, O: 'static>(&self, f: F, r: Sender<O>) {
        self.execute(async move {
            let _ = r.send(f.await);
        });
    }

    /// Execute a future asynchronously
    /// and send its result to the passed channel
    /// if it finishes with [Some]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute_maybe_to<F: SendFuture<Option<O>> + 'static, O: 'static + Send>(
        &self,
        f: F,
        r: Sender<O>,
    ) {
        self.execute(async move {
            if let Some(o) = f.await {
                let _ = r.send(o);
            }
        });
    }

    /// Execute a future asynchronously
    /// and send its result to the passed channel
    /// if it finishes with [Some]
    #[cfg(target_arch = "wasm32")]
    pub fn execute_maybe_to<F: SendFuture<Option<O>> + 'static, O: 'static>(
        &self,
        f: F,
        r: Sender<O>,
    ) {
        self.execute(async move {
            if let Some(o) = f.await {
                let _ = r.send(o);
            }
        });
    }
}
