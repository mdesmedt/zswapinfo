# zswapinfo - Show basic zswap stats on Linux

This tool is inspired by `zramctl`. I just wanted to quickly view the current
zswap stats as the system is running. The code reads these stats from
`/sys/module/zswap/parameters` and `/proc/meminfo`.

# Usage

Simply run `zswapinfo`. For example:

```
❯ zswapinfo
Settings
  enabled               Y
  compressor            zstd
  max_pool_percent      20
  accept_threshold      90
  shrinker_enabled      Y

RAM (zswap pool)
  limit                     3176.7 MiB   20% of 15883.6 MiB RAM
  used (compressed)          504.3 MiB   15.9% of limit
  used (raw)                1361.5 MiB
  compression ratio               2.70
  saved                      857.2 MiB

Swap device(s)
  total                     8192.0 MiB
  used                      1662.2 MiB   20.3% of total
  used by zswap             1361.5 MiB   81.9% of used
```
