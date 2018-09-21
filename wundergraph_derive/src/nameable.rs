use diagnostic_shim::Diagnostic;
use proc_macro2::TokenStream;
use syn;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let item_name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    Ok(wrap_in_dummy_mod(
        "nameable",
        item_name,
        &quote! {
            use wundergraph::helper::Nameable;

            impl #impl_generics Nameable for #item_name #ty_generics
                #where_clause
            {
                fn name() -> String {
                    String::from(stringify!(#item_name))
                }
            }
        },
    ))
}
