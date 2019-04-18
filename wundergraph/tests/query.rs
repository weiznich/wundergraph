use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn query_filter_eq() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(filter: {id: {eq: 1}}) {
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
        "heroName": "Luke Skywalker"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_not_eq() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(filter: {id: {not_eq: 1}}) {
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
fn query_filter_like() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {heroName: {like: "Leia %"}}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Leia Organa"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_cannot_use_like_with_non_strings() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {heroName: {id: "Leia %"}}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_err());
}

#[test]
fn query_filter_eq_any() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(filter: {id: {eq_any: [1, 4]}}) {
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
        "heroName": "Luke Skywalker"
      },
      {
        "heroName": "Leia Organa"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_and() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {and: [{heroName: {like: "Leia %"}}, {id: {eq: 4}}]}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Leia Organa"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_or() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {or: [{heroName: {like: "Leia %"}}, {id: {eq: 1}}]}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Luke Skywalker"
      },
      {
        "heroName": "Leia Organa"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_is_null() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {hair_color: {is_null: true}}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
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
        "heroName": "Wilhuff Tarkin"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_is_not_null() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {hair_color: {is_null: false}}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Luke Skywalker"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_is_null_cannot_be_used_with_not_nullable_fields() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {id: {is_null: true}}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_err());
}

#[test]
fn query_filter_type_could_have_2_modifiers() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
    Heros(filter: {and: [{hair_color: {is_null: false}}, {hair_color: {like: "%"}}]}) {
        heroName
    }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "heroName": "Luke Skywalker"
      }
    ]
  },
  []
]"###
    );
}

#[test]
fn query_filter_not() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(filter: {not: {id: {eq: 1}}}) {
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
fn filter_on_field() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
  Speciess {
    id
    name
    heros(filter: {heroName: {like: "Luke%"}}) {
      heroName
    }
  }
}
"#,
    );
    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Speciess": [
      {
        "heros": [
          {
            "heroName": "Luke Skywalker"
          }
        ],
        "id": 1,
        "name": "Human"
      },
      {
        "heros": [],
        "id": 2,
        "name": "Robot"
      }
    ]
  },
  []
]"###
    );
}
