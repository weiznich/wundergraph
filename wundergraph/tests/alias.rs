use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn check_alias() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros {
        name: heroName
        HomeWorld: home_world {
            planet: name
        }
    }
}
",
    );
    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "HomeWorld": {
          "planet": "Tatooine"
        },
        "name": "Luke Skywalker"
      },
      {
        "HomeWorld": {
          "planet": "Tatooine"
        },
        "name": "Darth Vader"
      },
      {
        "HomeWorld": null,
        "name": "Han Solo"
      },
      {
        "HomeWorld": {
          "planet": "Alderaan"
        },
        "name": "Leia Organa"
      },
      {
        "HomeWorld": null,
        "name": "Wilhuff Tarkin"
      }
    ]
  },
  []
]"###
    );
}
