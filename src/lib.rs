//! Copy `impl Foo` blocks into `impl ArchivedFoo`.
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
//! #[archive_impl(transform_bounds(T))]
//! impl<T> Foo<T> {
//!     // Notice that the where clause is transformed so that
//!     // `T` is replaced with `T::Archived` in the generated `impl`.
//!     #[archive_method(transform_bounds(T))]
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
//!     // Call the generated method!
//!     let _ = foo.sum::<u32>();
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, visit_mut::VisitMut,
    ImplItem, ItemImpl, Meta, Token, Type, WhereClause, WherePredicate,
};

/// Supports the same arguments as [`macro@archive_impl`], but applies to
/// methods on an `impl` block.
#[proc_macro_attribute]
pub fn archive_method(_: TokenStream, item: TokenStream) -> TokenStream {
    // No-op that just fails if placed on anything but a method. Arguments are
    // inspected from the `archive_impl` macro.
    let cloned_item = item.clone();
    let parsed = parse_macro_input!(cloned_item as ImplItem);
    match parsed {
        ImplItem::Fn(_) => (),
        unsupported_item => {
            let item_verbatim = quote! { #unsupported_item };
            panic!(
                "Unsupported item `{item_verbatim}`. `archive_method` can only be applied to methods."
            );
        }
    }
    item
}

/// Decorates an `impl T` (or `impl FooTrait for T`) block and generates an
/// equivalent `impl T::Archived`.
///
/// Arguments to this attribute macro include:
/// - `transform_bounds(T)`: Adds a `T: Archive` bound and transforms `T` into
///   `T::Archived` in all trait bounds on the `impl`. Can take a list of
///   multiple parameters, like `transform_bounds(T, S)`.
/// - `bounds(...)`: Adds bounds to the generated `impl`. Takes a list of
///   predicates, for example: `bounds(T: PartialEq, S: Hash)`.
///
/// Note that generated bounds are only added to the `where` clause on the
/// `impl`. To transform or add bounds to specific methods, see
/// [`macro@archive_method`].
#[proc_macro_attribute]
pub fn archive_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse macro arguments.
    let mut arg_metas = Vec::new();
    let mut add_impl_bounds = Vec::new();
    let mut transform_params = Vec::new();
    if !args.is_empty() {
        if let Err(e) = parse_argument_metas(args, &mut arg_metas) {
            return e.to_compile_error().into();
        }
        for meta in arg_metas {
            if meta.path().is_ident("transform_bounds") {
                if let Err(e) = parse_transform_bounds(&meta, &mut transform_params) {
                    return e.to_compile_error().into();
                }
            } else if meta.path().is_ident("bounds") {
                if let Err(e) = parse_bounds(&meta, &mut add_impl_bounds) {
                    return e.to_compile_error().into();
                }
            } else {
                let meta_path = meta.path().get_ident().unwrap();
                panic!("Unsupported argument `{meta_path}`");
            }
        }
        for param in &transform_params {
            add_impl_bounds.push(parse_quote! { #param: Archive });
        }
    }

    let orig_impl = parse_macro_input!(item as ItemImpl);

    let archived_path = match &*orig_impl.self_ty {
        Type::Path(path) => replace_last_path_segment(&path.path),
        unsupported_self_ty => {
            let self_ty_verbatim = quote! { #unsupported_self_ty };
            panic!("`impl {self_ty_verbatim}` unsupported: self type can only be syn::Type::Path")
        }
    };

    let (impl_generics, _ty_generics, orig_where_clause) = orig_impl.generics.split_for_impl();

    let mut archived_where_clause = orig_where_clause.cloned();
    transform_where_clause(&transform_params, &mut archived_where_clause);

    let archived_where_clause = add_bounds_to_where_clause(archived_where_clause, add_impl_bounds);

    let mut augmented_impl_items = orig_impl.items.clone();
    if let Err(e) = augment_methods(&mut augmented_impl_items) {
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
fn augment_methods(augmented_items: &mut [ImplItem]) -> syn::Result<()> {
    for item in augmented_items {
        if let ImplItem::Fn(fn_item) = item {
            let mut add_bounds = Vec::new();
            let mut transform_params = Vec::new();
            for attr in &fn_item.attrs {
                if !attr.path().is_ident("archive_method") {
                    continue;
                }

                match &attr.meta {
                    Meta::List(meta_list) => {
                        let mut arg_metas = Vec::new();
                        parse_argument_metas(meta_list.tokens.clone().into(), &mut arg_metas)?;

                        for meta in arg_metas {
                            if meta.path().is_ident("transform_bounds") {
                                parse_transform_bounds(&meta, &mut transform_params)?;
                            } else if meta.path().is_ident("bounds") {
                                parse_bounds(&meta, &mut add_bounds)?;
                            } else {
                                let meta_path = meta.path().get_ident().unwrap();
                                panic!("Unsupported argument `{meta_path}`");
                            }
                        }
                    }
                    unsupported_meta => {
                        let meta_verbatim = quote! { #unsupported_meta };
                        panic!(
                            "Unsupported meta `{meta_verbatim}`: meta can only be structure list `archive_method(...)`"
                        );
                    }
                }
            }
            for param in &transform_params {
                add_bounds.push(parse_quote! { #param: Archive });
            }
            let method_where = &mut fn_item.sig.generics.where_clause;
            transform_where_clause(&transform_params, method_where);
            *method_where = add_bounds_to_where_clause(std::mem::take(method_where), add_bounds);
        }
    }
    Ok(())
}

fn transform_where_clause(replace_params: &[Ident], clause: &mut Option<WhereClause>) {
    struct TypeReplacer<'a> {
        replace_params: &'a [Ident],
        archived_assoc: Ident,
    }
    impl<'a> VisitMut for TypeReplacer<'a> {
        fn visit_type_path_mut(&mut self, i: &mut syn::TypePath) {
            for r in self.replace_params {
                if i.path.is_ident(r) {
                    i.path.segments.push(self.archived_assoc.clone().into());
                }
            }
        }
    }

    let Some(clause) = clause else { return };

    TypeReplacer {
        replace_params,
        archived_assoc: Ident::new("Archived", Span::call_site()),
    }
    .visit_where_clause_mut(clause);
}

fn add_bounds_to_where_clause(
    orig_where_clause: Option<WhereClause>,
    additional_bounds: Vec<WherePredicate>,
) -> Option<WhereClause> {
    if orig_where_clause.is_none() && additional_bounds.is_empty() {
        return None;
    }

    let mut bounds = additional_bounds;
    if let Some(clause) = orig_where_clause {
        bounds.extend(clause.predicates.into_iter());
    }

    Some(WhereClause {
        where_token: Token![where](Span::call_site()),
        predicates: Punctuated::from_iter(bounds),
    })
}

fn parse_argument_metas(args: TokenStream, arg_lists: &mut Vec<Meta>) -> syn::Result<()> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    arg_lists.extend(parser.parse(args)?.into_iter());
    Ok(())
}

fn parse_transform_bounds(meta: &Meta, transform_params: &mut Vec<Ident>) -> syn::Result<()> {
    match meta {
        Meta::List(meta_list) => {
            let parser = Punctuated::<Ident, Token![,]>::parse_terminated;
            transform_params.extend(parser.parse(meta_list.tokens.clone().into())?.into_iter());
            Ok(())
        }
        unsupported_meta => {
            let meta_verbatim = quote! { #unsupported_meta };
            panic!("Unsupported `{meta_verbatim}`: meta can only be structured list `transform_bounds(...)`");
        }
    }
}

fn parse_bounds(meta: &Meta, add_bounds: &mut Vec<WherePredicate>) -> syn::Result<()> {
    match meta {
        Meta::List(meta_list) => {
            let parser = Punctuated::<WherePredicate, Token![,]>::parse_terminated;
            add_bounds.extend(parser.parse(meta_list.tokens.clone().into())?.into_iter());
            Ok(())
        }
        unsupported_meta => {
            let meta_verbatim = quote! { #unsupported_meta };
            panic!("Unsupported `{meta_verbatim}`: meta can only be structured list `bounds(...)`");
        }
    }
}
