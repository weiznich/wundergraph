use crate::infer_schema_internals::*;
use std::fmt::{self, Display, Formatter, Write};

pub struct TableDefinitions<'a> {
    pub tables: &'a [TableData],
    //    fk_constraints: Vec<ForeignKeyConstraint>,
    pub include_docs: bool,
    pub import_types: Option<&'a [String]>,
}

impl<'a> Display for TableDefinitions<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut is_first = true;
        for table in self.tables {
            if is_first {
                is_first = false;
            } else {
                writeln!(f)?;
            }
            writeln!(
                f,
                "{}",
                TableDefinition {
                    table,
                    include_docs: self.include_docs,
                    import_types: self.import_types,
                }
            )?;
        }

        if self.tables.len() > 1 {
            write!(f, "\nallow_tables_to_appear_in_same_query!(")?;
            {
                let mut out = PadAdapter::new(f);
                writeln!(out)?;
                for table in self.tables {
                    writeln!(out, "{},", table.name.name)?;
                }
            }
            writeln!(f, ");")?;
        }

        // for table in self.tables {
        //     if let Some(schema) = &table.name.schema {
        //         writeln!(f, "use self::{}::{};", schema, table.name.name)?;
        //     }
        // }

        Ok(())
    }
}

pub struct TableDefinition<'a> {
    table: &'a TableData,
    import_types: Option<&'a [String]>,
    include_docs: bool,
}

impl<'a> Display for TableDefinition<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "table! {{")?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;

            if let Some(types) = self.import_types {
                for import in types {
                    writeln!(out, "use {};", import)?;
                }
                writeln!(out)?;
            }

            if self.include_docs {
                for d in self.table.docs.lines() {
                    writeln!(out, "///{}{}", if d.is_empty() { "" } else { " " }, d)?;
                }
            }

            write!(out, "{} (", self.table.name)?;
            for (i, pk) in self.table.primary_key.iter().enumerate() {
                if i != 0 {
                    write!(out, ", ")?;
                }
                write!(out, "{}", pk)?;
            }

            write!(
                out,
                ") {}",
                ColumnDefinitions {
                    columns: &self.table.column_data,
                    include_docs: self.include_docs,
                }
            )?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

pub struct ColumnDefinitions<'a> {
    columns: &'a [ColumnDefinition],
    include_docs: bool,
}

impl<'a> Display for ColumnDefinitions<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        {
            let mut out = PadAdapter::new(f);
            writeln!(out, "{{")?;
            for column in self.columns {
                if self.include_docs {
                    for d in column.docs.lines() {
                        writeln!(out, "///{}{}", if d.is_empty() { "" } else { " " }, d)?;
                    }
                }
                if let Some(ref rust_name) = column.rust_name {
                    writeln!(out, r#"#[sql_name = "{}"]"#, column.sql_name)?;
                    writeln!(out, "{} -> {},", rust_name, column.ty)?;
                } else {
                    writeln!(out, "{} -> {},", column.sql_name, column.ty)?;
                }
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// Lifted directly from libcore/fmt/builders.rs
pub struct PadAdapter<'a, W> {
    fmt: &'a mut W,
    on_newline: bool,
}

impl<'a, W: 'a> PadAdapter<'a, W> {
    fn new(fmt: &'a mut W) -> PadAdapter<'a, W> {
        PadAdapter {
            fmt,
            on_newline: false,
        }
    }
}

impl<'a, W> Write for PadAdapter<'a, W>
where
    W: Write + 'a,
{
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            let on_newline = self.on_newline;

            let split = if let Some(pos) = s.find('\n') {
                self.on_newline = true;
                pos + 1
            } else {
                self.on_newline = false;
                s.len()
            };
            let to_write = &s[..split];
            if on_newline && to_write != "\n" {
                self.fmt.write_str("    ")?;
            }
            self.fmt.write_str(to_write)?;

            s = &s[split..];
        }

        Ok(())
    }
}

pub struct GraphqlDefinition<'a> {
    pub tables: &'a [TableData],
    pub foreign_keys: Vec<ForeignKeyConstraint>,
}

