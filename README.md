# PSXemcee

PSXemcee is a library developed specifically for communicating with a PlayStation 1 / PlayStation X :tm: memory card via the GPIO pins on a Raspberry Pi.

Currently both reading and writing of arbitrary data is supported. The API for this crate is unfinished, and will be changing in future versions.

## Setup

### PSX Memory Card Pinout

```
 ----------------
| 12 | 345 | 678 |
 ----------------
POV: Looking at the Memory card
```
Pinout:
1. DAT  // Data pin
1. CMD  // Command pin
1. 7.6v // 7.6v input
1. GND  // Ground pin
1. 3.3v // 3.3v input
1. SEL  // Chip select pin
1. CLK  // Clock pin
1. ACK  // Acknowledge pin

For more information, see: https://psx-spx.consoledev.net/controllersandmemorycards/#controller-and-memory-card-signals

### Connection
To connect a PSX memory card to the Pi, use the following mapping:
```rust
/// Data GPIO pin
const DAT_GPIO: u8 = 23;
/// Command GPIO pin
const CMD_GPIO: u8 = 24;
/// Chip Select GPIO pin
const SEL_GPIO: u8 = 17;
/// Clock GPIO pin
const CLK_GPIO: u8 = 27;
/// Acknowledge GPIO pin
const ACK_GPIO: u8 = 22;
```
Or alternatively update these fields in `lib.rs` to whichever pins you want to use.

Be sure to connect GND to a ground GPIO, and 3.3v to a 3.3v GPIO! Do not connect the 7.6v pin, it is not needed.

### Build

To build PSXemcee, run:
```
cargo build
```

### Run

```
Usage: psxemcee --file <FILE> <COMMAND>

Commands:
  read-all     Read entire memory card
  read-frame   Read a specific frame
  read-block   Read a specific block
  status       Get memory card status
  write-all    Write file to memory card
  write-frame  Write frame to memory card
  write-block  Write block to memory card
  help         Print this message or the help of the given subcommand(s)

Options:
  -f, --file <FILE>  Filepath to save memory card data to, or write to the memory card from
  -h, --help         Print help information
```

