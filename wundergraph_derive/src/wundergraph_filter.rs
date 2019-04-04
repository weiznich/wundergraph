use crate::diagnostic_shim::Diagnostic;
use crate::field::Field;
use crate::model::Model;
use crate::utils::{inner_of_box_ty, inner_of_option_ty, is_box_ty, wrap_in_dummy_mod};
use proc_macro2::TokenStream;
use syn;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let inner_filter = inner_filter(item)?;
    let build_filter = build_filter(item)?;
    let nameable = crate::wundergraph_value::nameable(item);

    Ok(wrap_in_dummy_mod(
        "WundergraphFilter",
        &item.ident,
        &quote! {
            use wundergraph::juniper::{self, InputValue, LookAheadValue, Registry};
            use wundergraph::juniper::meta::Argument;
            use wundergraph::indexmap::IndexMap;
            use wundergraph::filter::inner_filter::InnerFilter;
            use wundergraph::helper::NameBuilder;
            use wundergraph::scalar::WundergraphScalarValue;
            use wundergraph::filter::build_filter::BuildFilter;
            use wundergraph::filter::collector::AndCollector;
            use wundergraph::diesel_ext::BoxableFilter;
            use wundergraph::diesel::sql_types::Bool;
            use wundergraph::helper::Nameable;
            use wundergraph::diesel;

            #inner_filter
            #build_filter
            #nameable
        },
    ))
}

fn build_filter(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let model = Model::from_item(item)?;
    let table = &model.table_type()?;
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
    Ok(quote! {
        #pg
        #sqlite
    })
}

fn inner_filter(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let model = Model::from_item(item)?;
    let item_name = &item.ident;

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let field_count = model.fields().len();

    let from_inner_input_value = build_from_inner_input_value(&model)?;
    let from_inner_look_ahead = build_from_look_ahead(&model)?;
    let to_inner_input_value = build_to_inner_input_value(&model)?;
    let register_fields = build_register_fields(&model)?;
    Ok(quote! {
        impl #impl_generics InnerFilter for #item_name #ty_generics
            #where_clause
        {
            type Context = ();

            const FIELD_COUNT: usize = #field_count;

            fn from_inner_input_value(
                obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>
            ) -> Option<Self> {
                #from_inner_input_value
            }

            fn from_inner_look_ahead(
                obj: &[(&str, LookAheadValue<WundergraphScalarValue>)]
            ) -> Self {
                #from_inner_look_ahead
            }

            fn to_inner_input_value(
                &self, v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>
            ) {
                #to_inner_input_value
            }

            fn register_fields<'r>(
                _info: &NameBuilder<Self>,
                registry: &mut Registry<'r, WundergraphScalarValue>,
            ) -> Vec<Argument<'r, WundergraphScalarValue>> {
                #register_fields
            }
        }
    })
}

fn build_from_inner_input_value(model: &Model) -> Result<TokenStream, Diagnostic> {
    let build_field = model.fields().iter().map(|f| {
        let field_name = f.rust_name();
        let graphql_name = f.graphql_name();
        let map_box = if is_box_ty(inner_of_option_ty(&f.ty)) {
            Some(quote!(.map(Box::new)))
        } else {
            None
        };
        quote!(
            let #field_name = obj.get(stringify!(#graphql_name))
                .map(|v| <Option<_> as juniper::FromInputValue<_>>::from_input_value(*v))
                .unwrap_or_else(|| {
                    let v: &InputValue<WundergraphScalarValue> = &InputValue::Null;
                    <Option<_> as juniper::FromInputValue<_>>::from_input_value(v)
                });
            let #field_name = match #field_name {
                Some(v) => v#map_box,
                None => return None,
            };
        )
    });
    let fields = model.fields().iter().map(Field::rust_name);
    Ok(quote! {
        #(#build_field)*

        Some(Self{ #(#fields,)* })
    })
}

fn build_from_look_ahead(model: &Model) -> Result<TokenStream, Diagnostic> {
    let build_field = model.fields().iter().map(|f| {
        let field_name = f.rust_name();
        let graphq_name = f.graphql_name();
        let ty = inner_of_option_ty(&f.ty);
        let map_box = if is_box_ty(ty) {
            Some(quote!(.map(Box::new)))
        } else {
            None
        };
        let ty = inner_of_box_ty(ty);
        quote!{
            let #field_name = obj.iter()
                .find(|o| o.0 == stringify!(#graphq_name))
                .and_then(|o| <#ty as wundergraph::helper::FromLookAheadValue>::from_look_ahead(&o.1))
                #map_box;
        }
    });
    let fields = model.fields().iter().map(Field::rust_name);
    Ok(quote! {
        #(#build_field)*

        Self{ #(#fields,)* }
    })
}

fn build_to_inner_input_value(model: &Model) -> Result<TokenStream, Diagnostic> {
    let to_values = model.fields().iter().map(|f| {
        let name = &f.rust_name().access();

        quote! {
            v.insert(stringify!(#name), juniper::ToInputValue::<_>::to_input_value(&self#name));
        }
    });
    Ok(quote! {
        #(#to_values)*
    })
}

fn build_register_fields(model: &Model) -> Result<TokenStream, Diagnostic> {
    let register_field = model.fields().iter().map(|f| {
        let field_name = f.graphql_name();
        let ty = inner_of_option_ty(&f.ty);
        quote! {
            let #field_name = registry.arg_with_default::<Option<#ty>>(
                stringify!(#field_name),
                &None,
                &wundergraph::helper::NameBuilder::default()
            );
        }
    });
    let fields = model.fields().iter().map(Field::graphql_name);
    Ok(quote! {
        #(#register_field)*
        vec![#(#fields,)*]
    })
}

fn impl_build_filter(
    item: &syn::DeriveInput,
    fields: &[TokenStream],
    backend: &TokenStream,
    table: &syn::Ident,
) -> TokenStream {
    let item_name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    quote! {
        impl #impl_generics BuildFilter<#backend> for #item_name #ty_generics
            #where_clause

        {
            type Ret = Box<BoxableFilter<#table::table, #backend, SqlType = Bool>>;

            fn into_filter(self) -> Option<Self::Ret>
            {

                let mut and = AndCollector::<_, #backend>::default();

                #(#fields)*

                and.into_filter()
            }
        }
    }
}

fn build_field_filter(field: &Field) -> Result<TokenStream, Diagnostic> {
    let field_access = field.rust_name().access();
    Ok(
        quote!(<_ as wundergraph::filter::collector::FilterCollector<_, _>>::append_filter(&mut and, self #field_access);),
    )
}
