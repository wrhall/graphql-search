# Getting started

If you have cargo installed (`apt-get install cargo`), then you can simply run:

```sh
cargo build --release
```

From there, navigate to the directory you want to recursively search, and then run something like this:

```sh
../graphql-search/target/release/graphql-search mytoplevelquery.subfield.subsubfield

../graphql-search/target/release/graphql-search mytoplevelquery.subfield.subsubfield --verbose
```

It currently doesn't do anything particularly intelligent, like combining fragments into the main query.
