use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Task, attributes(title))]
pub fn derive_task(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_task_derive(&ast)
}

fn impl_task_derive(ast: &DeriveInput) -> TokenStream {
    eprintln!("ast: {:#?}", ast);
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
    let arc_lock_fields: Vec<TokenStream> = vec![quote! {}.into()];

    let title = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("title") {
            attr.parse_args::<syn::LitStr>().ok()
        } else {
            None
        }
    });

    let title_impl = match title {
        Some(title) => {
            let title = title.value();
            quote! {
                impl #impl_generics Task for #name #ty_generics #where_clause {
                    fn title() -> &'static str {
                        #title
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

    quote! {
        //impl #impl_generics #name #ty_generics #where_clause {
        //    pub fn new() -> Self {
        //        Self {
        //            #(#arc_lock_fields),*
        //        }
        //    }
        //}
        impl #impl_generics UniqueId for #name #ty_generics #where_clause {}
        #title_impl
    }
    .into()
}

#[proc_macro_derive(Salt, attributes(title))]
pub fn derive_salt(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_salt_derive(&ast)
}

fn impl_salt_derive(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Salt for #name #ty_generics #where_clause {}
    }
    .into()
}
