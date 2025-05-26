use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

use crate::VariadicStringParams;

pub(crate) fn impl_salt_derive(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let salt = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("salt") {
            attr.parse_args::<VariadicStringParams>().ok()
        } else {
            None
        }
    });

    match salt {
        Some(salt) => {
            let salt_format = salt;
            let mut format_str = salt_format.format_str.clone();
            let mut format_args = Vec::new();
            for field_name in &salt_format.field_names {
                let pattern = format!("{{self.{}}}", field_name);
                format_str = format_str.replace(&pattern, "{}");
                let field_ident = syn::Ident::new(field_name, Span::call_site().into());
                format_args.push(quote! { &*self.#field_ident.read().unwrap() });
            }
            let format_str_lit = syn::LitStr::new(&format_str, Span::call_site().into());
            quote! {
                impl #impl_generics Salt for #name #ty_generics #where_clause {
                    fn salt(&self) -> String {
                        format!(#format_str_lit, #(#format_args),*)
                    }
                }
            }
        }
        None => {
            quote! {
                impl #impl_generics Salt for #name #ty_generics #where_clause {}
            }
        }
    }
    .into()
}
