use super::data_structures::ForeignKeyConstraint;
use super::inference::get_primary_keys;
use super::table_data::TableName;
use crate::database::InferConnection;

pub fn remove_unsafe_foreign_keys_for_codegen(
    connection: &InferConnection,
    foreign_keys: &[ForeignKeyConstraint],
    safe_tables: &[TableName],
) -> Vec<ForeignKeyConstraint> {
    let duplicates = foreign_keys
        .iter()
        .map(ForeignKeyConstraint::ordered_tables)
        .filter(|tables| {
            let dup_count = foreign_keys
                .iter()
                .filter(|fk| tables == &fk.ordered_tables())
                .count();
            dup_count > 1
        })
        .collect::<Vec<_>>();

    foreign_keys
        .iter()
        .filter(|fk| fk.parent_table != fk.child_table)
        .filter(|fk| safe_tables.contains(&fk.parent_table))
        .filter(|fk| safe_tables.contains(&fk.child_table))
        .filter(|fk| {
            let pk_columns = get_primary_keys(connection, &fk.parent_table)
                .unwrap_or_else(|_| panic!("Error loading primary keys for `{}`", fk.parent_table));
            pk_columns.len() == 1 && pk_columns[0] == fk.primary_key
        })
        .filter(|fk| !duplicates.contains(&fk.ordered_tables()))
        .cloned()
        .collect()
}
