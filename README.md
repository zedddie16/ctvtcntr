# ctvtcntr 
ctvtcntr (activity counter) is a simple activity monitoring app for hyprland
just a silly project of mine to know how much time I spend by playing games, searching web, coding and etc.

### Todos
1. [x] - basic activity counter application
2. [ ] - duckdb integration 
3. [ ] - telegram bot for data visualising

## How to use
### cargo install
```
cargo install ctvtcntr (not yet available)
```
### include in hyprland.conf
to set on a system start:
```
exec-once = ctvtcntr
```
records are stored in `~/.local/ctvtcntr/app_usage.csv`
### build from source

you can compile binary, and add ctvtcntr as a service to be loaded every hyprland/system start.
Here is how you do it:
```
git clone https://github.com/zedddie16/ctvtcntr
```
Compile binary
```sh
cargo build --release
```
test it, and if needed, set on startup in hyprland.conf:
```
exec-once = <path_to_clone>/target/release/ctvtcntr
```

#### systemd initialization
If systemd is preffered, create ```~/.config/systemd/user/ctvtcntr.service``` 

```
[Unit]
Description=ctvtcntr is a simple hyprland activity counter
After=graphical-session.target

[Service]
ExecStart=ctvtcntr
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```
then just

```sh
systemctl --user enable ctvtcntr.service
systemctl --user start ctvtcntr.service
```

to check status do not forget add --user flag, otherwise it wont show.
```sh
systemctl status --user ctvtcntr
```

🧐🧐🧐if service loading fails during listeting for env, add following line in hyprland.conf🧐🧐🧐
```config
exec-once = systemctl --user import-environment XDG_RUNTIME_DIR HYPRLAND_INSTANCE_SIGNATURE
```
<!-- ; EnvironmentFile=/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/env add under service in unit if env isnt properly passed to the service -->
