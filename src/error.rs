use thiserror::Error;

pub type Result<T> = std::result::Result<T, PrinterError>;

#[derive(Debug, Error)]
pub enum PrinterError {
    #[error("error USB: {0}")]
    Usb(#[from] nusb::Error),

    #[error("no se pudo leer la configuración activa del dispositivo: {0}")]
    Configuration(#[from] nusb::ActiveConfigurationError),

    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("no hay ninguna impresora seleccionada")]
    NoPrinterSelected,

    #[error("impresora no encontrada: {0} (¿hace falta volver a escanear?)")]
    PrinterNotFound(String),

    #[error("la interfaz USB no tiene un endpoint de salida (bulk OUT)")]
    NoOutEndpoint,

    #[error("tiempo de espera agotado al conectar con la impresora")]
    Timeout,

    #[error("datos inválidos: {0}")]
    InvalidData(String),
}
