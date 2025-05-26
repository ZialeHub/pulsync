use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Field, Ident, Type};

use crate::VariadicStringParams;

pub(crate) fn impl_task_derive(ast: &DeriveInput) -> TokenStream {
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
    // Verify that each fields are TaskState<T> type
    // and collect the field names and types
    let arc_lock_fields: Vec<(Ident, Type)> = match named
        .into_iter()
        .map(|field| get_field_name_and_type(field))
        .collect::<Result<Vec<(Ident, Type)>, TokenStream>>()
    {
        Ok(arc_lock_fields) => arc_lock_fields,
        Err(err) => return err,
    };

    let title = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("title") {
            attr.parse_args::<VariadicStringParams>().ok()
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
                        #field_names: TaskState::new(#field_names)
                    ),*
                }
            }
        }
        impl #impl_generics UniqueId for #name #ty_generics #where_clause {}
        #title_impl
    }
    .into()
}

/// Collect the name and type of the state struct
///
/// This will ignore the TaskState wrapper around it
fn get_field_name_and_type(field: &Field) -> Result<(Ident, Type), TokenStream> {
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
    if segment.ident != "TaskState" {
        return Err(quote::quote! {
            compile_error!("State fields must be TaskState<T>");
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
    Ok((field_name.clone(), inner_type.clone()))
}
