extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, Type};

#[proc_macro_attribute]
pub fn archive_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    assert!(attr.is_empty(), "No attribute inputs expected");

    let orig_impl = parse_macro_input!(item as ItemImpl);
    let orig_name = match &*orig_impl.self_ty {
        Type::Path(path) => path.path.segments.last().unwrap().ident.clone(),
        _ => unimplemented!(),
    };
    let archived_name = format!("Archived{orig_name}");
    let archived_ident = syn::Ident::new(&archived_name, orig_name.span());
    let (impl_generics, ty_generics, where_clause) = orig_impl.generics.split_for_impl();
    let impl_items = &orig_impl.items;

    // TODO: is there a way to avoid duplication here?
    if let Some((_, trait_path, _)) = &orig_impl.trait_ {
        quote! {
            #orig_impl

            impl #impl_generics #trait_path for #archived_ident #ty_generics #where_clause {
                #(#impl_items)*
            }
        }
    } else {
        quote! {
            #orig_impl

            impl #impl_generics #archived_ident #ty_generics #where_clause {
                #(#impl_items)*
            }
        }
    }
    .into()
}
