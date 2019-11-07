use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn simple_query_single_field() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros {
        heroName
    }
}
",
    );
    assert!(res.is_ok());
    assert_json_snapshot!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Luke Skywalker"
      },
      {
        "heroName": "Darth Vader"
      },
      {
        "heroName": "Han Solo"
      },
      {
        "heroName": "Leia Organa"
      },
      {
        "heroName": "Wilhuff Tarkin"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn simple_query_multiple_field() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros {
        id
        heroName
    }
}
",
    );
    assert!(res.is_ok());
    assert_json_snapshot!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Luke Skywalker",
        "id": 1
      },
      {
        "heroName": "Darth Vader",
        "id": 2
      },
      {
        "heroName": "Han Solo",
        "id": 3
      },
      {
        "heroName": "Leia Organa",
        "id": 4
      },
      {
        "heroName": "Wilhuff Tarkin",
        "id": 5
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn simple_query_nested() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros {
        heroName
        home_world {
             name
        }
    }
}
",
    );
    assert!(res.is_ok());
    assert_json_snapshot!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Luke Skywalker",
        "home_world": {
          "name": "Tatooine"
        }
      },
      {
        "heroName": "Darth Vader",
        "home_world": {
          "name": "Tatooine"
        }
      },
      {
        "heroName": "Han Solo",
        "home_world": null
      },
      {
        "heroName": "Leia Organa",
        "home_world": {
          "name": "Alderaan"
        }
      },
      {
        "heroName": "Wilhuff Tarkin",
        "home_world": null
      }
    ]
  },
  []
]"###
    );
}
