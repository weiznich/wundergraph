use proc_macro2::{Span, TokenStream};
use syn::*;

pub fn wrap_in_dummy_mod(const_name: &Ident, item: &TokenStream) -> TokenStream {
    wrap_in_dummy_mod_with_reeport(const_name, item, &[])
}

pub fn wrap_in_dummy_mod_with_reeport(
    const_name: &Ident,
    item: &TokenStream,
    reexport: &[TokenStream],
) -> TokenStream {
    let reexport = reexport.iter().map(|r| {
        quote!{
            #[doc(inline)]
            pub use self::#const_name::#r;
        }
    });
    let call_site = root_span(Span::call_site());
    let use_everything = quote_spanned!(call_site=> __wundergraph_use_everything!());
    quote! {
        #[allow(non_snake_case)]
        mod #const_name {
            // https://github.com/rust-lang/rust/issues/47314
            extern crate std;

            mod wundergraph {
                #use_everything;
            }
            #item
        }
        #(#reexport)*
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
    inner_ty_arg(ty, "HasMany", 0).is_some()
}

pub fn is_has_one(ty: &Type) -> bool {
    inner_ty_arg(ty, "HasOne", 0).is_some()
}

pub fn is_lazy_load(ty: &Type) -> bool {
    inner_ty_arg(ty, "LazyLoad", 0).is_some()
}

pub fn inner_ty_arg<'a>(ty: &'a Type, type_name: &str, index: usize) -> Option<&'a Type> {
    use syn::PathArguments::AngleBracketed;

    match *ty {
        Type::Path(ref ty) => {
            let last_segment = ty.path
                .segments
                .iter()
                .last()
                .expect("Path without any segments");
            match last_segment.arguments {
                AngleBracketed(ref args) if last_segment.ident == type_name => {
                    match args.args[index] {
                        GenericArgument::Type(ref ty) => Some(ty),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn ty_name(ty: &Type) -> Option<&Ident> {
    match *ty {
        Type::Path(ref ty) => {
            let last_segment = ty.path
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
