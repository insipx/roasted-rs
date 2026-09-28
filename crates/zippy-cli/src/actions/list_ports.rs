use color_eyre::Result;
use tabled::{
    Table, Tabled,
    derive::display,
    settings::{Alignment, Style, object::Columns},
};
use tokio_serial::SerialPortType;

pub fn run() -> Result<()> {
    let ports = tokio_serial::available_ports()?;
    let ports = ports
        .into_iter()
        .map(|p| {
            let mut table = PortTable {
                name: p.port_name,
                port_type: port_type_str(&p.port_type),
                manufacturer: None,
                product: None,
                vendor_id: None,
                product_id: None,
            };
            if let SerialPortType::UsbPort(info) = p.port_type {
                table.manufacturer = info.manufacturer;
                table.product = info.product;
                table.vendor_id = Some(info.vid);
                table.product_id = Some(info.pid);
            }
            table
        })
        .collect::<Vec<PortTable>>();

    let mut table = Table::new(ports);
    table.with(Style::modern());
    table.modify(Columns::first(), Alignment::right());
    println!("{table}");
    Ok(())
}

fn port_type_str(t: &SerialPortType) -> String {
    use SerialPortType::*;
    match t {
        UsbPort(_) => format!("USB"),
        PciPort => format!("PCI"),
        BluetoothPort => format!("Bluetooth"),
        Unknown => format!("Unknown"),
    }
}

#[derive(Tabled)]
struct PortTable {
    name: String,
    port_type: String,
    #[tabled(display("display::option", "unknown"))]
    manufacturer: Option<String>,
    #[tabled(display("display::option", "unknown"))]
    product: Option<String>,
    #[tabled(display("display::option", "unknown"))]
    vendor_id: Option<u16>,
    #[tabled(display("display::option", "unknown"))]
    product_id: Option<u16>,
}
