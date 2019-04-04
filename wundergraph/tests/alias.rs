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
    println!("{:?}", res);
    assert!(res.is_ok());
    assert_eq!(json!([{"Heros": [
      {
        "name": "Luke Skywalker",
        "HomeWorld": {
          "planet": "Tatooine"
        }
      },
      {
        "name": "Darth Vader",
        "HomeWorld": {
          "planet": "Tatooine"
        }
      },
      {
        "name": "Han Solo",
        "HomeWorld": null
      },
      {
        "name": "Leia Organa",
        "HomeWorld": {
          "planet": "Alderaan"
        }
      },
      {
        "name": "Wilhuff Tarkin",
        "HomeWorld": null
      }
    ]}, []]), res.as_json());
}
