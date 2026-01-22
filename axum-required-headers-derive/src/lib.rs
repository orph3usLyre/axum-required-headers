use proc_macro::TokenStream;
use proc_macro_crate::FoundCrate;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitStr, parse_macro_input};

const ATTRIBUTE_IDENT: &str = "header";

/// Derive macro for individual header types.
///
/// Automatically implements both `RequiredHeader` and `OptionalHeader`
/// for a type, allowing it to be used with either `Required<T>` or `Optional<T>`.
///
/// # Attributes
/// - `#[header("header-name")]` - Specifies the header name to extract
///
/// See `axum-required-headers` for examples
///
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
/// - `#[header("header-name")]` - Marks a field as a header
/// - Fields with `Option<T>` are considered optional headers (will not error if not found in a
///   handler)
///
/// See `axum-required-headers` for examples
///
#[proc_macro_derive(Headers, attributes(header))]
pub fn derive_headers(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_headers_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_header_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Find the #[header("name")] attribute on the struct itself
    let header_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(ATTRIBUTE_IDENT))
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

fn derive_headers_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    // build `impl<S, ...>` generics
    let s_ident = syn::Ident::new("S", name.span());
    let mut impl_generics_with_s = input.generics.clone();
    impl_generics_with_s.params.insert(
        0,
        syn::GenericParam::Type(syn::TypeParam::from(s_ident.clone())),
    );
    let (impl_generics_with_s, _, _) = impl_generics_with_s.split_for_impl();

    // extend where-clause with `S: Send + Sync`
    let mut where_clause_with_s = where_clause.cloned();
    {
        let wc = where_clause_with_s.get_or_insert_with(|| syn::WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        wc.predicates
            .push(syn::parse_quote!(#s_ident: ::std::marker::Send + ::std::marker::Sync));
    }

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
            .find(|attr| attr.path().is_ident(ATTRIBUTE_IDENT))
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    field,
                    "Missing #[header(\"header-name\")] attribute on field",
                )
            })?;

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
    let axum_crate = get_crate("axum")?;
    let http_crate = get_crate("http")?;

    let expanded = quote! {
        impl #impl_generics_with_s ::#axum_crate::extract::FromRequestParts<#s_ident>
            for #name #ty_generics
            #where_clause_with_s
        {
            type Rejection = ::axum_required_headers::HeaderError;

            async fn from_request_parts(
                parts: &mut ::#http_crate::request::Parts,
                _state: &#s_ident,
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
            if let Some(last_segment) = type_path.path.segments.last()
                && last_segment.ident == "Option"
            {
                return true;
            }
            false
        }
        _ => false,
    }
}

fn get_crate(crate_name: &str) -> syn::Result<proc_macro2::TokenStream> {
    let Ok(found_crate) = proc_macro_crate::crate_name(crate_name) else {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("Expected to find the '{crate_name}' dependency but none was found"),
        ));
    };

    let tokens = match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    };
    Ok(tokens)
}
