use proc_macro2::{Ident, Span};
use syn;

use diagnostic_shim::*;
use field::*;
use meta::*;

pub struct Model {
    pub name: syn::Ident,
    fields: Vec<Field>,
    flags: MetaItem,
    table_name: Option<syn::Ident>,
    pub docs: Option<String>,
}

impl Model {
    pub fn from_item(item: &syn::DeriveInput) -> Result<Self, Diagnostic> {
        let table_name =
            MetaItem::with_name(&item.attrs, "table_name").map(|m| m.expect_ident_value());
        let fields = fields_from_item_data(&item.data)?;
        let flags = MetaItem::with_name(&item.attrs, "wundergraph")
            .unwrap_or_else(|| MetaItem::empty("wundergraph"));
        let docs = MetaItem::get_docs(&item.attrs);
        Ok(Self {
            name: item.ident.clone(),
            fields,
            flags,
            table_name,
            docs,
        })
    }

    pub fn dummy_mod_name(&self, trait_name: &str) -> syn::Ident {
        let name = self.name.to_string().to_lowercase();
        Ident::new(
            &format!("_impl_{}_for_{}", trait_name, name),
            Span::call_site(),
        )
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn table_type(&self) -> Result<syn::Ident, Diagnostic> {
        self.table_name.clone().map(Ok).unwrap_or_else(|| {
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
        let filter_name = format!("{}Filter", self.name);
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

    pub fn context_type(&self, connection: &syn::Type) -> Result<syn::Path, Diagnostic> {
        self.flags
            .get_flag::<syn::Path>("context")
            .map(|mut p| {
                let span = Span::call_site();
                if let Some(mut l) = p.segments.last_mut() {
                    let l = l.value_mut();
                    if let syn::PathArguments::AngleBracketed(ref mut a) = l.arguments {
                        if let Some(arg) = a.args.pop() {
                            if !a.args.is_empty() {
                                return Err(span.error(
                                    "Context type needs exactly one generic argument called Conn",
                                ));
                            }
                            if let syn::GenericArgument::Type(ref t) = *arg.value() {
                                if let syn::Type::Path(ref p) = *t {
                                    if p.path.segments.len() != 1
                                        || p.path.segments[0].ident != "Conn"
                                    {
                                        return Err(span.error(
                                            format!("Expected context time to have generic parameter Conn, but found {}", p.path.segments[0].ident)
                                        ));
                                    }
                                } else {
                                    return Err(span.error("Invalid context type"));
                                }
                            } else {
                                return Err(span.error("Invalid context type"));
                            }
                            a.args.push(syn::GenericArgument::Type(connection.clone()));
                        } else {
                            return Err(span.error(
                                "Context type needs exactly one generic argument called Conn",
                            ));
                        }
                    } else {
                        return Err(span.error(
                            "Context type needs exactly one generic argument called Conn",
                        ));
                    }
                } else {
                    return Err(span.error("Invalid context type"));
                }
                Ok(p)
            })
            .unwrap_or_else(|_| {
                Ok(parse_quote!{
                    self::wundergraph::diesel::r2d2::PooledConnection<
                        self::wundergraph::diesel::r2d2::ConnectionManager<
                        #connection
                        >
                    >
                })
            })
    }

    pub fn query_modifier_type(&self) -> syn::Path {
        self.flags
            .get_flag::<syn::Path>("query_modifier")
            .unwrap_or_else(|_| {
                parse_quote!{
                    self::wundergraph::query_modifier::DefaultModifier<Self::Context, Self>
                }
            })
    }

    pub fn select(&self) -> Vec<syn::Ident> {
        self.flags
            .nested_item("select")
            .ok()
            .and_then(|s| {
                s.nested().ok().map(|m| {
                    m.into_iter()
                        .filter_map(|m| m.word().ok())
                        .collect()
                })
            })
            .unwrap_or_else(Vec::new)
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
