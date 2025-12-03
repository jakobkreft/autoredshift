# Autoredshift

A tool to automatically adjust screen temperature based on a custom curve.

Autoredshift allows you to define a custom temperature curve throughout the day and automatically adjusts your screen's color temperature using `redshift`. It includes a graphical configuration tool to easily edit your curve.

## Prerequisites

- **Linux** (tested on X11)
- **Redshift**: Ensure `redshift` is installed and in your PATH.
  ```bash
  sudo apt install redshift  # Debian/Ubuntu
  sudo pacman -S redshift    # Arch Linux
  sudo dnf install redshift  # Fedora
  ```
- **Rust/Cargo**: Required to build the project.

## Installation

1. Clone the repository or download the source.
2. Install using Cargo:
   ```bash
   cargo install --path .
   ```
   This will install the `autoredshift` binary to your Cargo bin directory (usually `~/.cargo/bin`).

## Usage

### Run Manually
To calculate the current target temperature and apply it immediately:
```bash
autoredshift
```

### Configuration
To open the graphical configuration editor:
```bash
autoredshift --config
```
- **Double-click** to add a point.
- **Right-click** to remove a point.
- **Drag** points to adjust the curve.
- **Scroll** to zoom.

<img width="956" height="1086" alt="autoredshift_graph" src="https://github.com/user-attachments/assets/9e76c571-dc59-4f2d-918a-f6f5cab19872" />

## Automation

To have Autoredshift run automatically every minute, you can add a cron job.

1. Open your crontab:
   ```bash
   crontab -e
   ```
2. Add the following line (replace `<user>` with your username and verify paths):

   ```bash
   * * * * * DISPLAY=:0 XAUTHORITY=/run/lightdm/<user>/xauthority /home/<user>/.cargo/bin/autoredshift >> /home/<user>/autoredshift.log 2>&1
   ```

   **Note**:
   - `DISPLAY=:0` is standard for most X11 setups.
   - `XAUTHORITY` path may vary depending on your display manager (e.g., `/home/<user>/.Xauthority` or `/run/user/<uid>/gdm/Xauthority`). Check `echo $XAUTHORITY` in your terminal.
   - Ensure the path to `autoredshift` matches where `cargo install` placed it.
