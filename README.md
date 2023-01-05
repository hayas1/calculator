# Calculator [![Rust](https://github.com/hayas1/calculator/actions/workflows/rust.yml/badge.svg)](https://github.com/hayas1/calculator/actions/workflows/rust.yml)
Calculate string expression.
- Scalar numeric only. Variables, functions, or matrices are not supported.
- Four arithmetic operation only. Powers, power roots, or factorials are not supported.
- (CLI) Support precise calculation with rational.



# Command line interface
## Usage
### Install
```console
$ cargo install --git https://github.com/hayas1/calculator
```

### Uninstall
```console
$ cargo uninstall calculator
```

## Example
See `--help` also.
```console
$ calculator --help
Simple program to calculate simple expression

Usage: calculator [OPTIONS] <EXPRESSION>

Arguments:
  <EXPRESSION>  expression to calculate

Options:
  -p, --precisely  use rational
  -h, --help       Print help information
  -V, --version    Print version information
```

```console
$ calculator 1+2
3
```

```console
$ calculator 1/3+1/6 -p
1/2
```

# Library
## Usage
in `Cargo.toml`
```toml
[dependencies]
calculator = { git = "https://github.com/hayas1/calculator" }
```

## Examples
See [test code](/src/lib.rs#124) also.
```rust
let sum: i64 = calculator::calculate("1+2+3+4+5+6+7+8+9+10").unwrap();
println!("{}", sum); // 55
```

For precise, use [rational](https://docs.rs/num/0.4.0/num/rational/type.Rational64.html).
```rust
let half: num::Rational64 = calculator::calculate("1/3+1/6").unwrap();
println!("{}", half); // 1/2
```
