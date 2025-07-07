# DRIView

DRIView is a tool that shows you what program is running on what gpu when using DRI_PRIME by analyzing which process has which gpu file descriptor open

## Example usage

```sh
$ driview
Found GPU: card1 - 0x8086:0x0046
Found GPU: renderD129 - 0x8086:0x0046
Found GPU: card0 - 0x1002:0x68e4
Found GPU: renderD128 - 0x1002:0x68e4

Intel Corporation Core Processor Integrated Graphics Controller
(8086:0046) [0000:00:02.0] is used by:
(1954) Xwayland
(1954) Xwayland
(1954) Xwayland
(1954) Xwayland
(1954) Xwayland
(1954) Xwayland
(3992) dolphin
(3992) dolphin
(3992) dolphin
(3992) dolphin
(4817) konsole
(4817) konsole
(4817) konsole
(4910) glxgears
(5177) dolphin

Advanced Micro Devices, Inc. [AMD/ATI] Robson CE [Radeon HD 6370M/7370M]
(1002:68e4) [0000:01:00.0] is used by:
(4910) glxgears
(4910) glxgears
(4910) glxgears
```
