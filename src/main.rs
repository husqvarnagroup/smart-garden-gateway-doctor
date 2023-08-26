use core::time::Duration;
use serialport::{available_ports, SerialPort};
use std::env;
use std::io::Read;

fn open_serial_port(path: &str) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    Ok(serialport::new(path, 115_200)
        .timeout(Duration::from_millis(1000))
        .open()?)
}

fn remove_non_printable(s: &str) -> String {
    s.chars()
        .filter(|&c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect()
}

fn receive(serial_port: Option<&mut Box<dyn SerialPort>>) -> Option<String> {
    let mut buf: Vec<u8> = vec![0; 1000];
    let bytes_read;
    if let Some(p) = serial_port {
        bytes_read = p.read(buf.as_mut_slice()).unwrap_or(0);
    } else {
        bytes_read = std::io::stdin()
            .read_to_end(&mut buf)
            .expect("Failed to read from stdin");
    }
    if bytes_read == 0 {
        return None;
    }
    let s = remove_non_printable(&String::from_utf8_lossy(&buf).to_owned());
    if s.len() == 0 {
        return None;
    }
    Some(s)
}

fn main() {
    let mut patterns = vec![
        "U-Boot",
        "DRAM:  128 MiB",
        "Net:   eth0: eth@10110000",
    ];

    let args: Vec<String> = env::args().collect();
    let serial_port_name = if args.len() > 1 {
        &args[1]
    } else {
        "/dev/ttyUSB0"
    };

    let mut serial_port: Option<Box<dyn SerialPort>> = None;

    if let Ok(p) = open_serial_port(serial_port_name) {
        serial_port = Some(p);
    } else {
        eprintln!("Could not open serial port {}", serial_port_name);
        eprintln!("You can pass one of the following as argument:");
        if let Ok(serial_port_list) = available_ports() {
            for port in serial_port_list {
                println!("{}", port.port_name);
            }
        } else {
            eprintln!("Failed to get serial port list");
        }
        println!("\nReading from stdin...")
    }

    let mut console_output = String::from("");
    let mut timeout_counter = 0;

    loop {
        if let Some(s) = receive(serial_port.as_mut()) {
            console_output += s.as_str();
            timeout_counter = 0;
        } else {
            timeout_counter += 1;
        }

        if let Some(_) = console_output.find(patterns[0]) {
            println!("{} ✔️", patterns[0]);
            patterns.drain(..1);
            if patterns.len() == 0 {
                break;
            }
        }

        if timeout_counter >= 10 {
            println!("{} ❌️", patterns[0]);
            patterns.drain(..1);
            if patterns.len() == 0 {
                break;
            }
            continue;
        }
    }
}
