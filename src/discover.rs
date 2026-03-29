use anyhow::Result;
use flume::RecvTimeoutError;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::net::IpAddr;
use std::time::{Duration, Instant};

pub const SERVICE_TYPE: &str = "_codea-air-code._tcp.local.";

#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum DiscoverEvent {
    Resolved,
}

pub fn discover_devices_with_progress<F>(timeout: Duration, mut on_event: F) -> Result<Vec<Device>>
where
    F: FnMut(DiscoverEvent),
{
    let mdns = ServiceDaemon::new()?;
    let receiver = mdns.browse(SERVICE_TYPE)?;
    let deadline = Instant::now() + timeout;
    let mut devices = Vec::new();

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining.min(Duration::from_millis(250))) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                let host = info.get_addresses().iter().find_map(|ip| match ip {
                    IpAddr::V4(v4) => Some(v4.to_string()),
                    IpAddr::V6(_) => None,
                });
                if let Some(host) = host {
                    let name = info.get_hostname().trim_end_matches('.').to_string();
                    if !devices
                        .iter()
                        .any(|d: &Device| d.host == host && d.port == info.get_port())
                    {
                        let device = Device {
                            name,
                            host,
                            port: info.get_port(),
                        };
                        on_event(DiscoverEvent::Resolved);
                        devices.push(device);
                    }
                }
            }
            Ok(_) => {}
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    mdns.shutdown()?;
    Ok(devices)
}
