use diagnostic_shim::Diagnostic;
use proc_macro2::{Span, TokenStream};
use syn;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let item_name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let dummy_mod = format!(
        "_impl_nameable_for_{}",
        item.ident.to_string().to_lowercase()
    );
    Ok(wrap_in_dummy_mod(
        &syn::Ident::new(&dummy_mod, Span::call_site()),
        &quote! {
            use self::wundergraph::helper::Nameable;

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