impl<'a> Display for GraphqlDefinition<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for t in self.tables {
            writeln!(
                f,
                "{}",
                GraphqlData {
                    table: t,
                    foreign_keys: &self.foreign_keys,
                }
            )?;
        }
        writeln!(f)?;
        writeln!(f)?;
        write!(f, "wundergraph::query_object!{{")?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            write!(out, "Query {{")?;
            {
                let mut out = PadAdapter::new(&mut out);
                writeln!(out)?;
                for t in self.tables {
                    let single = fix_table_name(&t.name.name);
                    writeln!(out, "{},", single)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

struct GraphqlData<'a> {
    table: &'a TableData,
    foreign_keys: &'a [ForeignKeyConstraint],
}

fn uppercase_table_name(name: &str) -> String {
    let mut next_uppercase = true;
    name.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c == '_' {
                next_uppercase = true;
                None
            } else if next_uppercase {
                next_uppercase = false;
                Some(c.to_uppercase().to_string())
            } else {
                Some(c.to_string())
            }
        })
        .fold(String::new(), |acc, s| acc + &s)
}

fn fix_table_name(name: &str) -> String {
    let mut name = uppercase_table_name(name);
    if name.ends_with('s') {
        name.pop();
        name
    } else {
        name
    }
}

fn write_primary_key_section<W>(f: &mut W, table: &TableData) -> fmt::Result
where
    W: Write,
{
    write!(f, "#[primary_key(")?;
    let mut first = true;
    for k in &table.primary_key {
        if first {
            first = false;
        } else {
            write!(f, ", ")?;
        }
        write!(f, "{}", k)?;
    }
    writeln!(f, ")]")?;
    Ok(())
}

impl<'a> Display for GraphqlData<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "#[derive(Clone, Debug, Identifiable, WundergraphEntity)]"
        )?;
        writeln!(f, "#[table_name = \"{}\"]", self.table.name.name)?;
        write_primary_key_section(f, self.table)?;
        write!(f, "pub struct {} {{", fix_table_name(&self.table.name.name))?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            for c in &self.table.column_data {
                writeln!(
                    out,
                    "{}",
                    GraphqlColumn {
                        column: c,
                        foreign_key: self.foreign_keys.iter().find(|f| f.child_table
                            == self.table.name
                            && f.foreign_key == c.sql_name),
                    }
                )?;
            }
            for f in self
                .foreign_keys
                .iter()
                .filter(|f| f.parent_table == self.table.name)
            {
                writeln!(
                    out,
                    "{}: HasMany<{}, {}::{}>,",
                    f.child_table.name,
                    fix_table_name(&f.child_table.name),
                    f.child_table.name,
                    f.foreign_key,
                )?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

struct GraphqlColumn<'a> {
    column: &'a ColumnDefinition,
    foreign_key: Option<&'a ForeignKeyConstraint>,
}

impl<'a> Display for GraphqlColumn<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut tpe = GraphqlType {
            sql_type: &self.column.ty,
            allow_option: true,
        };
        let name = self
            .column
            .rust_name
            .as_ref()
            .unwrap_or(&self.column.sql_name);
        if let Some(foreign_key) = self.foreign_key {
            tpe.allow_option = false;
            let referenced = fix_table_name(&foreign_key.parent_table.name);
            if self.column.ty.is_nullable {
                write!(f, "{}: Option<HasOne<{}, {}>>,", name, tpe, referenced)?;
            } else {
                write!(f, "{}: HasOne<{}, {}>,", name, tpe, referenced)?;
            }
        } else {
            write!(f, "{}: {},", name, tpe)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::needless_borrow)]
struct GraphqlType<'a> {
    sql_type: &'a ColumnType,
    allow_option: bool,
}

