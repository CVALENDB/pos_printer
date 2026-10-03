use pos_printer::{Align, Document, PrinterManager, PrinterProfile};


struct Producto {
    nombre: &'static str,
    cantidad: i32,
    precio_unitario: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = vec![
        Producto { nombre: "Café Espresso Doble", cantidad: 2, precio_unitario: 2500.0 },
        Producto { nombre: "Muffin de Arándanos", cantidad: 1, precio_unitario: 3800.0 },
        Producto { nombre: "Agua Mineral Con Gas", cantidad: 1, precio_unitario: 1800.0 },
        Producto { nombre: "Tarta de Queso Red Velvet", cantidad: 1, precio_unitario: 5500.0 },
    ];


    let mut subtotal = 0.0;
    for item in &items {
        subtotal += item.cantidad as f64 * item.precio_unitario;
    }
    let iva = subtotal * 0.19;
    let total = subtotal + iva;


    let factura = Document::new()
        .line("CAFÉ EL ERP S.A.S.").bold().center()
        .line("NIT: 900.123.456-7").center()
        .line("Calle 45 # 12-34, Bogotá").center()
        .line("Tel: (601) 555-1234").center()
        .blank()

        .line("FACTURA ELECTRÓNICA DE VENTA").bold().center()
        .line("Nro: FE-48291").center()
        .separator('-')
        .row("Fecha: 03/10/2026", "Hora: 12:01 PM")
        .row("Cajero: Cristian", "Terminal: POS-01")
        .row("Cliente: Consumidor Final", "C.C.: 22222222")
        .separator('=')


        .row("DESCRIPCION (Cant x Valor)", "TOTAL")
        .separator('-')


        .each(items, |doc, item| {
            let detalle_cant = format!("{} x ${:.2}", item.cantidad, item.precio_unitario);
            let total_item = format!("${:.2}", item.cantidad as f64 * item.precio_unitario);

            doc.line(item.nombre)
                .row(&detalle_cant, &total_item)
        })
        .separator('-')


        .row("SUBTOTAL", &format!("${:.2}", subtotal))
        .row("IVA (19%)", &format!("${:.2}", iva))
        .row("TOTAL A PAGAR", &format!("${:.2}", total)).bold()
        .separator('=')


        .row("Metodo de Pago: EFECTIVO", &format!("${:.2}", total))
        .row("Recibido:", &format!("${:.2}", total + 4000.0))
        .row("Cambio:", "$4,000.00")
        .blank()


        .line("Resolución DIAN No. 18760001234").center()
        .line("Habilitada desde 2026/01/01 al 2027/01/01").center()
        .line("Rango autorizado: FE-1 al FE-10000").center()
        .blank()
        .line("¡Gracias por tu compra!").center()
        .line("Vuelve pronto").center()
        .blank()

        .qr("https://www.youtube.com").center();

    let mut pm = PrinterManager::with_profile(PrinterProfile::mm80().with_margin(3));

    pm.add_network("0.0.0.0", 9100);

    pm.select_index(0).await?;
    pm.print(&factura).await?;

    println!("¡Factura enviada con éxito a escpresso!");
    Ok(())
}
