Wundergraph
==========

Wundergraph provides a platform to easily expose your database through a GraphQL interface.

**This library is currently a prof of concept. Expect bugs and crashes everywhere**


## Example
For a full example application see the [example project](https://github.com/weiznich/wundergraph/tree/master/wundergraph_example/src/main.rs)

```rust

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

#[derive(Clone, Debug, Identifiable, Hash, Eq, PartialEq, Queryable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "heros"]
#[belongs_to(Species, foreign_key = "species)]
pub struct Hero {
    id: i32,
    name: String,
    hair_color: Option<String>,
    species: HasOne<i32, Species>,
}

#[derive(Clone, Debug, Identifiable, Hash, Eq, PartialEq, Queryable, WundergraphEntity,
         WundergraphFilter)]
#[table_name = "species"]
pub struct Species {
    id: i32,
    name: String,
    #[diesel(default)]
    heros: HasMany<Hero>,
}

wundergraph_query_object!{
    Query {
        Heros(Hero, filter = HeroFilter),
        Species(Species, filter = SpeciesFilter),
    }
}

```

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
