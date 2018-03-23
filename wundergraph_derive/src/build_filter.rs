use quote;
use syn;
use diagnostic_shim::Diagnostic;
use utils::wrap_in_dummy_mod;
use model::Model;
use field::Field;

pub fn derive(item: &syn::DeriveInput) -> Result<quote::Tokens, Diagnostic> {
    let model = Model::from_item(item)?;
    let table = model.table_type()?;

    let fields = model
        .fields()
        .iter()
        .map(build_field_filter)
        .collect::<Result<Vec<_>, _>>()?;

    let pg = if cfg!(feature = "postgres") {
        Some(impl_build_filter(
            item,
            &fields,
            &quote!(diesel::pg::Pg),
            table,
        ))
    } else {
        None
    };

    let sqlite = if cfg!(feature = "sqlite") {
        Some(impl_build_filter(
            item,
            &fields,
            &quote!(diesel::sqlite::Sqlite),
            table,
        ))
    } else {
        None
    };

    let dummy_mod = model.dummy_mod_name("build_filter");
    Ok(wrap_in_dummy_mod(
        dummy_mod,
        &quote! {
            use self::wundergraph::filter::build_filter::BuildFilter;
            use self::wundergraph::filter::collector::AndCollector;
            use self::wundergraph::diesel::expression::BoxableExpression;
            use self::wundergraph::diesel::sql_types::Bool;
            use self::wundergraph::filter::transformator::Transformator;

            #pg
            #sqlite

        },
    ))
}

fn impl_build_filter(
    item: &syn::DeriveInput,
    fields: &[quote::Tokens],
    backend: &quote::Tokens,
    table: syn::Ident,
) -> quote::Tokens {
    let item_name = item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    quote!{
        impl #impl_generics BuildFilter<#backend> for #item_name #ty_generics
            #where_clause

        {
            type Ret = Box<BoxableExpression<#table::table, #backend, SqlType = Bool>>;

            fn into_filter<__T>(self, t: __T) -> Option<Self::Ret>
            where
                __T: Transformator
            {

                let mut and = AndCollector::<_, #backend>::default();

                #(#fields)*

                and.into_filter(t)
            }
        }
    }
}

fn build_field_filter(field: &Field) -> Result<quote::Tokens, Diagnostic> {
    let field_access = field.name.access();
    Ok(
        quote!(<_ as self::wundergraph::filter::collector::FilterCollector<_, _>>::append_filter(&mut and, self #field_access, t);),
    )
}
