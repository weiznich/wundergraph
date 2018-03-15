use quote;
use syn;
use diagnostic_shim::Diagnostic;
use utils::wrap_in_dummy_mod;
use model::Model;
use field::Field;

pub fn derive(item: &syn::DeriveInput) -> Result<quote::Tokens, Diagnostic> {
    let item_name = item.ident;
    let model = Model::from_item(item)?;
    let (impl_generics, ty_generics, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    {
        // TODO: improve this
        // maybe try to remove the explicit Backend bound and
        // replace it with with the next level of bounds?
        let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(DB: Backend + 'static));
    }
    let (_, _, where_clause) = generics.split_for_impl();

    let table = model.table_type()?;

    let fields = model
        .fields()
        .iter()
        .map(build_field_filter)
        .collect::<Result<Vec<_>, _>>()?;

    let dummy_mod = model.dummy_mod_name("build_filter");
    Ok(wrap_in_dummy_mod(
        dummy_mod,
        &quote! {
            use self::wundergraph::filter::build_filter::BuildFilter;
            use self::wundergraph::filter::collector::AndCollector;
            use self::wundergraph::diesel::expression::BoxableExpression;
            use self::wundergraph::diesel::sql_types::Bool;
            use self::wundergraph::filter::transformator::Transformator;

            impl #impl_generics BuildFilter for #item_name #ty_generics
                #where_clause
            {
                type Ret = Box<BoxableExpression<#table::table, DB, SqlType = Bool>>;

                fn into_filter<__T>(self, t: __T) -> Option<Self::Ret>
                where
                    __T: Transformator
                {

                    let mut and = AndCollector::default();

                    #(#fields)*

                    and.into_filter(t)
                }
            }
        },
    ))
}

fn build_field_filter(field: &Field) -> Result<quote::Tokens, Diagnostic> {
    let field_access = field.name.access();
    Ok(quote!(<_ as self::wundergraph::filter::collector::FilterCollector<_, _>>::append_filter(&mut and, self #field_access, t);))
}
