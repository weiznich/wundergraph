use quote;
use syn;
use diagnostic_shim::Diagnostic;
use utils::{inner_of_box_ty, inner_of_option_ty, is_box_ty, wrap_in_dummy_mod};
use model::Model;

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
    let field_count = model.fields().len();

    let from_inner_input_value = build_from_inner_input_value(&model)?;
    let from_inner_look_ahead = build_from_look_ahead(&model)?;
    let to_inner_input_value = build_to_inner_input_value(&model)?;
    let register_fields = build_register_fields(&model)?;

    let dummy_mod = model.dummy_mod_name("inner_filter");
    Ok(wrap_in_dummy_mod(
        dummy_mod,
        &quote! {
            use self::wundergraph::juniper::{self, InputValue, LookAheadValue, Registry};
            use self::wundergraph::juniper::meta::Argument;
            use self::wundergraph::ordermap::OrderMap;
            use self::wundergraph::filter::inner_filter::InnerFilter;
            use self::wundergraph::helper::NameBuilder;

            impl #impl_generics InnerFilter for #item_name #ty_generics
                #where_clause
            {
                type Context = ();

                const FIELD_COUNT: usize = #field_count;

                fn from_inner_input_value(obj: OrderMap<&str, &InputValue>) -> Option<Self> {
                    #from_inner_input_value
                }

                fn from_inner_look_ahead(obj: &[(&str, LookAheadValue)]) -> Self {
                    #from_inner_look_ahead
                }

                fn to_inner_input_value(&self, v: &mut OrderMap<&str, InputValue>) {
                    #to_inner_input_value
                }

                fn register_fields<'r>(
                    _info: &NameBuilder<Self>,
                    registry: &mut Registry<'r>,
                ) -> Vec<Argument<'r>> {
                    #register_fields
                }
            }
        },
    ))
}

fn build_from_inner_input_value(model: &Model) -> Result<quote::Tokens, Diagnostic> {
    let build_field = model.fields().iter().map(|f| {
        let field_name = &f.name;
        let map_box = if is_box_ty(inner_of_option_ty(&f.ty)) {
            Some(quote!(.map(Box::new)))
        } else {
            None
        };
        quote!(
            let #field_name = obj.get(stringify!(#field_name))
                .map(|v| <Option<_> as juniper::FromInputValue>::from_input_value(*v))
                .unwrap_or_else(|| <Option<_> as juniper::FromInputValue>::from_input_value(&InputValue::Null));
            let #field_name = match #field_name {
                Some(v) => v#map_box,
                None => return None,
            };
        )
    });
    let fields = model.fields().iter().map(|f| &f.name);
    Ok(quote!{
        #(#build_field)*

        Some(Self{ #(#fields,)* })
    })
}

fn build_from_look_ahead(model: &Model) -> Result<quote::Tokens, Diagnostic> {
    let build_field = model.fields().iter().map(|f| {
        let field_name = &f.name;
        let ty = inner_of_option_ty(&f.ty);
        let map_box = if is_box_ty(ty) {
            Some(quote!(.map(Box::new)))
        } else {
            None
        };
        let ty = inner_of_box_ty(ty);
        quote!{
            let #field_name = obj.iter()
                .find(|o| o.0 == stringify!(#field_name))
                .and_then(|o| <#ty as self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(&o.1))
                #map_box;
        }
    });
    let fields = model.fields().iter().map(|f| &f.name);
    Ok(quote!{
        #(#build_field)*

        Self{ #(#fields,)* }
    })
}

fn build_to_inner_input_value(model: &Model) -> Result<quote::Tokens, Diagnostic> {
    let to_values = model.fields().iter().map(|f| {
        let name = &f.name.access();

        quote!{
            v.insert(stringify!(#name), juniper::ToInputValue::to_input_value(&self#name));
        }
    });
    Ok(quote!{
        #(#to_values)*
    })
}

fn build_register_fields(model: &Model) -> Result<quote::Tokens, Diagnostic> {
    let register_field = model.fields().iter().map(|f| {
        let field_name = &f.name;
        let ty = inner_of_option_ty(&f.ty);
        quote!{
            let #field_name = registry.arg_with_default::<Option<#ty>>(
                stringify!(#field_name),
                &None,
                &Default::default()
            );
        }
    });
    let fields = model.fields().iter().map(|f| &f.name);
    Ok(quote!{
        #(#register_field)*
        vec![#(#fields,)*]
    })
}
