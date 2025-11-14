# DualSense Rainbow Lightbar

A **fun, colorful Rust program** that turns your **PlayStation 5 DualSense controller's lightbar into a smooth rainbow cycle** — perfect for adding flair to your setup!

Uses the `hidapi` crate to communicate directly with the controller over **USB or Bluetooth**, with proper report formatting and CRC32 for Bluetooth mode.

---

## Features

- Smooth **60 FPS rainbow animation** using HSV to RGB conversion
- Works in **both USB and Bluetooth** modes
- **No flickering** — avoids resending the same color
- **Real-time stats**: FPS, sent packets, errors, current color
- Colorful terminal UI with ANSI styling (Windows supported!)
- Graceful error handling and logging
- Press `CTRL+C` to exit

---

## Download & Run (No Rust Required!)

**Pre-built binaries** — just download and run. No Rust, no compilation needed!

| Platform | Download |
|--------|----------|
| **Windows (x64)** | [`dualsense-rainbow.exe`](https://github.com/borgox/dualsense-rainbow/releases/latest/download/dualsense-rainbow-windows.exe) |
| **Linux (x86_64)** | [`dualsense-rainbow`](https://github.com/borgox/dualsense-rainbow/releases/latest/download/dualsense-rainbow-linux) |
| **macOS (Universal)** | [`dualsense-rainbow-macos`](https://github.com/borgox/dualsense-rainbow/releases/latest/download/dualsense-rainbow-macos) |

### How to Run

1. **Download** the file for your operating system
2. **Make it executable** (Linux/macOS):
   ```bash
   chmod +x dualsense-rainbow*
    ```
   3. Run it:
    ```bash
    ./dualsense-rainbow*
     ```
   > Plug in your DualSense controller and watch the rainbow begin!
   

### Build from Source (Requires Rust)
If you have Rust and Cargo installed, you can build the project from source:
```bash
git clone https://github.com/borgox/dualsense-rainbow.git
cd dualsense-rainbow
cargo run --release
```

### Linux HID Permissions
On Linux, you may need to set up udev rules to allow non-root access to the
DualSense controller. Create a file at `/etc/udev/rules.d/99-dualsense.rules` with the following content:
```
SUBSYSTEM=="usb", ATTRS{idVendor}=="054c", ATTRS{idProduct}=="0ce6", MODE="0666"
```
Then reload udev rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details

