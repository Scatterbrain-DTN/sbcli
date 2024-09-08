use api::types::ScanResult;
use scatterbrain::mdns::HostRecord;
pub mod api;

pub use api::error;

impl From<HostRecord> for ScanResult {
    fn from(value: HostRecord) -> Self {
        Self {
            name: value.name,
            addrs: value.addr.into_iter().collect(),
            port: value.port,
        }
    }
}
