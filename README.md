# pos_printer

Gestor genérico de impresoras POS (ESC/POS) en Rust, con `nusb` + `tokio`.
Imprime por **USB** y por **red** (TCP 9100), y se puede ampliar con otros canales.

## Estructura

```
src/
├── lib.rs         re-exporta todo
├── manager.rs     PrinterManager: impresoras disponibles + la activa + print()
├── document.rs    Document: builder fluido  line("..").bold().align(..)
├── printable.rs   trait Printable (el contrato) y sus impls (&str, String, Vec<T>, Box<dyn>…)
├── builder.rs     EscPosBuilder: bajo nivel (texto, filas, QR, CODE128, cajón…)
├── transport.rs   trait Transport + UsbTransport, TcpTransport, MemoryTransport
├── profile.rs     PrinterProfile: columnas, margen, página de códigos, corte
├── info.rs        PrinterInfo y Connection (Usb / Network / Custom)
├── preview.rs     preview(): ver el resultado como texto, sin impresora
└── error.rs       PrinterError
examples/          factura, factura_trait, regla, red
tests/             pruebas con MemoryTransport (no necesitan impresora)
```

## Gestor: USB y red

```rust
let mut pm = PrinterManager::with_profile(PrinterProfile::mm80().with_margin(2));

pm.scan_usb().await?;                          // llena pm.available() con las USB
let id = pm.add_network("192.168.1.50", 9100); // o una de red

pm.select_index(0).await?;                     // o pm.select(&id).await?
println!("{:?}", pm.current());                // impresora activa

pm.print(&documento).await?;                   // cualquier T: Printable
```

## Builder: `Document`

Los modificadores se aplican al **último elemento** añadido.

```rust
let doc = Document::new()
    .line("CAFÉ EL ERP").bold().double().center()
    .line("NIT 900.123.456-7").center()
    .separator('=')
    .each(items, |d, it| d.row(&it.nombre, &it.precio))
    .row("TOTAL", "$49.385").bold()
    .qr("https://tu-erp.com/f/123").center();

pm.print(&doc).await?;
```

| Contenido | Modificadores (último elemento) | Composición |
|---|---|---|
| `line` `blank` `row` `separator` `feed` `qr` `qr_with` `barcode` `cash_drawer` `raw` | `bold` `underline` `align` `left` `center` `right` `size(w,h)` `double` | `each` `when` `append` `default_align` |

## El contrato `Printable`

Para tipos propios con lógica:

```rust
impl Printable for Recibo {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> { /* … */ }
}
```

El gestor inicializa la impresora antes y avanza papel + corta después (según el
perfil), así que `render` no debe hacerlo. `Document` ya implementa `Printable`.

## Ancho de papel, margen y centrado

La alineación se hace por software, dentro del área de contenido
(`chars_per_line - 2 * margin`), así que funciona igual en cualquier impresora.

```rust
PrinterProfile::mm58()                       // 32 columnas
PrinterProfile::mm80()                       // 48 columnas
PrinterProfile::mm80().with_chars_per_line(42).with_margin(2)
```

## Ver sin imprimir

```rust
println!("{}", pos_printer::preview(&doc, &perfil)?);
// o: cargo run --example factura -- --preview
```

## Otros canales

Implementa `Transport` y actívalo con `pm.use_transport(info, Box::new(mi_transporte))`.
Para tests sin hardware está `MemoryTransport`.

## Impresora que no aparece en el escaneo

```rust
pm.scan_usb_with(&ScanOptions {
    include_vendor_specific: true,
    extra_ids: vec![(0x0416, 0x5011)],
}).await?;
```

## Linux (Fedora)

Si da `Permission denied`, crea `/etc/udev/rules.d/99-pos-printer.rules`:

```
SUBSYSTEM=="usb", ATTR{idVendor}=="XXXX", MODE="0666"
```

y recarga con `sudo udevadm control --reload && sudo udevadm trigger`.

Requiere Rust 1.85 o superior (por `nusb` 0.2).
