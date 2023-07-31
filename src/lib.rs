extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, Type};

#[proc_macro_attribute]
pub fn archive_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    assert!(attr.is_empty(), "No attribute inputs expected");

    let orig_impl = parse_macro_input!(item as ItemImpl);
    let archived_path = match &*orig_impl.self_ty {
        Type::Path(path) => replace_last_path_segment(&path.path),
        unsupported_self_ty => {
            let self_ty_verbatim = quote! { #unsupported_self_ty };
            panic!("`impl {self_ty_verbatim}` unsupported: self type can only be syn::Type::Path")
        }
    };
    let (impl_generics, _ty_generics, where_clause) = orig_impl.generics.split_for_impl();
    let impl_items = &orig_impl.items;

    // TODO: is there a way to avoid duplication here?
    if let Some((_, trait_path, _)) = &orig_impl.trait_ {
        quote! {
            #orig_impl

            impl #impl_generics #trait_path for #archived_path #where_clause {
                #(#impl_items)*
            }
        }
    } else {
        quote! {
            #orig_impl

            impl #impl_generics #archived_path #where_clause {
                #(#impl_items)*
            }
        }
    }
    .into()
}

fn replace_last_path_segment(p: &syn::Path) -> syn::Path {
    let orig_ident = &p.segments.last().unwrap().ident;
    let archived_name = format!("Archived{orig_ident}");
    let archived_ident = syn::Ident::new(&archived_name, orig_ident.span());
    let mut archived_path = p.clone();
    archived_path.segments.last_mut().unwrap().ident = archived_ident;
    archived_path
}
