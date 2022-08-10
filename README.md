# Bitcask
This repo is a Rust implementation of the Bitcask storage engine. Bitcask is a hash-based storage engine. All of the commands will be added towards the end of an append-only log sequentialy. Compared to other storage engine, Bitcask's append-only fashion reduces the write-latency caused by random access significantly. 

For more information about Bitcast itself, please refer [here](https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf).
