use diagnostic_shim::*;
use proc_macro2::{Span, TokenStream};
use syn;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let e = match item.data {
        syn::Data::Enum(ref e) => e,
        _ => return Err(Span::call_site().error("This derive can only be used on enums")),
    };

    let item_name = &item.ident;
    let field_list = e.variants.iter().map(|f| {
        let name = f.ident.to_string().to_uppercase();
        let variant = &f.ident;
        quote!{
            #name => Some(#item_name::#variant)
        }
    });

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();


    Ok(wrap_in_dummy_mod(
        "from_look_ahead",
        &item.ident,
        &quote! {
            use wundergraph::helper::FromLookAheadValue;
            use wundergraph::juniper::LookAheadValue;

            impl #impl_generics FromLookAheadValue for #item_name #ty_generics
                #where_clause
            {
                fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
                    if let LookAheadValue::Enum(ref e) = *v {
                        match *e {
                            #(#field_list,)*
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
            }
        },
    ))
}
