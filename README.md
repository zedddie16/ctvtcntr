### ctvtcntr 
ctvtcntr is activity monitoring app for hyprland
just a silly project of mine to know how much time I spend by playing games, searching web, coding and etc.
mayyyy be I will do further stuff and implement something like telegram bot to give me my pc usement data/////

#### How to use
you can compile binary, and add ctvtcntr as a service to be loaded every time system boot.
Here is how you do it:
Compile binary
```sh
cargo build --release
```
then create ```~/.config/systemd/user/ctvtcntr.service``` with same arguments as in repo, but
modify the path to match actual compiled binary path and app_activity.csv file.


```
[Unit]
Description=ctvtcntr is activity counter of mine
After=graphical-session.target

[Service]
; EnvironmentFile=/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/env
WorkingDirectory=/home/tuturuu/dev/production/activity_counter/
ExecStart=/home/tuturuu/dev/production/activity_counter/target/release/ctvtcntr
Restart=on-failure
; ConditionPathExists=/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/env

[Install]
WantedBy=graphical-session.target
```
then just

```sh
systemctl --user enable ctvtcntr.service
systemctl --user start ctvtcntr.service
```

to check status do not forget add --user flag, otherwise it wont show it.
```sh
systemctl status --user ctvtcntr
```

if service loading fails during listeting for env, add following line in hyprland.conf
```config
exec-once = systemctl --user import-environment XDG_RUNTIME_DIR HYPRLAND_INSTANCE_SIGNATURE
```
