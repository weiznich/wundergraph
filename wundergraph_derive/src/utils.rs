use proc_macro2::{Ident, Span, TokenStream};
use syn::*;

pub fn wrap_in_dummy_mod(
    name_place_holder: &str,
    ident: &Ident,
    item: &TokenStream,
) -> TokenStream {
    let call_site = root_span(Span::call_site());
    let const_name = Ident::new(
        &format!("_impl_{}_for_{}", name_place_holder, ident.to_string()).to_uppercase(),
        call_site,
    );
    quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        const #const_name: () = {
            extern crate std;
            mod wundergraph {
                __wundergraph_use_everything!();
            }
            #item
        };
    }
}

pub fn inner_of_option_ty(ty: &Type) -> &Type {
    inner_ty_arg(ty, "Option", 0).unwrap_or(ty)
}

pub fn is_option_ty(ty: &Type) -> bool {
    inner_ty_arg(ty, "Option", 0).is_some()
}

pub fn inner_of_box_ty(ty: &Type) -> &Type {
    inner_ty_arg(ty, "Box", 0).unwrap_or(ty)
}

pub fn is_box_ty(ty: &Type) -> bool {
    inner_ty_arg(ty, "Box", 0).is_some()
}

pub fn is_has_many(ty: &Type) -> bool {
    inner_ty_arg(inner_of_option_ty(ty), "HasMany", 0).is_some()
}

pub fn is_has_one(ty: &Type) -> bool {
    inner_ty_arg(inner_of_option_ty(ty), "HasOne", 0).is_some()
}

pub fn inner_ty_args<'a>(
    ty: &'a Type,
    type_name: &str,
) -> Option<&'a syn::punctuated::Punctuated<syn::GenericArgument, syn::token::Comma>> {
    use syn::PathArguments::AngleBracketed;

    match *ty {
        Type::Path(ref ty) => {
            let last_segment = ty
                .path
                .segments
                .iter()
                .last()
                .expect("Path without any segments");
            match last_segment.arguments {
                AngleBracketed(ref args) if last_segment.ident == type_name => Some(&args.args),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn inner_ty_arg<'a>(ty: &'a Type, type_name: &str, index: usize) -> Option<&'a Type> {
    inner_ty_args(ty, type_name).and_then(|args| match args[index] {
        GenericArgument::Type(ref ty) => Some(ty),
        _ => None,
    })
}

pub fn ty_name(ty: &Type) -> Option<&Ident> {
    match *ty {
        Type::Path(ref ty) => {
            let last_segment = ty
                .path
                .segments
                .iter()
                .last()
                .expect("Path without any segments");
            Some(&last_segment.ident)
        }
        _ => None,
    }
}

pub fn fix_span(maybe_bad_span: Span, fallback: Span) -> Span {
    let bad_span_debug = "Span(Span { lo: BytePos(0), hi: BytePos(0), ctxt: #0 })";
    if format!("{:?}", maybe_bad_span) == bad_span_debug {
        fallback
    } else {
        maybe_bad_span
    }
}

#[cfg(not(feature = "nightly"))]
fn root_span(span: Span) -> Span {
    span
}

#[cfg(feature = "nightly")]
/// There's an issue with the resolution of `__diesel_use_everything` if the
/// derive itself was generated from within a macro. This is a shitty workaround
/// until we figure out the expected behavior.
fn root_span(span: Span) -> Span {
    span.unstable().source().into()
}
