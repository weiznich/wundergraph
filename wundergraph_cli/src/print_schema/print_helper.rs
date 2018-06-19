use infer_schema_internals::*;
use std::fmt::{self, Display, Formatter, Write};

pub struct TableDefinitions<'a> {
    pub tables: &'a [TableData],
    //    fk_constraints: Vec<ForeignKeyConstraint>,
    pub include_docs: bool,
    pub import_types: Option<&'a [String]>,
}

impl<'a> Display for TableDefinitions<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

        // if !self.fk_constraints.is_empty() {
        //     writeln!(f)?;
        // }

        // for foreign_key in &self.fk_constraints {
        //     writeln!(f, "{}", Joinable(foreign_key))?;
        // }

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

        Ok(())
    }
}

pub struct TableDefinition<'a> {
    table: &'a TableData,
    import_types: Option<&'a [String]>,
    include_docs: bool,
}

impl<'a> Display for TableDefinition<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
pub struct PadAdapter<'a, W: 'a> {
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

            let split = match s.find('\n') {
                Some(pos) => {
                    self.on_newline = true;
                    pos + 1
                }
                None => {
                    self.on_newline = false;
                    s.len()
                }
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for t in self.tables {
            writeln!(
                f,
                "{}",
                GraphqlData {
                    table: t,
                    foreign_keys: &self.foreign_keys
                }
            )?;
        }
        writeln!(f)?;
        writeln!(f)?;
        write!(f, "wundergraph_query_object!{{")?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            write!(out, "Query {{")?;
            {
                let mut out = PadAdapter::new(&mut out);
                writeln!(out)?;
                for t in self.tables {
                    let uppercase = uppercase_table_name(&t.name.name);
                    let single = fix_table_name(&t.name.name);
                    writeln!(
                        out,
                        "{upper}({single}, filter = {single}Filter),",
                        upper = uppercase,
                        single = single
                    )?;
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
    if name.ends_with("s") {
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity, WundergraphFilter, Associations)]")?;
        writeln!(f, "#[table_name = \"{}\"]", self.table.name.name)?;
        write_primary_key_section(f, self.table)?;
        for key in self.foreign_keys
            .iter()
            .filter(|f| f.child_table == self.table.name)
        {
            writeln!(
                f,
                "#[belongs_to({parent}, foreign_key = \"{foreign_key}\")]",
                parent = fix_table_name(&key.parent_table.name),
                foreign_key = key.foreign_key
            )?
        }
        write!(f, "struct {} {{", fix_table_name(&self.table.name.name))?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            for c in &self.table.column_data {
                writeln!(
                    out,
                    "{}",
                    GraphqlColumn {
                        column: c,
                        foreign_key: self.foreign_keys
                            .iter()
                            .filter(|f| f.child_table == self.table.name && f.foreign_key == c.sql_name)
                            .next(),
                    }
                )?;
            }
            for f in self.foreign_keys
                .iter()
                .filter(|f| f.parent_table == self.table.name)
            {
                writeln!(out, "#[diesel(default)]")?;
                writeln!(
                    out,
                    "{}: HasMany<{}>,",
                    f.child_table.name,
                    fix_table_name(&f.child_table.name)
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let tpe = GraphqlType {
            sql_type: &self.column.ty,
        };
        if let Some(ref rust_name) = self.column.rust_name {
            // TODO: implement this
            unimplemented!()
        } else {
            if let Some(foreign_key) = self.foreign_key {
                let referenced = if self.column.ty.is_nullable {
                    format!("Option<{}>", fix_table_name(&foreign_key.parent_table.name))
                } else {
                    fix_table_name(&foreign_key.parent_table.name)
                };

                write!(
                    f,
                    "{}: HasOne<{}, {}>,",
                    self.column.sql_name, tpe, referenced
                )?;
            } else {
                write!(f, "{}: {},", self.column.sql_name, tpe)?;
            }
        }
        Ok(())
    }
}

struct GraphqlType<'a> {
    sql_type: &'a ColumnType,
}

impl<'a> Display for GraphqlType<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self.sql_type {
            ColumnType {
                is_nullable: true,
                ref rust_name,
                is_array,
                is_unsigned,
            } => {
                let t = ColumnType {
                    is_nullable: false,
                    rust_name: rust_name.clone(),
                    is_array,
                    is_unsigned,
                };
                write!(f, "Option<{}>", GraphqlType { sql_type: &t })?;
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
                write!(f, "Vec<{}>", GraphqlType { sql_type: &t })?;
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
                write!(f, "DateTime<Utc>")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Timestamp" => {
                write!(f, "NaiveDateTime")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Uuid" => {
                write!(f, "Uuid")?;
            }
            ColumnType { ref rust_name, .. } if rust_name == "Numeric" => {
                write!(f, "BigDecimal")?;
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for t in self.tables {
            writeln!(f, "{}", GraphqlInsertable { table: t })?;
            writeln!(f, "{}", GraphqlChangeSet { table: t })?;
        }

        write!(f, "wundergraph_mutation_object!{{")?;
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.table.primary_key.len() == self.table.column_data.len() {
            return Ok(());
        }
        writeln!(f, "#[derive(Insertable, GraphQLInputObject, Clone, Debug)]")?;
        writeln!(f, "#[table_name = \"{}\"]", self.table.name)?;
        write!(f, "struct New{} {{", fix_table_name(&self.table.name.name))?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            // TODO: we need some information about autoincrementing here
            for c in self.table
                .column_data
                .iter()
                .filter(|c| !self.table.primary_key.contains(&c.sql_name))
            {
                let t = GraphqlType { sql_type: &c.ty };
                if let Some(rust_name) = c.rust_name.as_ref() {
                    // TODO: sql name annotation
                } else {
                    writeln!(out, "{}: {},", c.sql_name, t)?;
                }
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.table.primary_key.len() == self.table.column_data.len() {
            return Ok(());
        }
        writeln!(
            f,
            "#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]"
        )?;
        writeln!(f, "#[table_name = \"{}\"]", self.table.name)?;
        write_primary_key_section(f, self.table)?;
        write!(
            f,
            "struct {}Changeset {{",
            fix_table_name(&self.table.name.name)
        )?;
        {
            let mut out = PadAdapter::new(f);
            writeln!(out)?;
            for c in &self.table.column_data {
                let t = GraphqlType { sql_type: &c.ty };
                if let Some(rust_name) = c.rust_name.as_ref() {
                    // TODO: sql name annotation
                } else {
                    writeln!(out, "{}: {},", c.sql_name, t)?;
                }
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
