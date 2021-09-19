# Australian Crypto Capital Gains Calculator

## Running

### GUI
todo

### CLI
To run the CLI, loading up one file using the CoinJar reader, try this:
```
cargo run -p cli -- --calculator fifo --paths data/fake.csv --readers coinjar
```

Output:
```
[2021-09-19T02:56:59Z INFO  cli] Capital gain for BTC: $2.00 AUD
```

If you would like to see debug output, include `-d`.
