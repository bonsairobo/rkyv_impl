//! Copy `impl T` blocks into `impl ArchivedT`.
//!
//! ```
//! use rkyv::Archive;
//! use rkyv_impl::*;
//! use std::iter::Sum;
//!
//! #[derive(Archive)]
//! struct Foo<T> {
//!     elements: Vec<T>
//! }
//!
//! #[archive_impl(bounds(T: Archive))]
//! impl<T> Foo<T> {
//!     #[archive_method(bounds(T::Archived: Clone, S: Sum<T::Archived>))]
//!     fn sum<S>(&self) -> S
//!     where
//!         T: Clone,
//!         S: Sum<T>
//!     {
//!         self.elements.iter().cloned().sum()
//!     }
//! }
//!
//! fn use_generated_method(foo: &ArchivedFoo<u32>) {
//!     let _ = foo.sum::<u32>();
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, ImplItem, ItemImpl, Meta, Token,
    Type, WhereClause, WherePredicate,
};

#[proc_macro_attribute]
pub fn archive_method(_: TokenStream, item: TokenStream) -> TokenStream {
    // No-op that just fails if placed on anything but a method.
    let cloned_item = item.clone();
    parse_macro_input!(cloned_item as ImplItem);
    item
}

#[proc_macro_attribute]
pub fn archive_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let archive_impl_bounds = if attr.is_empty() {
        Vec::new()
    } else {
        let meta = parse_macro_input!(attr as Meta);
        match parse_bounds(&meta) {
            Ok(b) => b,
            Err(e) => return e.to_compile_error().into(),
        }
    };
    let orig_impl = parse_macro_input!(item as ItemImpl);

    let archived_path = match &*orig_impl.self_ty {
        Type::Path(path) => replace_last_path_segment(&path.path),
        unsupported_self_ty => {
            let self_ty_verbatim = quote! { #unsupported_self_ty };
            panic!("`impl {self_ty_verbatim}` unsupported: self type can only be syn::Type::Path")
        }
    };

    let (impl_generics, _ty_generics, orig_where_clause) = orig_impl.generics.split_for_impl();

    let archived_where_clause = add_bounds_to_where_clause(orig_where_clause, archive_impl_bounds);

    let mut augmented_impl_items = orig_impl.items.clone();
    if let Err(e) = add_bounds_to_methods(&mut augmented_impl_items) {
        return e.to_compile_error().into();
    }

    // TODO: is there a way to avoid duplication here?
    if let Some((_, trait_path, _)) = &orig_impl.trait_ {
        quote! {
            #orig_impl

            impl #impl_generics #trait_path for #archived_path #archived_where_clause {
                #(#augmented_impl_items)*
            }
        }
    } else {
        quote! {
            #orig_impl

            impl #impl_generics #archived_path #archived_where_clause {
                #(#augmented_impl_items)*
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

// Augments the where clause of each method with an `archive_method` attribute.
fn add_bounds_to_methods(augmented_items: &mut [ImplItem]) -> syn::Result<()> {
    for item in augmented_items {
        if let ImplItem::Fn(fn_item) = item {
            let mut new_bounds = Vec::new();
            for attr in &fn_item.attrs {
                if !attr.path().is_ident("archive_method") {
                    continue;
                }

                match &attr.meta {
                    Meta::List(meta_list) => {
                        let inner_meta = syn::parse2::<Meta>(meta_list.tokens.clone())?;
                        new_bounds.append(&mut parse_bounds(&inner_meta)?);
                    }
                    unsupported_meta => {
                        let meta_verbatim = quote! { #unsupported_meta };
                        panic!(
                            "Unsupported meta `{meta_verbatim}`: meta can only be structure list `archive_method(...)`"
                        );
                    }
                }
            }
            let method_where = &mut fn_item.sig.generics.where_clause;
            *method_where = add_bounds_to_where_clause(method_where.as_ref(), new_bounds);
        }
    }
    Ok(())
}

fn add_bounds_to_where_clause(
    orig_where_clause: Option<&WhereClause>,
    additional_bounds: Vec<WherePredicate>,
) -> Option<WhereClause> {
    if orig_where_clause.is_none() && additional_bounds.is_empty() {
        return None;
    }

    let mut bounds = additional_bounds;
    if let Some(clause) = orig_where_clause {
        bounds.extend(clause.predicates.clone().into_iter());
    }

    Some(WhereClause {
        where_token: Token![where](Span::call_site()),
        predicates: Punctuated::from_iter(bounds),
    })
}

fn parse_bounds(meta: &Meta) -> syn::Result<Vec<WherePredicate>> {
    match meta {
        Meta::List(meta_list) => {
            if meta_list.path.is_ident("bounds") {
                let parser = Punctuated::<WherePredicate, Token![,]>::parse_terminated;
                Ok(parser
                    .parse(meta_list.tokens.clone().into())?
                    .into_iter()
                    .collect())
            } else {
                // panic!("Unsupported meta: {}", meta_list.path.get_ident().unwrap());
                Ok(Vec::new())
            }
        }
        unsupported_meta => {
            let meta_verbatim = quote! { #unsupported_meta };
            panic!(
                "Unsupported meta `{meta_verbatim}`: meta can only be structure list `bound(...)`"
            );
        }
    }
}
