# g-raph

> An investigation into practical and optimized implementations of graph algorithms.

## Development

This project, as of yet, uses one simple development technology `rust`.

To install rust, first ensure that you have homebrew installed. Follow instructions [here](https://brew.sh/).

Once you have homebrew, install `rust`:

```
> brew install rustup
> rustup-init
```

That should do it! Open the project in your text-editor of choice, and mess around! I use VS-Code, and use the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) extension to help with development

## Documentation

Documentation can be found [here](https://graph.host.dartmouth.edu/doc/g_raph/index.html)

## Testing

To run all tests, run `cargo test`.

To run specific test, either install `rust-analyzer` on VS-Code, or run the following from the root directory

```shell
> cargo test --package g-raph --lib -- graph::streaming::path-to-test --exact --nocapture
```

## Author

Benjamin Cape
Dartmouth College '22

Professor Amit Chakrabarti
Dartmouth College CS Department
