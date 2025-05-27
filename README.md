# windows-gif-screenshotter

**⚠️ IMPORTANT NOTICE:**  
This tool is slated for a rewrite with Linux (Wayland) support, as I no longer use Windows & Wayland does not support global key listeners.


## Overview

`windows-gif-screenshotter` is a lightweight screenshot utility designed to capture **animated GIFs** instead of static images. Ideal for quick visual demonstrations, bug reports, or sharing short screen actions.

## How It Works

By leveraging Rust crates like [`xcap`](https://crates.io/crates/xcap) for screen capture and [`rdev`](https://crates.io/crates/rdev) for global input listening, the tool allows users to:

1. Select a screen region.
2. Capture a series of screenshots.
3. Automatically encode and save them as a `.gif` file.

Captured GIFs are saved to a target directory automatically after recording.

## Example Output

This GIF was taken using the tool:  
![📷 screenshot.gif](https://github.com/Bloodhundur/windows-gif-screenshotter/blob/main/screenshot.gif?raw=true)

## Upcoming Features

- Adjustable **frame rate** and **frame count** via a user interface.
- **Overlay boundary box** during selection for easier visual alignment.
