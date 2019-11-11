Wundergraph
==========

Wundergraph provides a platform to easily expose your database through a GraphQL interface.

**This library is currently a prof of concept. Expect bugs and crashes everywhere**

[![Build Status](https://travis-ci.org/weiznich/wundergraph.svg?branch=master)](https://travis-ci.org/weiznich/wundergraph)


## Example
For a full example application see the [example project](https://github.com/weiznich/wundergraph/blob/master/wundergraph_example/src/bin/main.rs)

```rust
#[macro_use] extern crate diesel;
use wundergraph::prelude::*;

table! {
    heros {
        id -> Integer,
        name -> Text,
        hair_color -> Nullable<Text>,
        species -> Integer,
    }
}

table! {
    species {
        id -> Integer,
        name -> Text,
    }
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "heros"]
pub struct Hero {
    id: i32,
    name: String,
    hair_color: Option<String>,
    species: HasOne<i32, Species>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "species"]
pub struct Species {
    id: i32,
    name: String,
    heros: HasMany<Hero, heros::species>,
}

wundergraph::query_object!{
    Query {
       Hero,
       Species,
   }
}
```

## Building

Just install some dependencies (`libsqlite3-dev` and `libpq-dev` in Debian) and `cargo build`.

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

### Contributing

Unless you explicitly state otherwise, any contribution you intentionally submit
for inclusion in the work, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.
