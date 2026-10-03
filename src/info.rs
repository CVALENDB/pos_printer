use crate::profile::PrinterProfile;
use std::fmt;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Connection {
    Usb {
        vendor_id: u16,
        product_id: u16,
        bus_id: String,
        device_address: u8,
        interface_number: u8,
        serial: Option<String>,
    },
    Network { host: String, port: u16 },
    Custom { label: String },
}

#[derive(Clone, Debug)]
pub struct PrinterInfo {
    pub id: String,
    pub name: String,
    pub connection: Connection,
    pub profile: PrinterProfile,
}

impl PrinterInfo {
    pub fn network(host: &str, port: u16, profile: PrinterProfile) -> Self {
        Self {
            id: format!("tcp:{host}:{port}"),
            name: format!("Impresora de red {host}:{port}"),
            connection: Connection::Network {
                host: host.to_string(),
                port,
            },
            profile,
        }
    }

    pub fn custom(id: &str, name: &str, profile: PrinterProfile) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            connection: Connection::Custom {
                label: id.to_string(),
            },
            profile,
        }
    }

    pub fn is_usb(&self) -> bool {
        matches!(self.connection, Connection::Usb { .. })
    }
}

impl fmt::Display for PrinterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.name, self.id)
    }
}
