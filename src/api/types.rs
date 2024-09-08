use flutter_rust_bridge::frb;
use scatterbrain::mdns::HostRecord;
pub use std::collections::BTreeMap;
pub use std::net::IpAddr;

#[frb(mirror(HostRecord))]
pub struct ScanResult {
    pub name: String,
    pub addrs: Vec<IpAddr>,
    pub port: u16,
}

#[frb(mirror(ServiceScanner))]
pub struct _ServiceScanner(tokio::sync::RwLock<BTreeMap<String, HostRecord>>);

#[frb(mirror(Ipv4Addr))]
pub struct Ipv4Address {
    octets: [u8; 4],
}

#[frb(mirror(Ipv6Addr))]
pub struct Ipv6Addres {
    octets: [u8; 16],
}

#[frb(mirror(IpAddr))]
pub enum IpAddress {
    V4(Ipv4Address),
    V6(Ipv6Addres),
}
