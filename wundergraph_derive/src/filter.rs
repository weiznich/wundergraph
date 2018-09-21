use diagnostic_shim::Diagnostic;
use field::Field;
use model::Model;
use proc_macro2::{Ident, Span, TokenStream};
use syn;
use utils::{inner_of_option_ty, inner_ty_arg, is_has_many, is_has_one, is_option_ty};

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let item_name = &item.ident;
    let filter_name = syn::Ident::new(&format!("{}Filter", item_name), Span::call_site());
    let model = Model::from_item(item)?;
    let table_ty = model.table_type()?;
    let table = table_ty.to_string();

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
                handle_has_one(f, &table_ty)
            } else if is_has_many(field_ty) {
                handle_has_many(f, item_name, &table_ty)
            } else {
                let graphql_name = f.graphql_name().to_string();
                Some(Ok(quote!{
                    #[wundergraph(graphql_name = #graphql_name)]
                    #field_name: ::std::option::Option<::wundergraph::filter::FilterOption<
                        #field_ty,
                        #table_ty::#sql_name,
                    >>
                }))
            }
        }).collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #[derive(Nameable, Debug, Clone, BuildFilter, InnerFilter)]
        #[wundergraph(table_name = #table)]
        pub struct #filter_name {
            #(#fields,)*
        }
    })
}

fn handle_has_many(
    f: &Field,
    item_name: &Ident,
    table_ty: &Ident,
) -> Option<Result<TokenStream, Diagnostic>> {
    let field_name = f.rust_name();
    let reference_ty = if f.is_nullable_reference() {
        quote!(::wundergraph::filter::ReverseNullableReferenceFilter)
    } else {
        quote!(::wundergraph::filter::ReferenceFilter)
    };
    let remote_table = f.remote_table().map(|t| quote!(#t)).unwrap_or_else(|_| {
                    let remote_type = inner_ty_arg(inner_of_option_ty(&f.ty), "HasMany", 0)
                        .expect("It is HasMany");
                    quote!(<<#remote_type as ::wundergraph::diesel::associations::BelongsTo<#item_name>>::ForeignKeyColumn as ::wundergraph::diesel::Column>::Table)
                });
    let foreign_key = f
                    .foreign_key()
                    .map(|k| quote!(#remote_table::#k))
                    .unwrap_or_else(|_| {
                        let remote_type = inner_ty_arg(inner_of_option_ty(&f.ty), "HasMany", 0)
                            .expect("It is HasMany");
                        quote!(<#remote_type as ::wundergraph::diesel::associations::BelongsTo<#item_name>>::ForeignKeyColumn)
                    });
    let remote_filter = f.filter().expect("Filter is missing");
    let graphql_name = f.graphql_name().to_string();
    Some(Ok(quote!{
        #[wundergraph(graphql_name = #graphql_name)]
        #field_name: ::std::option::Option<#reference_ty<
        <#table_ty::table as ::wundergraph::diesel::Table>::PrimaryKey,
        #remote_filter,
        #foreign_key,
        >>
    }))
}

fn handle_has_one(f: &Field, table_ty: &Ident) -> Option<Result<TokenStream, Diagnostic>> {
    let field_name = f.rust_name();
    let field_ty = &f.ty;
    let sql_name = f.sql_name();
    let reference_ty = if is_option_ty(inner_ty_arg(field_ty, "HasOne", 1).expect("It's there")) {
        quote!(::wundergraph::filter::NullableReferenceFilter)
    } else {
        quote!(::wundergraph::filter::ReferenceFilter)
    };
    let remote_table = f
        .remote_table()
        .map(|t| quote!(#t::table))
        .unwrap_or_else(|_| {
            let remote_type =
                inner_of_option_ty(inner_ty_arg(&f.ty, "HasOne", 1).expect("It's HasOne"));
            quote!{
                <#remote_type as ::wundergraph::diesel::associations::HasTable>::Table
            }
        });
    let remote_filter = f.filter().expect("Filter is missing");
    let graphql_name = f.graphql_name().to_string();
    Some(Ok(quote!{
        #[wundergraph(graphql_name = #graphql_name)]
        #field_name: ::std::option::Option<#reference_ty<
        #table_ty::#sql_name,
        #remote_filter,
        <#remote_table as ::wundergraph::diesel::Table>::PrimaryKey,
        >>
    }))
}
