### ctvtcntr 
ctvtcntr is activity monitoring app for hyprland
just a silly project of mine to know how much time I spend by playing games, searching web, coding and etc.
mayyyy be I will do further stuff and implement something like telegram bot to give me my pc usement data/////

#### how to use
you can compile binary, and add ctvtcntr as a service to be reloaded every time system boot.
Here is how you do it:
Compile binary
```sh
cargo build --release
```
then create ```~/.config/systemd/user/ctvtcntr.service``` with same arguments as in repo, but modify path to actual compiled binary and app_activity.csv file.


```
[Unit]
Description=ctvtcntr is activity counter of mine

[Service]
WorkingDirectory=/home/tuturuu/dev/ctvtcntr/
ExecStart=/home/tuturuu/dev/ctvtcntr/target/release/ctvtcntr
ExecStartPre=/bin/sleep 50
Restart=on-failure

[Install]
WantedBy=default.target
```
then just

```sh
systemctl --user enable ctvtcntr.service
systemctl --user start ctvtcntr.service
```
