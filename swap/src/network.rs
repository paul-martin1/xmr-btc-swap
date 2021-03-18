pub mod cbor_request_response;
pub mod encrypted_signature;
pub mod peer_tracker;
pub mod quote;
pub mod spot_price;
pub mod transfer_proof;
pub mod transport;

use libp2p::core::Executor;
use std::future::Future;
use std::pin::Pin;
use tokio::runtime::Handle;

#[allow(missing_debug_implementations)]
pub struct TokioExecutor {
    pub handle: Handle,
}

impl Executor for TokioExecutor {
    fn exec(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        let _ = self.handle.spawn(future);
    }
}
