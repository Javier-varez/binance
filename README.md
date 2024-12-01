# Binance

## Overview

Implements a query for the Binance `GET /eapi/v1/ticker` endpoint, as well as a 
parser for the data returned by the endpoint.

It actually implements multiple parsers with different tradeoffs and performs comparisons 
between them. Please, refer to [lib.rs](./src/lib.rs) for a description of each parser and
its tradeoffs. The documentation can also be viewed with `rustdoc`.

4 of the parsers use `serde` or `sonic-rs`, while 2 more are custom hand-rolled parsers.
One of the hand-rolled parsers implements a lazy evaluation approach that does not parse 
the json message unless the user wants to look something up inside it. When the user requests 
an action, the parsing is incrementally performed, only to the extent that is actually needed
to recover the requested data. This parser can have very low latency if we are only interested 
in a small subset of all the data parsed by the parser.

In the interest of exploring multiple tradeoffs, the custom parsers are not built to support 
data formats different than the one expected by the `GET /eapi/v1/ticker` endpoint.

## Building and running

All the code has been kept with 0 clippy lints and formatted with rustfmt.

Build the library with:
```shell
cargo build --release
```

Run tests with: 
```shell
cargo run --release
```

Run tests:
```shell
# if you have cargo-nextest
cargo nextest run
# otherwise
cargo test
```

Run benchmarks:
```shell
# if needed, install criterion
cargo binstall cargo-criterion
# then run the benchmarks
cargo criterion
```

Open documentation:
```shell
cargo doc --open
```
  
## Benchmark results

Benchmarking on a Ryzen 3900X CPU with 32 GiB of RAM. Each benchmark has been
computed based on a run of a single reuslt of the API, with the exception of 
the last benchmark, which measures obtaining one of the fields of said record
to demonstrate the potential of the approach.

```
serde                   time:   [807.87 ns 811.29 ns 814.43 ns]

serde_borrowed          time:   [558.72 ns 560.21 ns 562.13 ns]

serde_lazy              time:   [563.37 ns 565.87 ns 568.78 ns]

sonic                   time:   [537.92 ns 538.54 ns 539.18 ns]

custom                  time:   [1.3835 µs 1.3860 µs 1.3888 µs]

custom_lazy             time:   [309.16 ns 309.56 ns 309.98 ns]
```

