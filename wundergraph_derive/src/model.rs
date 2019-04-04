use proc_macro2::Span;
use syn;

use crate::diagnostic_shim::*;
use crate::field::*;
use crate::meta::*;

pub struct Model {
    pub name: syn::Ident,
    fields: Vec<Field>,
    flags: MetaItem,
    table_name: Option<syn::Ident>,
    pub docs: Option<String>,
    primary_keys: Vec<syn::Ident>,
}

impl Model {
    pub fn from_item(item: &syn::DeriveInput) -> Result<Self, Diagnostic> {
        let table_name =
            MetaItem::with_name(&item.attrs, "table_name").map(|m| m.expect_ident_value());
        let fields = fields_from_item_data(&item.data)?;
        let flags = MetaItem::with_name(&item.attrs, "wundergraph")
            .unwrap_or_else(|| MetaItem::empty("wundergraph"));
        let docs = MetaItem::get_docs(&item.attrs);
        let primary_keys = MetaItem::with_name(&item.attrs, "primary_key")
            .map(|m| m.nested()?.map(|m| m.word()).collect())
            .unwrap_or_else(|| Ok(vec![syn::Ident::new("id", Span::call_site())]))?;
        Ok(Self {
            name: item.ident.clone(),
            fields,
            flags,
            table_name,
            docs,
            primary_keys,
        })
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn table_type(&self) -> Result<syn::Ident, Diagnostic> {
        self.table_name.clone().map_or_else(
            || {
                self.flags
                    .nested_item("table_name")
                    .and_then(|t| t.ident_value())
            },
            Ok,
        )
    }

    pub fn primary_key(&self) -> &[syn::Ident] {
        &self.primary_keys
    }

    pub fn filter_type(&self) -> Option<syn::Path> {
        self.flags.get_flag("filter").ok()
    }
}

fn fields_from_item_data(data: &syn::Data) -> Result<Vec<Field>, Diagnostic> {
    use syn::Data::*;

    let struct_data = match *data {
        Struct(ref d) => d,
        _ => return Err(Span::call_site().error("This derive can only be used on structs")),
    };
    struct_data
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| Field::from_struct_field(f, i))
        .collect()
}
