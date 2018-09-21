use diagnostic_shim::Diagnostic;
use proc_macro2::TokenStream;
use syn;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let item_name = &item.ident;
    let (_, ty_generics, where_clause) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics.params.push(parse_quote!(__C));
    let (impl_generics, _, _) = generics.split_for_impl();

    Ok(wrap_in_dummy_mod(
        "filter_value_for",
        &item.ident,
        &quote! {
            use wundergraph::filter::filter_value::FilterValue;

            impl #impl_generics FilterValue<__C> for #item_name #ty_generics
                #where_clause
            {
                type RawValue = #item_name #ty_generics;
                type AdditionalFilter = ();
            }
        },
    ))
}
