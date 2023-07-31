extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, ItemImpl, Meta, MetaList, Token,
    Type, WhereClause, WherePredicate,
};

#[proc_macro_attribute]
pub fn archive_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let archive_impl_bounds = if attr.is_empty() {
        Vec::new()
    } else {
        let meta = parse_macro_input!(attr as Meta);
        match meta {
            Meta::List(meta_list) => match parse_bounds(meta_list) {
                Ok(b) => b,
                Err(e) => return e.to_compile_error().into(),
            },
            unsupported_meta => {
                let meta_verbatim = quote! { #unsupported_meta };
                panic!("Unsupported meta `{meta_verbatim}`: meta can only be structure list `bound(...)`")
            }
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

    let archived_where_clause = combine_bounds(orig_where_clause.cloned(), archive_impl_bounds);

    // TODO: is there a way to avoid duplication here?
    let impl_items = &orig_impl.items;
    if let Some((_, trait_path, _)) = &orig_impl.trait_ {
        quote! {
            #orig_impl

            impl #impl_generics #trait_path for #archived_path #archived_where_clause {
                #(#impl_items)*
            }
        }
    } else {
        quote! {
            #orig_impl

            impl #impl_generics #archived_path #archived_where_clause {
                #(#impl_items)*
            }
        }
    }
    .into()
}

fn parse_bounds(meta_list: MetaList) -> syn::Result<Vec<WherePredicate>> {
    if meta_list.path.is_ident("bounds") {
        let parser = Punctuated::<WherePredicate, Token![,]>::parse_terminated;
        Ok(parser.parse(meta_list.tokens.into())?.into_iter().collect())
    } else {
        panic!("Unsupported meta: {}", meta_list.path.get_ident().unwrap());
    }
}

fn replace_last_path_segment(p: &syn::Path) -> syn::Path {
    let orig_ident = &p.segments.last().unwrap().ident;
    let archived_name = format!("Archived{orig_ident}");
    let archived_ident = syn::Ident::new(&archived_name, orig_ident.span());
    let mut archived_path = p.clone();
    archived_path.segments.last_mut().unwrap().ident = archived_ident;
    archived_path
}

fn combine_bounds(
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
