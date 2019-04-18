use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn delete_existing() {
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
    assert_json_snapshot_matches!(
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

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation DeleteHero {
  DeleteHero(DeleteHero: {id: 5}) {
    count
  }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "DeleteHero": {
      "count": 1
    }
  },
  []
]"###
    );

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
    assert_json_snapshot_matches!(
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
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn delete_non_existing() {
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
    assert_json_snapshot_matches!(
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

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation DeleteHero {
  DeleteHero(DeleteHero: {id: 42}) {
    count
  }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "DeleteHero": {
      "count": 0
    }
  },
  []
]"###
    );

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
    assert_json_snapshot_matches!(
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
