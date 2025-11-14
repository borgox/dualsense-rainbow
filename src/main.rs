use hidapi::{HidApi, HidDevice};
use std::thread;
use std::time::{Duration, Instant};

// Vendor ID and Product ID for the DualSense controller
const DUALSENSE_VID: u16 = 0x054C;
const DUALSENSE_PID: u16 = 0x0CE6;

// ANSI Color codes for terminal output
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const RED: &str = "\x1b[31m";
    pub const GRAY: &str = "\x1b[90m";
}

// A struct to manage the DualSense controller
struct DualSenseController {
    device: HidDevice,
    usb_mode: bool,
    last_color: (u8, u8, u8),
    send_count: u64,
    error_count: u64,
}

impl DualSenseController {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("{}{} Searching for DualSense...{}", colors::BOLD, colors::CYAN, colors::RESET);

        let api = HidApi::new()?;

        // Search for the DualSense device
        let device_info = api
            .device_list()
            .find(|d| d.vendor_id() == DUALSENSE_VID && d.product_id() == DUALSENSE_PID)
            .ok_or("DualSense not found")?;

        let device = device_info.open_device(&api)?;

        // Determine connection mode based on interface number
        let usb_mode = device_info.interface_number() == 3;

        println!("{}{}✓ DualSense found!{}", colors::BOLD, colors::GREEN, colors::RESET);
        println!("  {}Mode:{} {}{}{}",
                 colors::GRAY, colors::RESET,
                 colors::BOLD, if usb_mode { "USB" } else { "Bluetooth" }, colors::RESET);
        println!("  {}Vendor ID:{} 0x{:04X}", colors::GRAY, colors::RESET, DUALSENSE_VID);
        println!("  {}Product ID:{} 0x{:04X}", colors::GRAY, colors::RESET, DUALSENSE_PID);
        println!("  {}Interface:{} {}\n", colors::GRAY, colors::RESET, device_info.interface_number());

        Ok(Self {
            device,
            usb_mode,
            last_color: (0, 0, 0),
            send_count: 0,
            error_count: 0,
        })
    }

    fn set_lightbar(&mut self, r: u8, g: u8, b: u8) -> Result<(), Box<dyn std::error::Error>> {
        // Avoid sending the same color repeatedly (reduces flickering)
        if (r, g, b) == self.last_color {
            return Ok(());
        }

        let mut report = if self.usb_mode {
            vec![0; 48]
        } else {
            vec![0; 78]
        };

        if self.usb_mode {
            // USB: report ID 0x02
            report[0] = 0x02;
            report[1] = 0xFF; // Flag to enable edits
            report[2] = 0xF7; // Flag for LEDs and "engines"? (idk translation)

            // LED RGB (offset 45-47 for USB)
            report[45] = r;
            report[46] = g;
            report[47] = b;
        } else {
            // Bluetooth: report ID 0x31
            report[0] = 0x31;
            report[1] = 0x02;
            report[2] = 0xFF;
            report[3] = 0xF7;

            // LED RGB (offset 47-49 for Bluetooth)
            report[47] = r;
            report[48] = g;
            report[49] = b;

            // Calculate CRC32 for Bluetooth
            let crc = calculate_crc32(&report[0..74]);
            report[74] = (crc & 0xFF) as u8;
            report[75] = ((crc >> 8) & 0xFF) as u8;
            report[76] = ((crc >> 16) & 0xFF) as u8;
            report[77] = ((crc >> 24) & 0xFF) as u8;
        }

        match self.device.write(&report) {
            Ok(_) => {
                self.last_color = (r, g, b);
                self.send_count += 1;
                Ok(())
            },
            Err(e) => {
                self.error_count += 1;
                Err(e.into())
            }
        }
    }

    fn get_stats(&self) -> (u64, u64) {
        (self.send_count, self.error_count)
    }
}

// Function to calculate CRC32 (needed for Bluetooth)
fn calculate_crc32(data: &[u8]) -> u32 {
    const CRC32_TABLE: [u32; 256] = generate_crc32_table();

    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[index];
    }
    !crc
}

const fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

// Converts HSV to RGB to create the rainbow effect
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn get_color_name(h: f32) -> (&'static str, &'static str) {
    match h as u32 {
        0..=30 => ("Red", colors::RED),
        31..=90 => ("Yellow", colors::YELLOW),
        91..=150 => ("Green", colors::GREEN),
        151..=210 => ("Cyan", colors::CYAN),
        211..=270 => ("Blue", colors::BLUE),
        271..=330 => ("Magenta", colors::MAGENTA),
        _ => ("Red", colors::RED),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable ANSI escape codes on Windows
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use std::io::stdout;

        unsafe {
            let handle = stdout().as_raw_handle();
            let mut mode: u32 = 0;
            if winapi::um::consoleapi::GetConsoleMode(handle as *mut _, &mut mode) != 0 {
                winapi::um::consoleapi::SetConsoleMode(
                    handle as *mut _,
                    mode | winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                );
            }
        }
    }

    // Dont flame me for this "ui" :3
    println!("\n{}{}╔══════════════════════════════════════╗{}", colors::BOLD, colors::MAGENTA, colors::RESET);
    println!("{}{}║  DualSense Rainbow Lightbar          ║{}", colors::BOLD, colors::MAGENTA, colors::RESET);
    println!("{}{}╚══════════════════════════════════════╝{}\n", colors::BOLD, colors::MAGENTA, colors::RESET);

    let mut controller = DualSenseController::new()?;

    println!("{}{} Starting effect...{}", colors::BOLD, colors::GREEN, colors::RESET);
    println!("{}Press CTRL+C to exit{}\n", colors::GRAY, colors::RESET);

    let mut hue = 0.0;
    let speed = 1.5; // Slower speed for smoother transition
    let target_fps = 60.0;
    let frame_duration = Duration::from_secs_f32(1.0 / target_fps);

    let mut frame_count = 0;
    let mut last_log = Instant::now();
    let log_interval = Duration::from_secs(2);

    let start_time = Instant::now();

    loop {
        let frame_start = Instant::now();

        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);

        match controller.set_lightbar(r, g, b) {
            Ok(_) => {
                frame_count += 1;

                // Log periodico con statistiche
                if last_log.elapsed() >= log_interval {
                    let elapsed = start_time.elapsed().as_secs();
                    let (sent, errors) = controller.get_stats();
                    let (color_name, color_code) = get_color_name(hue);

                    println!("{}[{:02}:{:02}]{} {}{}●{} {} | RGB: ({:3},{:3},{:3}) | Sent: {} | Errors: {} | FPS: {:.1}",
                             colors::GRAY,
                             elapsed / 60,
                             elapsed % 60,
                             colors::RESET,
                             colors::BOLD,
                             color_code,
                             colors::RESET,
                             color_name,
                             r, g, b,
                             sent,
                             errors,
                             frame_count as f32 / last_log.elapsed().as_secs_f32()
                    );

                    frame_count = 0;
                    last_log = Instant::now();
                }
            },
            Err(e) => {
                eprintln!("{}{}✗ Error:{} {}", colors::BOLD, colors::RED, colors::RESET, e);
                thread::sleep(Duration::from_millis(100));
            }
        }

        hue = (hue + speed) % 360.0;

        // Precise timing to avoid flickering
        let frame_time = frame_start.elapsed();
        if frame_time < frame_duration {
            thread::sleep(frame_duration - frame_time);
        }
    }
}