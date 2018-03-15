use quote;
use syn;
use diagnostic_shim::Diagnostic;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<quote::Tokens, Diagnostic> {
    let item_name = item.ident;
    let (_, ty_generics, where_clause) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics.params.push(parse_quote!(__C));
    generics.params.push(parse_quote!(__DB));
    let (impl_generics, _, _) = generics.split_for_impl();

    let dummy_mod = format!(
        "_impl_filter_value_for_{}",
        item.ident.as_ref().to_lowercase()
    );
    Ok(wrap_in_dummy_mod(
        dummy_mod.into(),
        &quote! {
            use self::wundergraph::filter::filter_value::FilterValue;

            impl #impl_generics FilterValue<__C, __DB> for #item_name #ty_generics
                #where_clause
            {
                type RawValue = #item_name #ty_generics;
                type AdditionalFilter = ();
            }
        },
    ))
}
