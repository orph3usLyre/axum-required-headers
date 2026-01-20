use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

/// Derive macro for individual header types.
///
/// Automatically implements both `RequiredHeader` and `OptionalHeader`
/// for a type, allowing it to be used with either `Required<T>` or `Optional<T>`.
///
/// # Attributes
///
/// - `#[header("header-name")]` - Specifies the header name to extract
///
/// # Examples
///
/// ```ignore
/// #[derive(Header)]
/// #[header("x-user-id")]
/// struct UserId(String);
///
/// // Now you can use:
/// async fn handler(
///     Required(user_id): Required<UserId>,
///     Optional(token): Optional<UserId>,
/// ) { }
/// ```
#[proc_macro_derive(Header, attributes(header))]
pub fn derive_header(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_header_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive macro for header extraction.
///
/// # Attributes
///
/// - `#[header("header-name")]` - Marks a field as a required header
/// - `#[header("header-name")]` - Option<T> - Marks a field as optional
///
/// # Examples
///
/// ```ignore
/// #[derive(Headers)]
/// struct MyHeaders {
///     #[header("x-user-id")]
///     user_id: String,
///
///     #[header("x-tenant-id")]
///     tenant_id: Option<String>,
/// }
/// ```
#[proc_macro_derive(Headers, attributes(header))]
pub fn derive_headers(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_headers_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_headers_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Headers can only be derived for structs",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "Headers only supports named fields",
        ));
    };

    let mut field_parsers = Vec::new();
    let mut field_names = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        field_names.push(field_name);

        // Find #[header(...)] attribute
        let header_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("header"))
            .ok_or_else(|| syn::Error::new_spanned(field, "Missing #[header(...)] attribute"))?;

        // Parse the attribute
        let header_name = parse_header_attr(header_attr)?;
        let is_optional = is_option_type(field_type);

        if is_optional {
            // Optional header
            field_parsers.push(quote! {
                let #field_name: #field_type = {
                    parts.headers
                        .get(#header_name)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse().ok())
                };
            });
        } else {
            // Required header
            field_parsers.push(quote! {
                let #field_name: #field_type = {
                    parts.headers
                        .get(#header_name)
                        .ok_or_else(|| ::axum_required_headers::HeaderError::Missing(#header_name))?
                        .to_str()
                        .map_err(|_| ::axum_required_headers::HeaderError::InvalidValue(#header_name))?
                        .parse()
                        .map_err(|_| ::axum_required_headers::HeaderError::Parse(#header_name))?
                };
            });
        }
    }

    let field_constructions = field_names.iter().map(|name| quote! { #name });

    let expanded = quote! {
        impl #impl_generics ::axum::extract::FromRequestParts<()> for #name #ty_generics #where_clause {
            type Rejection = ::axum_required_headers::HeaderError;

            async fn from_request_parts(
                parts: &mut ::http::request::Parts,
                _state: &(),
            ) -> ::std::result::Result<Self, Self::Rejection> {
                #(#field_parsers)*

                Ok(Self {
                    #(#field_constructions),*
                })
            }
        }
    };

    Ok(expanded)
}

fn derive_header_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Find the #[header("name")] attribute on the struct itself
    let header_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("header"))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &input,
                "Missing #[header(\"header-name\")] attribute on struct",
            )
        })?;

    let header_name = parse_header_attr(header_attr)?;

    let expanded = quote! {
        // Implement RequiredHeader
        impl #impl_generics ::axum_required_headers::RequiredHeader for #name #ty_generics #where_clause {
            const HEADER_NAME: &'static str = #header_name;
        }

        // Implement OptionalHeader
        impl #impl_generics ::axum_required_headers::OptionalHeader for #name #ty_generics #where_clause {
            const HEADER_NAME: &'static str = #header_name;
        }
    };

    Ok(expanded)
}

fn parse_header_attr(attr: &syn::Attribute) -> syn::Result<String> {
    let lit: LitStr = attr.parse_args()?;
    let header_name = lit.value();

    if header_name.is_empty() {
        return Err(syn::Error::new_spanned(attr, "header name cannot be empty"));
    }

    Ok(header_name)
}

/// Helper function to detect if a type is `Option<T>` or `std::option::Option<T>`
fn is_option_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            // Check if the last segment is "Option"
            if let Some(last_segment) = type_path.path.segments.last() {
                if last_segment.ident == "Option" {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}
