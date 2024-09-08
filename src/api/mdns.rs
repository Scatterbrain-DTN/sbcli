use std::sync::Arc;

pub use super::{error::SbResult, types::ScanResult};
pub use flutter_rust_bridge::DartFnFuture;
pub use scatterbrain::mdns::ServiceScanner;

pub fn service_scanner() -> ServiceScanner {
    ServiceScanner::new()
}

pub async fn discover_devices(
    cb: impl Fn(Vec<ScanResult>) -> DartFnFuture<()> + Send + Sync + 'static,
) -> SbResult<()> {
    discover_devices_impl(Arc::new(cb)).await
}

async fn discover_devices_impl(
    cb: Arc<dyn Fn(Vec<ScanResult>) -> DartFnFuture<()> + Send + Sync>,
) -> SbResult<()> {
    let scanner = ServiceScanner::new();
    scanner
        .mdns_scan(|res| {
            let cb = cb.clone();
            async move {
                cb(res.iter().map(|(k, v)| v.clone().into()).collect()).await;
                Ok(())
            }
        })
        .await?;
    Ok(())
}
