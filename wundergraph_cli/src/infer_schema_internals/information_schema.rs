use std::error::Error;

use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::expression::NonAggregate;
#[cfg(feature = "mysql")]
use diesel::mysql::Mysql;
#[cfg(feature = "postgres")]
use diesel::pg::Pg;
use diesel::query_builder::{QueryFragment, QueryId};
use diesel::*;

use super::data_structures::*;
use super::table_data::TableName;

pub trait UsesInformationSchema: Backend {
    type TypeColumn: SelectableExpression<
            self::information_schema::columns::table,
            SqlType = sql_types::Text,
        > + NonAggregate
        + QueryId
        + QueryFragment<Self>;

    fn type_column() -> Self::TypeColumn;
    fn default_schema<C>(conn: &C) -> QueryResult<String>
    where
        C: Connection,
        String: FromSql<sql_types::Text, C::Backend>;
}

#[cfg(feature = "postgres")]
impl UsesInformationSchema for Pg {
    type TypeColumn = self::information_schema::columns::udt_name;

    fn type_column() -> Self::TypeColumn {
        self::information_schema::columns::udt_name
    }

    fn default_schema<C>(_conn: &C) -> QueryResult<String> {
        Ok("public".into())
    }
}

#[cfg(feature = "mysql")]
impl UsesInformationSchema for Mysql {
    type TypeColumn = self::information_schema::columns::column_type;

    fn type_column() -> Self::TypeColumn {
        self::information_schema::columns::column_type
    }

    fn default_schema<C>(conn: &C) -> QueryResult<String>
    where
        C: Connection,
        String: FromSql<sql_types::Text, C::Backend>,
    {
        no_arg_sql_function!(database, sql_types::VarChar);
        select(database).get_result(conn)
    }
}

#[allow(clippy::module_inception)]
mod information_schema {
    table! {
        information_schema.tables (table_schema, table_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            table_type -> VarChar,
        }
    }

    table! {
        information_schema.columns (table_schema, table_name, column_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            column_name -> VarChar,
            is_nullable -> VarChar,
            ordinal_position -> BigInt,
            udt_name -> VarChar,
            column_type -> VarChar,
            column_default -> Nullable<VarChar>,
        }
    }

    table! {
        information_schema.key_column_usage (table_schema, table_name, column_name, constraint_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            column_name -> VarChar,
            constraint_schema -> VarChar,
            constraint_name -> VarChar,
            ordinal_position -> BigInt,
        }
    }

    table! {
        information_schema.table_constraints (table_schema, table_name, constraint_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            constraint_schema -> VarChar,
            constraint_name -> VarChar,
            constraint_type -> VarChar,
        }
    }

    table! {
        information_schema.referential_constraints (constraint_schema, constraint_name) {
            constraint_schema -> VarChar,
            constraint_name -> VarChar,
            unique_constraint_schema -> VarChar,
            unique_constraint_name -> VarChar,
        }
    }

    allow_tables_to_appear_in_same_query!(table_constraints, referential_constraints);
    allow_tables_to_appear_in_same_query!(key_column_usage, table_constraints);
}

pub fn get_table_data<Conn>(conn: &Conn, table: &TableName) -> QueryResult<Vec<ColumnInformation>>
where
    Conn: Connection,
    Conn::Backend: UsesInformationSchema,
    String: FromSql<sql_types::Text, Conn::Backend>,
{
    use self::information_schema::columns::dsl::*;

    let schema_name = match table.schema {
        Some(ref name) => name.clone(),
        None => Conn::Backend::default_schema(conn)?,
    };

    let type_column = Conn::Backend::type_column();
    columns
        .select((column_name, type_column, is_nullable, column_default))
        .filter(table_name.eq(&table.name))
        .filter(table_schema.eq(schema_name))
        .order(ordinal_position)
        .load(conn)
}