### Example
```
[geno@alarmpi psxemcee]$ sudo psxemcee -f /tmp/out read-frame -o 128            # Read original frame number 128
Memory card read complete!
[geno@alarmpi psxemcee]$ hexdump -C /tmp/out                                    # View original data
00000000  53 43 13 01 82 76 82 89  82 8c 82 84 81 40 82 60  |SC...v.......@.`|
00000010  82 92 82 8d 82 93 81 40  82 65 82 68 82 6b 82 64  |.......@.e.h.k.d|
00000020  82 4f 82 51 81 40 82 6b  82 75 82 50 82 55 81 40  |.O.Q.@.k.u.P.U.@|
00000030  81 40 81 40 81 40 81 40  81 40 81 40 81 40 81 40  |.@.@.@.@.@.@.@.@|
00000040  81 40 81 40 00 00 00 00  00 00 00 00 00 00 00 00  |.@.@............|
00000050  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
00000060  00 00 02 80 50 8c 15 98  00 94 63 b8 e6 e0 68 e9  |....P.....c...h.|
00000070  ea f1 6c fe 69 80 0e 91  b4 a1 59 b2 1f c3 ff ff  |..l.i.....Y.....|
00000080
[geno@alarmpi psxemcee]$ dd if=/dev/random of=/tmp/rand bs=1 count=128          # Generate random data to use for writing
128+0 records in
128+0 records out
128 bytes copied, 0.0015776 s, 81.1 kB/s
[geno@alarmpi psxemcee]$ hexdump -C /tmp/rand                                   # View random data
00000000  5f 5e ee cc d3 66 56 c7  96 b9 92 be 66 af f1 af  |_^...fV.....f...|
00000010  28 90 ab ec 1b 2c 97 ec  95 cb 52 22 16 6c 27 96  |(....,....R".l'.|
00000020  bc 66 b2 a0 52 8b 84 7d  9e b4 31 f9 2b ae 79 a3  |.f..R..}..1.+.y.|
00000030  2a e1 8d 85 c6 a6 fb 83  4b 07 2f 11 10 e1 aa 53  |*.......K./....S|
00000040  21 fe 65 21 74 d7 be 4c  89 25 fe 4d ed ca cf 22  |!.e!t..L.%.M..."|
00000050  51 8b 31 dd 55 d2 88 61  2e 46 02 24 07 b4 05 63  |Q.1.U..a.F.$...c|
00000060  85 5b 82 cf 4e c2 2e f3  3e 40 e5 f6 3f bb 00 4d  |.[..N...>@..?..M|
00000070  ec d5 d9 cc 71 17 51 d2  ae ef d7 d2 37 63 23 b8  |....q.Q.....7c#.|
00000080
[geno@alarmpi psxemcee]$ sudo psxemcee -f /tmp/rand write-frame -o 128          # Write random data to the memory card to frame 128
Memory card write complete!
[geno@alarmpi psxemcee]$ sudo psxemcee -f /tmp/out-mod read-frame -o 128        # Read frame 128 again
Memory card read complete!
[geno@alarmpi psxemcee]$ hexdump -C /tmp/out-mod                                # View updated data
00000000  5f 5e ee cc d3 66 56 c7  96 b9 92 be 66 af f1 af  |_^...fV.....f...|
00000010  28 90 ab ec 1b 2c 97 ec  95 cb 52 22 16 6c 27 96  |(....,....R".l'.|
00000020  bc 66 b2 a0 52 8b 84 7d  9e b4 31 f9 2b ae 79 a3  |.f..R..}..1.+.y.|
00000030  2a e1 8d 85 c6 a6 fb 83  4b 07 2f 11 10 e1 aa 53  |*.......K./....S|
00000040  21 fe 65 21 74 d7 be 4c  89 25 fe 4d ed ca cf 22  |!.e!t..L.%.M..."|
00000050  51 8b 31 dd 55 d2 88 61  2e 46 02 24 07 b4 05 63  |Q.1.U..a.F.$...c|
00000060  85 5b 82 cf 4e c2 2e f3  3e 40 e5 f6 3f bb 00 4d  |.[..N...>@..?..M|
00000070  ec d5 d9 cc 71 17 51 d2  ae ef d7 d2 37 63 23 b8  |....q.Q.....7c#.|
00000080
[geno@alarmpi psxemcee]$ sudo psxemcee -f /tmp/out write-frame -o 128           # Write out original data back to frame 128
Memory card write complete!
[geno@alarmpi psxemcee]$ sudo psxemcee -f /tmp/out-mod-revert read-frame -o 128 # Read frame 128 again
Memory card read complete!
[geno@alarmpi psxemcee]$ hexdump -C /tmp/out-mod-revert                         # View restored original data
00000000  53 43 13 01 82 76 82 89  82 8c 82 84 81 40 82 60  |SC...v.......@.`|
00000010  82 92 82 8d 82 93 81 40  82 65 82 68 82 6b 82 64  |.......@.e.h.k.d|
00000020  82 4f 82 51 81 40 82 6b  82 75 82 50 82 55 81 40  |.O.Q.@.k.u.P.U.@|
00000030  81 40 81 40 81 40 81 40  81 40 81 40 81 40 81 40  |.@.@.@.@.@.@.@.@|
00000040  81 40 81 40 00 00 00 00  00 00 00 00 00 00 00 00  |.@.@............|
00000050  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
00000060  00 00 02 80 50 8c 15 98  00 94 63 b8 e6 e0 68 e9  |....P.....c...h.|
00000070  ea f1 6c fe 69 80 0e 91  b4 a1 59 b2 1f c3 ff ff  |..l.i.....Y.....|
00000080
[geno@alarmpi psxemcee]$
```

## References
https://psx-spx.consoledev.net/controllersandmemorycards/#controller-and-memory-card-signals
