use std::collections::HashSet;

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, token::Comma, DeriveInput, Field,
    Ident, LitStr, Type,
};

/// Implementation for a Task to run with the scheduler
///
/// The derive will generate a `new` method to create a task,
/// and wrap each fields inside Arc<RwLock<>>.
///
/// The `title` attribute, allow the user to choose a unique title.
/// If the title attribute is not passed, the user can implement the
#[proc_macro_derive(Task, attributes(title))]
pub fn derive_task(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_task_derive(&ast)
}

fn impl_task_derive(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Verify that it's a struct
    let syn::Data::Struct(data) = &ast.data else {
        return quote::quote! {
            compile_error!("Only structs are supported");
        }
        .into();
    };

    // Verify that it has named fields
    let syn::Fields::Named(syn::FieldsNamed { named, .. }) = &data.fields else {
        return quote::quote! {
            compile_error!("Only named fields supported");
        }
        .into();
    };
    // Verify that each fields are Arc<RwLock<>> type
    // and collect the field names and types
    let arc_lock_fields: Vec<(Ident, Type)> = match get_fields_name_and_type(named) {
        Ok(arc_lock_fields) => arc_lock_fields,
        Err(err) => return err,
    };

    let title = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("title") {
            attr.parse_args::<FormatedString>().ok()
        } else {
            None
        }
    });

    let title_impl = match title {
        Some(title) => {
            let title_format = title;
            let mut format_str = title_format.format_str.clone();
            let mut format_args = Vec::new();
            for field_name in &title_format.field_names {
                let pattern = format!("{{self.{}}}", field_name);
                format_str = format_str.replace(&pattern, "{}");
                let field_ident = syn::Ident::new(field_name, Span::call_site().into());
                format_args.push(quote! { &*self.#field_ident.read().unwrap() });
            }
            let format_str_lit = syn::LitStr::new(&format_str, Span::call_site().into());
            quote! {
                impl #impl_generics Task for #name #ty_generics #where_clause {
                    fn title(&self) -> String {
                        format!(#format_str_lit, #(#format_args),*)
                    }
                }
            }
        }
        None => {
            quote! {
                impl #impl_generics Task for #name #ty_generics #where_clause {}
            }
        }
    };

    let field_names = arc_lock_fields
        .iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    let field_types = arc_lock_fields.iter().map(|(_, ty)| ty).collect::<Vec<_>>();

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(
                        #field_names: std::sync::Arc::new(std::sync::RwLock::new(#field_names))
                    ),*
                }
            }
        }
        impl #impl_generics UniqueId for #name #ty_generics #where_clause {}
        #title_impl
    }
    .into()
}

fn get_fields_name_and_type(
    fields: &Punctuated<Field, Comma>,
) -> Result<Vec<(Ident, Type)>, TokenStream> {
    let mut arc_lock_fields: Vec<(Ident, Type)> = Vec::new();
    for field in fields {
        let Some(field_name) = &field.ident else {
            return Err(quote::quote! {
                compile_error!("Only named fields supported");
            }
            .into());
        };
        let syn::Type::Path(type_path) = &field.ty else {
            return Err(quote::quote! {
                compile_error!("Only path types supported");
            }
            .into());
        };
        let Some(segment) = &type_path.path.segments.first() else {
            return Err(quote::quote! {
                compile_error!("No segments found");
            }
            .into());
        };
        if segment.ident != "Arc" {
            return Err(quote::quote! {
                compile_error!("State fields must be Arc<RwLock<T>>");
            }
            .into());
        }
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return Err(quote::quote! {
                compile_error!("No angle bracketed arguments found");
            }
            .into());
        };
        let Some(syn::GenericArgument::Type(syn::Type::Path(type_path))) = args.args.first() else {
            return Err(quote::quote! {
                compile_error!("No type path found");
            }
            .into());
        };
        let Some(segment) = &type_path.path.segments.first() else {
            return Err(quote::quote! {
                compile_error!("No segments found");
            }
            .into());
        };
        if segment.ident != "RwLock" {
            return Err(quote::quote! {
                compile_error!("State fields must be Arc<RwLock<T>>");
            }
            .into());
        }
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return Err(quote::quote! {
                compile_error!("No angle bracketed arguments found");
            }
            .into());
        };
        let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() else {
            return Err(quote::quote! {
                compile_error!("No type path found");
            }
            .into());
        };
        arc_lock_fields.push((field_name.clone(), inner_type.clone()));
    }
    Ok(arc_lock_fields)
}

#[derive(Debug)]
struct FormatedString {
    format_str: String,
    field_names: HashSet<String>,
}
impl FormatedString {
    fn extract_field_name(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
        let mut field = String::new();

        // Skip "self." prefix if present
        if chars
            .clone()
            .collect::<String>()
            .as_str()
            .starts_with("self.")
        {
            for _ in 0..5 {
                chars.next();
            }
        }

        // Collect characters until closing brace
        while let Some(&c) = chars.peek() {
            if c == '}' {
                chars.next();
                return Some(field);
            }
            field.push(chars.next().unwrap());
        }

        None // Missing closing brace
    }

    fn parse_format_string(format_str: &str) -> HashSet<String> {
        let mut field_names = HashSet::new();
        let mut chars = format_str.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if let Some(field) = Self::extract_field_name(&mut chars) {
                    if !field.is_empty() {
                        field_names.insert(field);
                    }
                }
            }
        }

        field_names
    }
}

impl Parse for FormatedString {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let format_lit = input.parse::<LitStr>()?;
        let format_str = format_lit.value();
        let field_names = Self::parse_format_string(&format_str);

        Ok(FormatedString {
            format_str,
            field_names,
        })
    }
}

#[proc_macro_derive(Salt, attributes(salt))]
pub fn derive_salt(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_salt_derive(&ast)
}

fn impl_salt_derive(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let salt = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("salt") {
            attr.parse_args::<FormatedString>().ok()
        } else {
            None
        }
    });

    let salt_impl = match salt {
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
    };

    quote! { #salt_impl }.into()
}
