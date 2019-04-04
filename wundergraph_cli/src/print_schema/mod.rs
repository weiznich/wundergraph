use crate::infer_schema_internals::*;
use std::error::Error;

mod print_helper;
use self::print_helper::*;

pub fn print(database_url: &str, schema_name: Option<&str>) -> Result<(), Box<dyn Error>> {
    let table_names = load_table_names(database_url, schema_name)?;
    let foreign_keys = load_foreign_key_constraints(database_url, schema_name)?;
    let foreign_keys =
        remove_unsafe_foreign_keys_for_codegen(database_url, &foreign_keys, &table_names);

    let table_data = table_names
        .into_iter()
        .map(|t| load_table_data(database_url, t))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let definitions = TableDefinitions {
        tables: &table_data,
        include_docs: false,
        import_types: None,
    };
    let graphql = GraphqlDefinition {
        tables: &table_data,
        foreign_keys,
    };

    let mutations = GraphqlMutations {
        tables: &table_data,
    };
    println!("use wundergraph::query_helper::{{HasMany, HasOne}};");
    println!("use wundergraph::scalar::WundergraphScalarValue;");
    println!("use wundergraph::WundergraphEntity;");
    println!();
    println!("{}", definitions);
    println!();
    println!("{}", graphql);
    println!();
    println!("{}", mutations);
    Ok(())
}
