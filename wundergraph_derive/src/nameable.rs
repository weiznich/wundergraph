use quote;
use syn;
use diagnostic_shim::Diagnostic;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<quote::Tokens, Diagnostic> {
    let item_name = item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let dummy_mod = format!("_impl_nameable_for_{}", item.ident.as_ref().to_lowercase());
    Ok(wrap_in_dummy_mod(
        dummy_mod.into(),
        &quote! {
            use self::wundergraph::helper::Nameable;

            impl #impl_generics Nameable for #item_name #ty_generics
                #where_clause
            {
                fn name() -> String {
                    String::from(stringify!(#item_name))
                }
            }
        }
    ))
}
