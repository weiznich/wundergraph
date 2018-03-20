use quote;
use syn;
use diagnostic_shim::Diagnostic;
use utils::{inner_ty_arg, is_has_many, is_has_one, is_option_ty, wrap_in_dummy_mod_with_reeport};
use model::Model;

pub fn derive(item: &syn::DeriveInput) -> Result<quote::Tokens, Diagnostic> {
    let item_name = item.ident;
    let filter_name = syn::Ident::from(format!("{}Filter", item_name));
    let model = Model::from_item(item)?;
    let table_ty = model.table_type()?;
    let table = table_ty.to_string();

    let dummy_mod = model.dummy_mod_name("wundergraph_filter");
    let fields = model
        .fields()
        .iter()
        .filter_map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            if f.has_flag("skip") {
                None
            } else if is_has_one(field_ty) {
                let reference_ty = if is_option_ty(
                    inner_ty_arg(field_ty, "HasOne", 0).expect("We checked if this is HasOne"),
                ) {
                    quote!(self::wundergraph::filter::NullableReferenceFilter)
                } else {
                    quote!(self::wundergraph::filter::ReferenceFilter)
                };
                let remote_table = match f.remote_table() {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let remote_filter = f.filter().expect("Filter is missing");
                Some(Ok(quote!{
                    #field_name: Option<#reference_ty<
                    #table_ty::#field_name,
                    #remote_filter,
                    #remote_table::id,
                    >>
                }))
            } else if is_has_many(field_ty) {
                let reference_ty = if f.is_nullable_reference() {
                    quote!(self::wundergraph::filter::ReverseNullableReferenceFilter)
                } else {
                    quote!(self::wundergraph::filter::ReferenceFilter)
                };
                let remote_table = match f.remote_table() {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let foreign_key = match f.foreign_key() {
                    Ok(k) => k,
                    Err(e) => return Some(Err(e)),
                };
                let remote_filter = f.filter().expect("Filter is missing");
                Some(Ok(quote!{
                    #field_name: Option<#reference_ty<
                    #table_ty::id,
                    #remote_filter,
                    #remote_table::#foreign_key,
                    >>
                }))
            } else {
                Some(Ok(quote!{
                    #field_name: Option<self::wundergraph::filter::FilterOption<
                        #field_ty,
                        #table_ty::#field_name,
                    >>
                }))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(wrap_in_dummy_mod_with_reeport(
        dummy_mod,
        &quote! {
            #[derive(Nameable, BuildFilter, InnerFilter, Debug, Clone)]
            #[wundergraph(table_name = #table)]
            pub struct #filter_name {
                #(#fields,)*
            }
        },
        &[quote!(#filter_name)],
    ))
}
