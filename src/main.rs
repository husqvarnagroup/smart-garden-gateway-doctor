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

fn enter_uboot(serial_port: &mut Box<dyn SerialPort>) -> String {
    let mut console_output = String::new();
    let mut timeout_counter = 0;

    loop {
        send(serial_port, b"x").expect("Failed to write to serial port");

        if let Some(s) = receive(serial_port) {
            console_output += s.as_str();
            timeout_counter = 0;
        } else {
            timeout_counter += 1;
        }

        if console_output.contains("=>") || timeout_counter >= 100 {
            break;
        }
    }
    console_output
}

fn run_uboot_cmd(serial_port: &mut Box<dyn SerialPort>, cmd: &str) -> String {
    send(serial_port, format!("{cmd}\n").as_bytes()).expect("Failed to write to serial port");

    let mut console_output = String::new();
    let mut timeout_counter = 0;

    loop {
        if let Some(s) = receive(serial_port) {
            console_output += s.as_str();
            timeout_counter = 0;
        } else {
            timeout_counter += 1;
        }

        if console_output.contains("=>") || timeout_counter >= 100 {
            break;
        }
    }
    console_output
}

fn print_lm_issue(issue: &str) {
    println!("! {issue}");
    println!("-> Linux Module (probably) faulty, return to UniElec");
}

fn analyze(serial_port: &mut Option<Box<dyn SerialPort>>, initial_console_output: &str) {
    let patterns_and_issues = vec![
        ("U-Boot SPL", "No U-Boot detected"),
        ("DRAM:  128 MiB", "Wrong RAM size detected"),
        (
            "Net:   eth0: eth@10110000",
            "Ethernet could not be initialized",
        ),
        ("=>", "Could not enter U-Boot shell"),
    ];
    let mut console_output = String::from(initial_console_output);

    if let Some(p) = serial_port {
        console_output += enter_uboot(p).as_str();
    }

    for (pattern, issue) in patterns_and_issues {
        if !console_output.contains(pattern) {
            print_lm_issue(issue);
            return;
        }
    }

    if let Some(p) = serial_port {
        console_output = run_uboot_cmd(p, "mtd list");
    }

    if console_output.contains("Could not find a valid device for spi0.1") {
        print_lm_issue("NAND flash faulty");
        return;
    }

    println!("! No issues found");
}

fn main() {
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

    let mut initial_console_output = String::new();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        initial_console_output = std::fs::read_to_string(file_path).unwrap_or_else(|_| {
            eprintln!("Failed to read file \"{file_path}\"");
            std::process::exit(1);
        });
    } else if let Ok(p) = open_serial_port(serial_port_name.as_str()) {
        serial_port = Some(p);
    }

    loop {
        analyze(&mut serial_port, initial_console_output.as_str());

        if let Ok(false) = inquire::Confirm::new("Continue?")
            .with_default(true)
            .prompt()
        {
            break;
        }
    }
}
