use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn order_asc() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(order: [{column: heroName, direction: ASC}]) {
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
        "heroName": "Darth Vader"
      },
      {
        "heroName": "Han Solo"
      },
      {
        "heroName": "Leia Organa"
      },
      {
        "heroName": "Luke Skywalker"
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
fn order_desc() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(order: [{column: heroName, direction: DESC}]) {
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
        "heroName": "Wilhuff Tarkin"
      },
      {
        "heroName": "Luke Skywalker"
      },
      {
        "heroName": "Leia Organa"
      },
      {
        "heroName": "Han Solo"
      },
      {
        "heroName": "Darth Vader"
      }
    ]
  },
  []
]"###
    );
}
