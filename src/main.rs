use cliclack::{confirm, intro, log, outro, select, spinner};
use core::time::Duration;
use serialport::{available_ports, SerialPort};
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
    serial_port: &mut Option<&mut Box<dyn SerialPort>>,
    buf: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(p) = serial_port {
        p.write(buf)?;
        p.flush()?;
    }
    Ok(())
}

fn receive(serial_port: &mut Option<&mut Box<dyn SerialPort>>) -> Option<String> {
    let mut buf: Vec<u8> = vec![0; 1000];
    let bytes_read;
    if let Some(p) = serial_port {
        bytes_read = p.read(buf.as_mut_slice()).unwrap_or(0);
    } else {
        // TODO: remove stdin support
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

fn analyze(
    serial_port: &mut Option<&mut Box<dyn SerialPort>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut patterns = vec![
        "U-Boot",
        "DRAM:  128 MiB",
        "Net:   eth0: eth@10110000",
        "=>",
    ];
    let mut console_output = String::from("");
    let mut timeout_counter = 0;

    loop {
        if let Some(s) = receive(serial_port) {
            console_output += s.as_str();
            timeout_counter = 0;
        } else {
            timeout_counter += 1;
        }

        if let Some(_) = console_output.find(patterns[0]) {
            log::info(format!("{} ✔️", patterns[0]))?;
            patterns.drain(..1);
            if patterns.len() == 0 {
                break;
            }
        }

        if timeout_counter >= 100 {
            log::info(format!("{} ❌️", patterns[0]))?;
            patterns.drain(..1);
            if patterns.len() == 0 {
                break;
            }
            continue;
        }

        send(serial_port, b"x").expect("Failed to write to serial port");
    }

    Ok(())
}

fn exit(code: i32) -> Result<(), Box<dyn std::error::Error>> {
    outro("Bye!")?;
    std::process::exit(code);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    intro("GARDENA smart gateway boot analyzer")?;

    let mut serial_port_name = String::from("");

    if let Ok(serial_port_list) = available_ports() {
        if serial_port_list.len() > 0 {
            let mut sel = select("Select the serial port to use");

            for port in serial_port_list {
                sel = sel.item(port.port_name.clone(), port.port_name.clone(), "");
            }
            serial_port_name = sel.interact()?;
        } else {
            log::error("No serial port found")?;
            exit(1)?;
        }
    }

    let mut serial_port: Option<Box<dyn SerialPort>> = None;

    if let Ok(p) = open_serial_port(serial_port_name.as_str()) {
        serial_port = Some(p);
    } else {
        log::error(format!("Could not open serial port {}", serial_port_name))?;
        exit(1)?;
    }

    loop {
        let mut spinner = spinner();
        spinner.start("");

        analyze(&mut serial_port.as_mut())?;

        spinner.stop("Done");

        if !confirm("Again?").initial_value(true).interact().unwrap() {
            break;
        }
    }

    exit(1)?;

    Ok(())
}
