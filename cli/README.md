# bsx-cli

A CLI utility for encoding/decoding arbitrary base encoded data.

## Installation

Currently `bsx-cli` is only distributed through crates.io, [install Rust][]
then install with:

```console
> cargo install bsx-cli
Updating crates.io index
[...]

> bsx --help
bsx 0.1.0
A utility for encoding/decoding arbitrary base encoded data.
[...]
```

## Examples

### Encoding some data

```console
> echo '04305e2b2473f058' | xxd -r -p | bsx
he11owor1d
```

### Decoding some data

```console
> echo -n 'he11owor1d' | bsx -d | xxd -p
04305e2b2473f058
```

### Decoding with a different alphabet

```console
> echo -n 'he11owor1d' | bsx -da=ripple | xxd -p
6065e79bba2f78
```

### Encoding with a custom alphabet

```console
> echo 'babce1c947b425' | xxd -r -p | bsx -a='custom(abcdefghijkmnopqrstuvwxyz123456789ABCDEFGHJKLMNPQRSTUVWXYZ)'
he11owor1d
```

[install Rust]: https://www.rust-lang.org/tools/install
