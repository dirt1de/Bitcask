# Bitcask

This repo is a Rust implementation of the Bitcask storage engine. Bitcask is a hash-based storage engine. All of the commands will be added towards the end of an append-only log sequentialy. Compared to other storage engine, Bitcask's append-only fashion reduces the write-latency caused by random access significantly.

For more information about Bitcast itself, please refer [here](https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf).

## Performance Metrics

On my MacBook Pro (15-inch, 2019) with the processor of 2.6 GHz 6-Core Intel Core i7, I have tested the performance of both the `SET` and `GET` commands.

> Notice that the benchmark for each size of key-value pair will be executed multiple times by Criterion.rs to minimize the randomness. That's why the screenshot below shows the distribution.

### `SET` Performance

To test the performance of the `SET` operation, we start a Bitcask instance and `SET` 10000 distinct key-value pairs. In particular, the total size of the key-value-pairs in total ranges from 128 bytes all the way to 4KB. Below is the screenshot for the performance summary of `SET` operation.

#### `SET` Summary

![Bitcask SET performance graph](/benches/set-graphs/set-summary.png "SET performance graph")

#### `SET` with 128 bytes

![Bitcask SET-128-byte performance graph](/benches/set-graphs/set-128-byte.png "SET-128-byte-performance performance graph")

#### `SET` with 256 bytes

![Bitcask SET-256-byte performance graph](/benches/set-graphs/set-256-byte.png "SET-256-byte-performance performance graph")

#### `SET` with 512 bytes

![Bitcask SET-512-byte performance graph](/benches/set-graphs/set-512-byte.png "SET-512-byte-performance performance graph")

#### `SET` with 1 kb

![Bitcask SET-1kb performance graph](/benches/set-graphs/set-1kb.png "SET-1kb-performance performance graph")

#### `SET` with 2 kb

![Bitcask SET-2kb performance graph](/benches/set-graphs/set-2kb.png "SET-2kb-performance performance graph")

#### `SET` with 4 kb

![Bitcask SET-4kb performance graph](/benches/set-graphs/set-4kb.png "SET-4kb-performance performance graph")

### `GET` Performance

To test the performance of the `GET` operation, we first create a Bitcask instance and `SET` 10000 key-value paris as we did in the `SET` benchmark to construct our in-memory KeyDir. After this step, we begin to benchmark the time it takes to `GET` all the 10000 key-value pairs. That is to say, the time we spent on executing `SET` commands for the KeyDir construction will not be included. Below is the screenshot for the performance summary of `GET` operation.

#### `GET` Summary

![Bitcask GET performance graph](/benches/get-graphs/get-summary.png "GET performance graph")

#### `GET` with 128 bytes

![Bitcask GET-128-byte performance graph](/benches/get-graphs/get-128-byte.png "GET-128-byte-performance performance graph")

#### `GET` with 256 bytes

![Bitcask GET-256-byte performance graph](/benches/get-graphs/get-256-byte.png "GET-256-byte-performance performance graph")

#### `GET` with 512 bytes

![Bitcask GET-512-byte performance graph](/benches/get-graphs/get-512-byte.png "GET-512-byte-performance performance graph")

#### `GET` with 1 kb

![Bitcask GET-1kb performance graph](/benches/get-graphs/get-1kb.png "GET-1kb-performance performance graph")

#### `GET` with 2 kb

![Bitcask GET-2kb performance graph](/benches/get-graphs/get-2kb.png "GET-2kb-performance performance graph")

#### `GET` with 4 kb

![Bitcask GET-4kb performance graph](/benches/get-graphs/get-4kb.png "GET-4kb-performance performance graph")
