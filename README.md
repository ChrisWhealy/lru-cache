# Small Threadsafe Implementation of a Least Recently Used Cache

This is an exercise in implementing an LRU cache, then writing some benchmarks to examine performance

## Testing

`cargo nextest run --nocapture`

## Benchmarking

Single threaded tests `cargo bench --bench single_threaded`

Multi-threaded tests `cargo bench --bench multi_threaded`
