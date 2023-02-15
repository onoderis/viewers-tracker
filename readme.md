This is a simple CLI application which tracks viewers on your twitch channel.

Usage:
```
cargo build --release
./target/release/viewers-tracker.exe --channel=forsen
```

Output:
```
2023-02-15 19:35 was here: itsabdu221
2023-02-15 19:35 was here: wace_minduu
2023-02-15 19:36 joined:   jendrula3
2023-02-15 19:36 joined:   puff0
2023-02-15 19:37 left:     joceconc
```

Please take into account that twitch updates viewers list approximately every 2 minutes. 
