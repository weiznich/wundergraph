use syn;
use proc_macro2::Span;

use diagnostic_shim::*;
use field::*;
use meta::*;

pub struct Model {
    pub name: syn::Ident,
    fields: Vec<Field>,
    flags: MetaItem,
    table_name: Option<syn::Ident>,
}

impl Model {
    pub fn from_item(item: &syn::DeriveInput) -> Result<Self, Diagnostic> {
        let table_name =
            MetaItem::with_name(&item.attrs, "table_name").map(|m| m.expect_ident_value());
        let fields = fields_from_item_data(&item.data)?;
        let flags = MetaItem::with_name(&item.attrs, "wundergraph")
            .unwrap_or_else(|| MetaItem::empty("wundergraph"));
        Ok(Self {
            name: item.ident,
            fields,
            flags,
            table_name,
        })
    }

    pub fn dummy_mod_name(&self, trait_name: &str) -> syn::Ident {
        let name = self.name.as_ref().to_lowercase();
        format!("_impl_{}_for_{}", trait_name, name).into()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn table_type(&self) -> Result<syn::Ident, Diagnostic> {
        self.table_name.map(Ok).unwrap_or_else(|| {
            self.flags
                .nested_item("table_name")
                .and_then(|t| t.ident_value())
        })
    }

    pub fn should_have_limit(&self) -> bool {
        self.flags
            .nested_item("limit")
            .and_then(|m| m.bool_value())
            .unwrap_or(true)
    }

    pub fn should_have_offset(&self) -> bool {
        self.flags
            .nested_item("offset")
            .and_then(|m| m.bool_value())
            .unwrap_or(true)
    }

    pub fn should_have_order(&self) -> bool {
        self.flags
            .nested_item("order")
            .and_then(|m| m.bool_value())
            .unwrap_or(true)
    }

    pub fn filter_type(&self) -> Option<syn::Path> {
        let filter_name = format!("{}Filter<DB>", self.name);
        if let Ok(filter) = self.flags.nested_item("filter") {
            match filter.bool_value() {
                Ok(true) => syn::parse_str(&filter_name).ok(),
                Ok(false) => return None,
                Err(_) => self.flags.get_flag("filter").ok(),
            }
        } else {
            syn::parse_str(&filter_name).ok()
        }
    }
}

fn fields_from_item_data(data: &syn::Data) -> Result<Vec<Field>, Diagnostic> {
    use syn::Data::*;

    let struct_data = match *data {
        Struct(ref d) => d,
        _ => return Err(Span::call_site().error("This derive can only be used on structs")),
    };
    Ok(struct_data
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| Field::from_struct_field(f, i))
        .collect())
}
