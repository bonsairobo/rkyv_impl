//! Implement methods for `Foo` and `ArchivedFoo` in a single `impl` block.
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
use std::collections::HashSet;
use syn::{
    parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, visit_mut::VisitMut,
    ImplItem, ImplItemFn, ItemImpl, Meta, Token, Type, WhereClause, WherePredicate,
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
    let args = match Arguments::parse(args) {
        Ok(a) => a,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

    let orig_impl = parse_macro_input!(item as ItemImpl);

    // Clone the original impl to be sure we don't accidentally drop anything,
    // we only need to find and transform certain parts.
    let mut archived_impl = orig_impl.clone();
    replace_self_type(&mut archived_impl.self_ty);
    transform_where_clause(
        &args.transform_params,
        &mut archived_impl.generics.where_clause,
    );
    add_bounds_to_where_clause(args.add_bounds, &mut archived_impl.generics.where_clause);
    if let Err(e) = augment_methods(&mut archived_impl.items) {
        return e.to_compile_error().into();
    }

    quote! {
        #orig_impl
        #archived_impl
    }
    .into()
}

#[derive(Default)]
struct Arguments {
    add_bounds: Vec<WherePredicate>,
    transform_params: Vec<Ident>,
}

impl Arguments {
    fn parse(args: TokenStream) -> syn::Result<Self> {
        let mut builder = ArgumentsBuilder::default();
        builder.try_add_metas_token_stream(args)?;
        Ok(builder.build())
    }
}

#[derive(Default)]
struct ArgumentsBuilder {
    add_bounds: Vec<WherePredicate>,
    transform_params: HashSet<Ident>,
}

impl ArgumentsBuilder {
    fn try_add_metas_token_stream(&mut self, args: TokenStream) -> syn::Result<()> {
        if !args.is_empty() {
            let mut arg_metas = Vec::new();
            parse_argument_metas(args, &mut arg_metas)?;
            for meta in arg_metas {
                self.try_add_meta(meta)?;
            }
        }
        Ok(())
    }

    fn try_add_meta(&mut self, meta: Meta) -> syn::Result<()> {
        if meta.path().is_ident("transform_bounds") {
            parse_transform_bounds(&meta, &mut self.transform_params)?;
        } else if meta.path().is_ident("bounds") {
            parse_bounds(&meta, &mut self.add_bounds)?;
        } else {
            let meta_path = meta.path().get_ident().unwrap();
            panic!("Unsupported argument `{meta_path}`");
        }
        Ok(())
    }

    fn build(mut self) -> Arguments {
        for param in &self.transform_params {
            self.add_bounds.push(parse_quote! { #param: Archive });
        }
        Arguments {
            add_bounds: self.add_bounds,
            transform_params: self.transform_params.into_iter().collect(),
        }
    }
}

fn replace_self_type(self_type: &mut Type) {
    match &mut *self_type {
        Type::Path(path) => replace_last_path_segment(&mut path.path),
        unsupported_self_ty => {
            let self_ty_verbatim = quote! { #unsupported_self_ty };
            panic!("`impl {self_ty_verbatim}` unsupported: self type can only be syn::Type::Path")
        }
    }
}

fn replace_last_path_segment(p: &mut syn::Path) {
    let orig_ident = &p.segments.last().unwrap().ident;
    let archived_name = format!("Archived{orig_ident}");
    let archived_ident = syn::Ident::new(&archived_name, orig_ident.span());
    p.segments.last_mut().unwrap().ident = archived_ident;
}

// Augments the where clause of each method with an `archive_method` attribute.
fn augment_methods(augmented_items: &mut [ImplItem]) -> syn::Result<()> {
    for item in augmented_items {
        if let ImplItem::Fn(fn_item) = item {
            augment_method(fn_item)?;
        }
    }
    Ok(())
}

fn augment_method(fn_item: &mut ImplItemFn) -> syn::Result<()> {
    let mut args_builder = ArgumentsBuilder::default();
    for attr in &fn_item.attrs {
        if !attr.path().is_ident("archive_method") {
            continue;
        }

        match &attr.meta {
            Meta::List(meta_list) => {
                args_builder.try_add_metas_token_stream(meta_list.tokens.clone().into())?;
            }
            unsupported_meta => {
                let meta_verbatim = quote! { #unsupported_meta };
                panic!(
                    "Unsupported meta `{meta_verbatim}`: meta can only be structure list `archive_method(...)`"
                );
            }
        }
    }
    let args = args_builder.build();
    let method_where = &mut fn_item.sig.generics.where_clause;
    transform_where_clause(&args.transform_params, method_where);
    add_bounds_to_where_clause(args.add_bounds, method_where);
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
    additional_bounds: Vec<WherePredicate>,
    clause: &mut Option<WhereClause>,
) {
    if let Some(clause) = clause {
        clause.predicates.extend(additional_bounds.into_iter());
    } else if !additional_bounds.is_empty() {
        *clause = Some(parse_quote! { where #(#additional_bounds),* });
    }
}

fn parse_argument_metas(args: TokenStream, arg_lists: &mut Vec<Meta>) -> syn::Result<()> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    arg_lists.extend(parser.parse(args)?.into_iter());
    Ok(())
}

fn parse_transform_bounds(meta: &Meta, transform_params: &mut HashSet<Ident>) -> syn::Result<()> {
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
