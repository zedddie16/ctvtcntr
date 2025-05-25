# ctvtcntr 
ctvtcntr (activity counter) is a simple activity monitoring app for hyprland
just a silly project of mine to know how much time I spend by playing games, searching web, coding and etc.

### Todos
1. [x] - basic activity counter application
2. [x] - duckdb integration 
3. [ ] - telegram bot for data visualising

## How to use
### Install via Cargo (Recommended)
Once published to `crates.io` (planned for a future release), you'll be able to install it directly:

```sh
cargo install ctvtcntr
```
For now, you can install it from source:

```
# Clone the repository
git clone https://github.com/zedddie16/ctvtcntr
cd ctvtcntr
cargo install --path .
```

### Auto-start with Hyprland (exec-once)
This is the simplest way to ensure ctvtcntr starts with your Hyprland session. 
Add the following line to your ~/.config/hypr/hyprland.conf:
```
# Auto-start ctvtcntr for activity logging
exec-once = ctvtcntr
```
records are stored in `~/.local/share/ctvtcntr/records.db`

#### systemd initialization (restart-on-fail)
If systemd is preffered(e.g for restart on fail), create the service file at `~/.config/systemd/user/ctvtcntr.service`:
```
[Unit]
Description=ctvtcntr is a simple Hyprland activity counter
After=graphical-session.target

[Service]
; Assumes ctvtcntr is in your PATH
ExecStart=ctvtcntr
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```
Enable and start the service:

```sh
systemctl --user daemon-reload
systemctl --user enable ctvtcntr.service
systemctl --user start ctvtcntr.service
```

Check the service status:
```sh
systemctl status --user ctvtcntr
```

🧐🧐🧐if service loading fails during listeting for env, add following line in hyprland.conf🧐🧐🧐
```config
exec-once = systemctl --user import-environment XDG_RUNTIME_DIR HYPRLAND_INSTANCE_SIGNATURE
```
This project is licensed under the [GNU General Public License v3.0 or later](https://www.gnu.org/licenses/gpl-3.0.txt).

<!-- ; EnvironmentFile=/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/env add under service in unit if env isnt properly passed to the service -->
