# Australian Crypto Capital Gains Calculator

This repo contains my efforts to build a generic tool for consuming crypto trades and determining the capital gains from them.

## Running
If you want to use the GUI and don't want / know how to build it yourself, check out the [Releases](https://github.com/banool/aus-crypto-capital-gains-calculator/releases) tab. You can download a binary for your operating system there.

## Developing

### GUI
```
cargo run -p gui
```
For testing, open `data/fake.csv`. This should succeed.

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

## Missing stuff
There are lots of things you could add to this:

- Additional readers for different data sources.
- Additional calculators, like LIFO. If doing this, you'd likely want to refactor the FIFO code, since most of it would be the same.
- Date range based filtering. For example, filter out transactions that didn't occur in the last financial year.
- Different GUIs. For example, it might be nice to take Bevy for a spin. Ultimately all Rust GUIs are pretty new, so I imagine you could do a lot if you came back to this in a couple of years.
- Even though you can ostensibly avoid importing druid in dependencies with tricks like partial_eq derives on the fields or Rc, when it comes to the Dropdown, I couldn't avoid that. It'd be nice to find a way to use ReaderType as a Dropdown option without having to make implement Data and therefore pollute the rest of the crates with Druid. Instead I made it a String and processed it myself later, which isn't very solid.