impl<'a> Display for GraphqlType<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self.sql_type {
            ColumnType {
                is_nullable: true, ..
            } if self.allow_option => {
                let mut t = self.clone();
                t.allow_option = false;
                write!(f, "Option<{}>", t)?;
            }
            ColumnType {
                is_array: true,
                is_nullable,
                ref rust_name,
                is_unsigned,
            } => {
                let t = ColumnType {
                    is_array: false,
                    is_nullable,
                    rust_name: rust_name.clone(),
                    is_unsigned,
                };
                write!(
                    f,
                    "Vec<{}>",
                    GraphqlType {
                        sql_type: &t,
                        ..self.clone()
                    }
                )?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Int2" || rust_name == "SmallInt" => {
                write!(f, "i16")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Int4" || rust_name == "Integer" => {
                write!(f, "i32")?;
            }

            ColumnType { ref rust_name, .. } if rust_name == "Int8" || rust_name == "BigInt" => {
                write!(f, "i64")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Float" || rust_name == "Float4" => {
                write!(f, "f32")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Double" || rust_name == "Float8" => {
                write!(f, "f64")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Text" || rust_name == "Varchar" => {
                write!(f, "String")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Bool" => {
                write!(f, "bool")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Timestamptz" => {
                write!(f, "chrono::DateTime<chrono::offset::Utc>")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Timestamp" => {
                write!(f, "chrono::naive::NaiveDateTime")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Uuid" => {
                write!(f, "uuid::Uuid")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Numeric" => {
                write!(f, "bigdecimal::BigDecimal")?;
            }
            ColumnType { ref rust_name, .. } => write!(f, "{}", fix_table_name(rust_name))?,
        }
        Ok(())
    }
}

pub struct GraphqlMutations<'a> {
    pub tables: &'a [TableData],
}

impl<'a> Display for GraphqlMutations<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for t in self.tables {
            writeln!(f, "{}", GraphqlInsertable { table: t })?;
            writeln!(f, "{}", GraphqlChangeSet { table: t })?;
        }

        write!(f, "wundergraph::mutation_object!{{")?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            write!(out, "Mutation{{")?;
            {
                let mut out = PadAdapter::new(&mut out);
                writeln!(out)?;
                for t in self.tables {
                    if t.primary_key.len() == t.column_data.len() {
                        writeln!(out, "{}(),", fix_table_name(&t.name.name))?;
                    //writeln!(out, "{name}")
                    } else {
                        writeln!(
                            out,
                            "{name}(insert = New{name}, update = {name}Changeset, ),",
                            name = fix_table_name(&t.name.name)
                        )?;
                    }
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

struct GraphqlInsertable<'a> {
    table: &'a TableData,
}

impl<'a> Display for GraphqlInsertable<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.table.primary_key.len() == self.table.column_data.len() {
            return Ok(());
        }
        writeln!(
            f,
            "#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]"
        )?;
        writeln!(f, "#[graphql(scalar = \"WundergraphScalarValue\")]")?;
        writeln!(f, "#[table_name = \"{}\"]", self.table.name.name)?;
        write!(
            f,
            "pub struct New{} {{",
            fix_table_name(&self.table.name.name)
        )?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            for c in self.table.column_data.iter().filter(|c| !c.has_default) {
                let t = GraphqlType {
                    sql_type: &c.ty,
                    allow_option: true,
                };
                let name = c.rust_name.as_ref().unwrap_or(&c.sql_name);
                writeln!(out, "{}: {},", name, t)?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

struct GraphqlChangeSet<'a> {
    table: &'a TableData,
}

impl<'a> Display for GraphqlChangeSet<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.table.primary_key.len() == self.table.column_data.len() {
            return Ok(());
        }
        writeln!(
            f,
            "#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]"
        )?;
        writeln!(f, "#[graphql(scalar = \"WundergraphScalarValue\")]")?;
        writeln!(f, "#[table_name = \"{}\"]", self.table.name.name)?;
        write_primary_key_section(f, self.table)?;
        write!(
            f,
            "pub struct {}Changeset {{",
            fix_table_name(&self.table.name.name)
        )?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            for c in &self.table.column_data {
                let t = GraphqlType {
                    sql_type: &c.ty,
                    allow_option: true,
                };
                let name = c.rust_name.as_ref().unwrap_or(&c.sql_name);
                writeln!(out, "{}: {},", name, t)?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
