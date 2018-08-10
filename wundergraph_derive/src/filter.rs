use diagnostic_shim::Diagnostic;
use model::Model;
use proc_macro2::{Span, TokenStream};
use syn;
use utils::{
    inner_of_option_ty, inner_ty_arg, is_has_many, is_has_one, is_option_ty,
    wrap_in_dummy_mod_with_reeport,
};

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let item_name = &item.ident;
    let filter_name = syn::Ident::new(&format!("{}Filter", item_name), Span::call_site());
    let model = Model::from_item(item)?;
    let table_ty = model.table_type()?;
    let table = table_ty.to_string();

    let dummy_mod = model.dummy_mod_name("wundergraph_filter");
    let fields = model
        .fields()
        .iter()
        .filter_map(|f| {
            let field_name = f.rust_name();
            let sql_name = f.sql_name();
            let field_ty = &f.ty;
            if f.has_flag("skip") {
                None
            } else if is_has_one(field_ty) {
                let reference_ty = if is_option_ty(field_ty) {
                    quote!(self::wundergraph::filter::NullableReferenceFilter)
                } else {
                    quote!(self::wundergraph::filter::ReferenceFilter)
                };
                let remote_table = f.remote_table().map(|t| quote!(#t::table)).unwrap_or_else(
                    |_| {
                        let remote_type = inner_ty_arg(inner_of_option_ty(&f.ty), "HasOne", 1)
                            .expect("It's HasOne");
                        quote!{
                            <#remote_type as diesel::associations::HasTable>::Table
                        }
                    },
                );
                let remote_filter = f.filter().expect("Filter is missing");
                let graphql_name = f.graphql_name().to_string();
                Some(Ok(quote!{
                    #[wundergraph(graphql_name = #graphql_name)]
                    #field_name: Option<#reference_ty<
                    #table_ty::#sql_name,
                    #remote_filter,
                    <#remote_table as diesel::Table>::PrimaryKey,
                    >>
                }))
            } else if is_has_many(field_ty) {
                let reference_ty = if f.is_nullable_reference() {
                    quote!(self::wundergraph::filter::ReverseNullableReferenceFilter)
                } else {
                    quote!(self::wundergraph::filter::ReferenceFilter)
                };
                let remote_table = f.remote_table().map(|t| quote!(#t)).unwrap_or_else(|_| {
                    let remote_type = inner_ty_arg(inner_of_option_ty(&f.ty), "HasMany", 0)
                        .expect("It is HasMany");
                    quote!(<<#remote_type as diesel::associations::BelongsTo<#item_name>>::ForeignKeyColumn as diesel::Column>::Table)
                });
                let foreign_key = f
                    .foreign_key()
                    .map(|k| quote!(#remote_table::#k))
                    .unwrap_or_else(|_| {
                        let remote_type = inner_ty_arg(inner_of_option_ty(&f.ty), "HasMany", 0)
                            .expect("It is HasMany");
                        quote!(<#remote_type as diesel::associations::BelongsTo<#item_name>>::ForeignKeyColumn)
                    });
                let remote_filter = f.filter().expect("Filter is missing");
                let graphql_name = f.graphql_name().to_string();
                Some(Ok(quote!{
                    #[wundergraph(graphql_name = #graphql_name)]
                    #field_name: Option<#reference_ty<
                    <#table_ty::table as diesel::Table>::PrimaryKey,
                    #remote_filter,
                    #foreign_key,
                    >>
                }))
            } else {
                let graphql_name = f.graphql_name().to_string();
                Some(Ok(quote!{
                    #[wundergraph(graphql_name = #graphql_name)]
                    #field_name: Option<self::wundergraph::filter::FilterOption<
                        #field_ty,
                        #table_ty::#sql_name,
                    >>
                }))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(wrap_in_dummy_mod_with_reeport(
        &dummy_mod,
        &quote! {
            use self::wundergraph::diesel;

            #[derive(Nameable, BuildFilter, InnerFilter, Debug, Clone)]
            #[wundergraph(table_name = #table)]
            pub struct #filter_name {
                #(#fields,)*
            }
        },
        &[quote!(#filter_name)],
    ))
}
