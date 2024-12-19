# aping
audible ping

## How to build
`cargo run -- google.com` to build and run against google.com in the /debug directory
or 
`cargo build --release` to build release binaries in the /release directory

## How to use

- `aping <target> [-i]` successful ping replies will produce an audible
   alert. using the `-i` flag will result in the inverse where audible
   alerts only sound when a ping times out.
- hit `q` or `Ctrl+C` to quit pinging.
