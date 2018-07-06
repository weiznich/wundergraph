use diagnostic_shim::Diagnostic;
use proc_macro2::{Span, TokenStream};
use syn;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let item_name = &item.ident;
    let (_, ty_generics, where_clause) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics.params.push(parse_quote!(__C));
    let (impl_generics, _, _) = generics.split_for_impl();

    let dummy_mod = format!(
        "_impl_filter_value_for_{}",
        item.ident.to_string().to_lowercase()
    );
    Ok(wrap_in_dummy_mod(
        &syn::Ident::new(&dummy_mod, Span::call_site()),
        &quote! {
            use self::wundergraph::filter::filter_value::FilterValue;

            impl #impl_generics FilterValue<__C> for #item_name #ty_generics
                #where_clause
            {
                type RawValue = #item_name #ty_generics;
                type AdditionalFilter = ();
            }
        },
    ))
}
