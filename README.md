[![Crates.io](https://img.shields.io/crates/v/fa2.svg)](https://crates.io/crates/fa2)
[![docs.rs](https://img.shields.io/docsrs/fa2)](https://docs.rs/fa2/latest/fa2/)

# Rust `fa2` crate

[Full documentation](https://docs.rs/fa2/latest/fa2/)

---

The `fa2` crate provide a Rust implementation of the
[`ForceAtlas2`](https://doi.org/10.1371/journal.pone.0098679) graph layout
algorithm.

> Jacomy M, Venturini T, Heymann S, Bastian M (2014) ForceAtlas2, a Continuous
> Graph Layout Algorithm for Handy Network Visualization Designed for the Gephi
> Software. PLoS ONE 9(6): e98679. <https://doi.org/10.1371/journal.pone.0098679>

This implementation is generic over its float precision so you can run it with `f32`
or `f64`.

*Running the algorithm*

```rust
// First you need to populate the graph data on which the algorithm will run:
let mut data = FA2Data::<f32>::new();

// Then you need to instantiate settings
let settings = FA2Settings::default();

// Then you can build the layout runner
let layout = settings.build(data);

// Running a fixed number of iterations
layout.run(100);

// Retrieving positions
layout.data().positions()
```
