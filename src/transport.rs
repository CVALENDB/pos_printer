use crate::error::{PrinterError, Result};
use crate::info::{Connection, PrinterInfo};
use async_trait::async_trait;
use nusb::descriptors::TransferType;
use nusb::io::EndpointWrite;
use nusb::transfer::{Bulk, Out};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[async_trait]
pub trait Transport: Send {
    async fn write_all(&mut self, data: &[u8]) -> Result<()>;
}


pub struct UsbTransport {
    writer: EndpointWrite<Bulk>,
    _interface: nusb::Interface,
}

impl UsbTransport {
    pub async fn open(info: &PrinterInfo) -> Result<Self> {
        let Connection::Usb {
            vendor_id,
            product_id,
            bus_id,
            device_address,
            interface_number,
            ..
        } = &info.connection
        else {
            return Err(PrinterError::InvalidData(
                "UsbTransport necesita una conexión USB".into(),
            ));
        };

        let candidates: Vec<_> = nusb::list_devices()
            .await?
            .filter(|d| d.vendor_id() == *vendor_id && d.product_id() == *product_id)
            .collect();

        let dev_info = candidates
            .iter()
            .find(|d| d.bus_id() == bus_id && d.device_address() == *device_address)
            .or_else(|| candidates.first())
            .ok_or_else(|| PrinterError::PrinterNotFound(info.id.clone()))?;

        let device = dev_info.open().await?;
        let ep_out = find_bulk_out(&device, *interface_number)?;

        let interface = device
            .detach_and_claim_interface(*interface_number)
            .await?;
        let endpoint = interface.endpoint::<Bulk, Out>(ep_out)?;

        Ok(Self {
            writer: endpoint.writer(4096),
            _interface: interface,
        })
    }
}

fn find_bulk_out(device: &nusb::Device, interface_number: u8) -> Result<u8> {
    let config = device.active_configuration()?;
    for group in config.interfaces() {
        if group.interface_number() != interface_number {
            continue;
        }
        for alt in group.alt_settings() {
            if alt.alternate_setting() != 0 {
                continue;
            }
            for ep in alt.endpoints() {
                if ep.address() & 0x80 == 0 && ep.transfer_type() == TransferType::Bulk {
                    return Ok(ep.address());
                }
            }
        }
    }
    Err(PrinterError::NoOutEndpoint)
}

#[async_trait]
impl Transport for UsbTransport {
    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data).await?;
        self.writer.flush().await?;
        Ok(())
    }
}

pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let stream = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect((host, port)))
            .await
            .map_err(|_| PrinterError::Timeout)??;
        Ok(Self { stream })
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data).await?;
        self.stream.flush().await?;
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct MemoryTransport {
    data: Arc<Mutex<Vec<u8>>>,
}

impl MemoryTransport {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bytes(&self) -> Vec<u8> {
        self.data.lock().unwrap().clone()
    }
}

#[async_trait]
impl Transport for MemoryTransport {
    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.data.lock().unwrap().extend_from_slice(data);
        Ok(())
    }
}
