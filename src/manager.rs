use crate::builder::EscPosBuilder;
use crate::error::{PrinterError, Result};
use crate::info::{Connection, PrinterInfo};
use crate::printable::Printable;
use crate::profile::PrinterProfile;
use crate::transport::{TcpTransport, Transport, UsbTransport};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ScanOptions {
    pub include_vendor_specific: bool,
    pub extra_ids: Vec<(u16, u16)>,
}

struct ActivePrinter {
    info: PrinterInfo,
    transport: Box<dyn Transport>,
}

pub struct PrinterManager {
    available: Vec<PrinterInfo>,
    current: Option<ActivePrinter>,
    pub default_profile: PrinterProfile,
}

impl Default for PrinterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PrinterManager {
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            current: None,
            default_profile: PrinterProfile::default(),
        }
    }

    pub fn with_profile(profile: PrinterProfile) -> Self {
        Self {
            default_profile: profile,
            ..Self::new()
        }
    }

    pub async fn scan_usb(&mut self) -> Result<usize> {
        self.scan_usb_with(&ScanOptions::default()).await
    }

    pub async fn scan_usb_with(&mut self, opts: &ScanOptions) -> Result<usize> {
        let profiles: HashMap<String, PrinterProfile> = self
            .available
            .iter()
            .map(|p| (p.id.clone(), p.profile.clone()))
            .collect();

        let mut founds = Vec::new();
        for dev in nusb::list_devices().await? {
            let force = opts
                .extra_ids
                .contains(&(dev.vendor_id(), dev.product_id()));

            let interface = dev
                .interfaces()
                .find(|i| i.class() == 0x07)
                .or_else(|| {
                    if opts.include_vendor_specific {
                        dev.interfaces().find(|i| i.class() == 0xFF)
                    } else {
                        None
                    }
                })
                .or_else(|| if force { dev.interfaces().next() } else { None });

            let Some(interface) = interface else { continue };

            let id = format!(
                "usb:{:04x}:{:04x}:{}:{}",
                dev.vendor_id(),
                dev.product_id(),
                dev.bus_id(),
                dev.device_address()
            );
            let name = match (dev.manufacturer_string(), dev.product_string()) {
                (Some(m), Some(p)) => format!("{m} {p}"),
                (None, Some(p)) => p.to_string(),
                _ => format!("USB {:04x}:{:04x}", dev.vendor_id(), dev.product_id()),
            };
            let profile = profiles
                .get(&id)
                .cloned()
                .unwrap_or_else(|| self.default_profile.clone());

            founds.push(PrinterInfo {
                id,
                name,
                connection: Connection::Usb {
                    vendor_id: dev.vendor_id(),
                    product_id: dev.product_id(),
                    bus_id: dev.bus_id().to_string(),
                    device_address: dev.device_address(),
                    interface_number: interface.interface_number(),
                    serial: dev.serial_number().map(str::to_string),
                },
                profile,
            });
        }

        let n = founds.len();
        self.available.retain(|p| !p.is_usb());
        self.available.extend(founds);
        Ok(n)
    }

    pub fn add_network(&mut self, host: &str, port: u16) -> String {
        let info = PrinterInfo::network(host, port, self.default_profile.clone());
        self.add(info)
    }

    pub fn add_network_with(&mut self, host: &str, port: u16, profile: PrinterProfile) -> String {
        self.add(PrinterInfo::network(host, port, profile))
    }


    pub fn add(&mut self, info: PrinterInfo) -> String {
        let id = info.id.clone();
        match self.available.iter_mut().find(|p| p.id == id) {
            Some(exist) => *exist = info,
            None => self.available.push(info),
        }
        id
    }


    pub fn available(&self) -> &[PrinterInfo] {
        &self.available
    }

    pub fn current(&self) -> Option<&PrinterInfo> {
        self.current.as_ref().map(|c| &c.info)
    }

    pub fn current_id(&self) -> Option<&str> {
        self.current().map(|i| i.id.as_str())
    }

    pub async fn select(&mut self, id: &str) -> Result<()> {
        let info = self
            .available
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| PrinterError::PrinterNotFound(id.to_string()))?;

        let transport: Box<dyn Transport> = match &info.connection {
            Connection::Usb { .. } => Box::new(UsbTransport::open(&info).await?),
            Connection::Network { host, port } => {
                Box::new(TcpTransport::connect(host, *port).await?)
            }
            Connection::Custom { .. } => {
                return Err(PrinterError::InvalidData(
                    "las impresoras Custom se activan con use_transport()".into(),
                ))
            }
        };

        self.current = Some(ActivePrinter { info, transport });
        Ok(())
    }

    pub async fn select_index(&mut self, index: usize) -> Result<()> {
        let id = self
            .available
            .get(index)
            .map(|p| p.id.clone())
            .ok_or_else(|| PrinterError::PrinterNotFound(format!("índice {index}")))?;
        self.select(&id).await
    }

    pub fn use_transport(&mut self, info: PrinterInfo, transport: Box<dyn Transport>) {
        self.add(info.clone());
        self.current = Some(ActivePrinter { info, transport });
    }

    pub fn deselect(&mut self) {
        self.current = None;
    }

    pub fn set_profile(&mut self, id: &str, profile: PrinterProfile) -> Result<()> {
        let info = self
            .available
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| PrinterError::PrinterNotFound(id.to_string()))?;
        info.profile = profile.clone();
        if let Some(c) = self.current.as_mut() {
            if c.info.id == id {
                c.info.profile = profile;
            }
        }
        Ok(())
    }


    pub async fn print<T: Printable + ?Sized>(&mut self, item: &T) -> Result<()> {
        let active = self
            .current
            .as_mut()
            .ok_or(PrinterError::NoPrinterSelected)?;

        let mut doc = EscPosBuilder::new(active.info.profile.clone());
        item.render(&mut doc)?;
        doc.finish();

        active.transport.write_all(&doc.into_bytes()).await
    }
}
