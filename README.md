cdb
====

This library provides pure Rust support for reading and writing
[CDB][cdb] files.  A CDB file is a constant key-value on-disk hash
table, designed for high-speed lookups.

This version was forked from Bruce Guenter's public domain version at
[cdb-rs][cdbrs], adding no_std stupport for UNIX like OSs and the
interface has changed accordingly.

[cdbrs]https://github.com/bruceg/cdb-rs
[cdb]: https://cdb.cr.yp.to/

## License

GPLv3
