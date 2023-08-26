
# GARDENA Smart Gateway Boot Analyzer

Read from serial port (default: /dev/ttyUSB0):
```
cargo run /dev/ttyUSB1
```

Read from file:
```
cat test_data/Finding\ 004\ -\ U-Boot\ Net\ not\ loading.txt | cargo run
```
