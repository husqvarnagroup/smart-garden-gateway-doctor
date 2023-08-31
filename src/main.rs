use core::time::Duration;
use serialport::{available_ports, SerialPort};
use std::env;
use std::io::Read;

fn open_serial_port(path: &str) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    Ok(serialport::new(path, 115_200)
        .timeout(Duration::from_millis(100))
        .open()?)
}

fn remove_non_printable(s: &str) -> String {
    s.chars()
        .filter(|&c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect()
}

fn send(
    serial_port: &mut Box<dyn SerialPort>,
    buf: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    serial_port.write_all(buf)?;
    serial_port.flush()?;
    Ok(())
}

fn receive(serial_port: &mut Box<dyn SerialPort>) -> Option<String> {
    let mut buf: Vec<u8> = vec![0; 1000];
    let bytes_read = serial_port.read(buf.as_mut_slice()).unwrap_or(0);
    if bytes_read == 0 {
        return None;
    }
    let s = remove_non_printable(&String::from_utf8_lossy(&buf));
    if s.is_empty() {
        return None;
    }
    Some(s)
}

fn main() {
    let mut patterns = vec![
        "U-Boot",
        "DRAM:  128 MiB",
        "Net:   eth0: eth@10110000",
        "=>",
    ];

    let mut serial_port: Option<Box<dyn SerialPort>> = None;
    let mut serial_port_name = String::new();

    if let Ok(ports) = available_ports() {
        if ports.len() == 1 {
            serial_port_name = ports[0].port_name.clone();
        } else {
            let choices = ports.into_iter().map(|p| p.port_name).collect();
            serial_port_name = inquire::Select::new("Select serial port", choices)
                .prompt()
                .expect("Invalid selection");
        }
    }

    let mut console_output = String::new();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        console_output = std::fs::read_to_string(file_path).unwrap_or_else(|_| {
            eprintln!("Failed to read file \"{file_path}\"");
            std::process::exit(1);
        });

    } else if let Ok(p) = open_serial_port(serial_port_name.as_str()) {
        serial_port = Some(p);
    }

    let mut timeout_counter = 0;

    loop {
        if let Some(ref mut p) = serial_port {
            if let Some(s) = receive(p) {
                console_output += s.as_str();
                timeout_counter = 0;
            } else {
                timeout_counter += 1;
            }
        }

        if console_output.contains(patterns[0]) {
            println!("{} ✔️", patterns[0]);
            patterns.drain(..1);
            if patterns.is_empty() {
                break;
            }
        }

        if timeout_counter >= 100 {
            println!("{} ❌️", patterns[0]);
            patterns.drain(..1);
            if patterns.is_empty() {
                break;
            }
            continue;
        }

        if let Some(ref mut p) = serial_port {
            send(p, b"x").expect("Failed to write to serial port");
        }
    }
}
