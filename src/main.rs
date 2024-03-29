use smart_garden_gateway_doctor::analyzer::analyze;
use smart_garden_gateway_doctor::config::Config;
use smart_garden_gateway_doctor::jig::{open_serial_port, power_off_dut, power_on_dut};
use std::io::prelude::*;

fn exit_with_error(msg: &str) {
    eprint!("{msg}\n\nHit \"return\" to exit...");
    std::io::stderr().flush().unwrap();
    let _ = std::io::stdin().read(&mut [0u8]).unwrap();
    std::process::exit(1);
}

fn main() {
    let mut config = Config::new();

    let serial_port_name = if let Ok(ports) = serialport::available_ports() {
        match ports.len() {
            0 => {
                exit_with_error("No serial ports found");
                std::unreachable!();
            }
            1 => ports[0].port_name.clone(),
            _ => {
                let choices: Vec<String> = ports.into_iter().map(|p| p.port_name).collect();
                let mut default = 0;

                let configured_serial_port = &config.serial_port;
                if !configured_serial_port.is_empty() {
                    if let Some(index) = choices
                        .iter()
                        .position(|p| p.as_str() == configured_serial_port)
                    {
                        default = index;
                    }
                }

                inquire::Select::new("Select serial port", choices)
                    .with_starting_cursor(default)
                    .prompt()
                    .expect("Failed to prompt for serial port")
            }
        }
    } else {
        exit_with_error("Failed to get serial port list");
        std::unreachable!();
    };

    let mut serial_port =
        open_serial_port(serial_port_name.as_str()).expect("Failed to open serial port");

    power_off_dut(&mut serial_port, config.invert_rts);

    config.serial_port = serial_port_name;
    config.save();

    loop {
        if let Ok(false) = inquire::Confirm::new("Continue?")
            .with_default(true)
            .prompt()
        {
            break;
        }

        power_on_dut(&mut serial_port, config.invert_rts);

        analyze(&mut serial_port, std::io::stderr());

        power_off_dut(&mut serial_port, config.invert_rts);
    }
}