pub fn get_primary_keys<Conn>(conn: &Conn, table: &TableName) -> QueryResult<Vec<String>>
where
    Conn: Connection,
    Conn::Backend: UsesInformationSchema,
    String: FromSql<sql_types::Text, Conn::Backend>,
{
    use self::information_schema::key_column_usage::dsl::*;
    use self::information_schema::table_constraints::{self, constraint_type};

    let pk_query = table_constraints::table
        .select(table_constraints::constraint_name)
        .filter(constraint_type.eq("PRIMARY KEY"));

    let schema_name = match table.schema {
        Some(ref name) => name.clone(),
        None => Conn::Backend::default_schema(conn)?,
    };

    key_column_usage
        .select(column_name)
        .filter(constraint_name.eq_any(pk_query))
        .filter(table_name.eq(&table.name))
        .filter(table_schema.eq(schema_name))
        .order(ordinal_position)
        .load(conn)
}

pub fn load_table_names<Conn>(
    connection: &Conn,
    schema_name: Option<&str>,
) -> Result<Vec<TableName>, Box<dyn Error>>
where
    Conn: Connection,
    Conn::Backend: UsesInformationSchema,
    String: FromSql<sql_types::Text, Conn::Backend>,
{
    use self::information_schema::tables::dsl::*;

    let default_schema = Conn::Backend::default_schema(connection)?;
    let schema_name = match schema_name {
        Some(name) => name,
        None => &default_schema,
    };

    let mut table_names = tables
        .select((table_name, table_schema))
        .filter(table_schema.eq(schema_name))
        .filter(table_name.not_like("\\_\\_%"))
        .filter(table_type.like("BASE TABLE"))
        .order(table_name)
        .load::<TableName>(connection)?;
    for table in &mut table_names {
        table.strip_schema_if_matches(&default_schema);
    }
    Ok(table_names)
}

#[allow(clippy::similar_names)]
#[cfg(feature = "postgres")]
pub fn load_foreign_key_constraints<Conn>(
    connection: &Conn,
    schema_name: Option<&str>,
) -> QueryResult<Vec<ForeignKeyConstraint>>
where
    Conn: Connection,
    Conn::Backend: UsesInformationSchema,
    String: FromSql<sql_types::Text, Conn::Backend>,
{
    use self::information_schema::key_column_usage as kcu;
    use self::information_schema::referential_constraints as rc;
    use self::information_schema::table_constraints as tc;

    let default_schema = Conn::Backend::default_schema(connection)?;
    let schema_name = match schema_name {
        Some(name) => name,
        None => &default_schema,
    };

    let constraint_names = tc::table
        .filter(tc::constraint_type.eq("FOREIGN KEY"))
        .filter(tc::table_schema.eq(schema_name))
        .inner_join(
            rc::table.on(tc::constraint_schema
                .eq(rc::constraint_schema)
                .and(tc::constraint_name.eq(rc::constraint_name))),
        )
        .select((
            rc::constraint_schema,
            rc::constraint_name,
            rc::unique_constraint_schema,
            rc::unique_constraint_name,
        ))
        .load::<(String, String, String, String)>(connection)?;

    constraint_names
        .into_iter()
        .map(
            |(foreign_key_schema, foreign_key_name, primary_key_schema, primary_key_name)| {
                let (mut foreign_key_table, foreign_key_column) = kcu::table
                    .filter(kcu::constraint_schema.eq(&foreign_key_schema))
                    .filter(kcu::constraint_name.eq(&foreign_key_name))
                    .select(((kcu::table_name, kcu::table_schema), kcu::column_name))
                    .first::<(TableName, _)>(connection)?;
                let (mut primary_key_table, primary_key_column) = kcu::table
                    .filter(kcu::constraint_schema.eq(primary_key_schema))
                    .filter(kcu::constraint_name.eq(primary_key_name))
                    .select(((kcu::table_name, kcu::table_schema), kcu::column_name))
                    .first::<(TableName, _)>(connection)?;

                foreign_key_table.strip_schema_if_matches(&default_schema);
                primary_key_table.strip_schema_if_matches(&default_schema);

                Ok(ForeignKeyConstraint {
                    child_table: foreign_key_table,
                    parent_table: primary_key_table,
                    foreign_key: foreign_key_column,
                    primary_key: primary_key_column,
                })
            },
        )
        .collect()
}
