# rs-schema
`rs-schema` is a Rust library that provides types modeling rust code structures.
Types also provide implementations for being serialized back into valid Rust code.

This crate does not provide models for every rust construct. Some can be supported via strings, others are not supported at all. Later versions may expand the supported constructs at the expense of breaking changes. Contributions are welcome!