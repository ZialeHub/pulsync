use std::collections::HashSet;

use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, DeriveInput, LitStr};

mod salt;
mod task;

/// Implementation for a Task to run with the scheduler
///
/// The derive will generate a `new` method to create a task,
/// and wrap each fields inside `TaskState<T>`.
///
/// The `title` attribute, allow the user to choose a unique title.
///
/// # Example
/// ```rust,ignore
/// #[derive(Task)]
/// #[title(format = "Task {self.id} {self.name}")]
/// struct Task {
///   id: TaskState<u32>,
///   name: TaskState<String>,
///   ...
/// }
///
/// struct SecondTask {
///  id: TaskState<u32>,
///  ...
/// }
/// ```
///
/// In this example, the task implementation will look like this:
/// ```rust,ignore
/// impl Task {
///     pub fn new(id: u32, name: String) -> Self {
///         Self {
///             id: TaskState::new(id),
///             name: TaskState::new(name),
///         }
///     }
/// }
/// impl UniqueId for Task {}
/// impl Task for Task {
///    fn title(&self) -> String {
///         format!("Task {} {}", &*self.id.read().unwrap(), &*self.name.read().unwrap())
///    }
/// }
///
/// impl SecondTask {
///     pub fn new(id: u32) -> Self {
///         Self {
///             id: TaskState::new(id),
///         }
///     }
/// }
/// impl UniqueId for SecondTask {}
/// impl Task for SecondTask {}
/// ```
#[proc_macro_derive(Task, attributes(title))]
pub fn derive_task(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    task::impl_task_derive(&ast)
}

/// Derive macro used to implement the `Salt` trait
///
/// If the user need to use multiple tasks of the same type,
/// the `salt` attribute can be used to define a unique salt.
///
/// The salt attribute is a string that can contain placeholders
/// for the fields of the struct. The placeholders must be in the
/// form of `{self.field_name}`.
///
/// # Example
/// ```rust,ignore
/// #[derive(Salt)]
/// #[salt(format = "Task {self.id} {self.name}")]
/// struct Task {
///    id: TaskState<u32>,
///    name: TaskState<String>,
///    ...
/// }
///
/// struct SecondTask {
///    id: TaskState<u32>,
///    ...
/// }
/// ```
///
/// In this example, the salt implementation will look like this:
/// ```rust,ignore
/// impl Salt for Task {
///     fn salt(&self) -> String {
///         format!("Task {} {}", &*self.id.read().unwrap(), &*self.name.read().unwrap())
///     }
/// }
///
/// impl Salt for SecondTask {}
/// ```
#[proc_macro_derive(Salt, attributes(salt))]
pub fn derive_salt(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    salt::impl_salt_derive(&ast)
}

#[derive(Debug)]
pub(crate) struct VariadicStringParams {
    format_str: String,
    field_names: HashSet<String>,
}
impl VariadicStringParams {
    /// Parse and collect the field name without the 'self.' prefix
    ///
    /// Example:
    /// ```ignore
    /// // For the following field
    /// "{self.counter}"
    /// "counter" will be extracted
    /// ```
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

    /// Parse and collect the fields names without the 'self.' prefix
    ///
    /// Example:
    /// ```ignore
    /// // For the following string
    /// "The title is {self.title} the task is {self.status}"
    ///
    /// "title" and "status" will be extracted
    /// ```
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

impl Parse for VariadicStringParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let format_lit = input.parse::<LitStr>()?;
        let format_str = format_lit.value();
        let field_names = Self::parse_format_string(&format_str);

        Ok(VariadicStringParams {
            format_str,
            field_names,
        })
    }
}
