use proc_macro2::TokenStream;
use quote::quote;

pub fn allow_deprecated(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    if should_allow_deprecated(input)? {
        Ok(quote! { #[allow(deprecated)]})
    } else {
        Ok(TokenStream::default())
    }
}

/// Determine if an `#[allow(deprecated)]` should be added to the derived impl.
///
/// This should happen if the derive input or a variant of the enum (if derive input is an enum)
/// has on of:
///   - `#[deprecated]`
///   - `#[allow(deprecated)]`
fn should_allow_deprecated(input: &syn::DeriveInput) -> syn::Result<bool> {
    if contains_deprecated_attrs(&input.attrs)? {
        return Ok(true);
    }
    if let syn::Data::Enum(data_enum) = &input.data {
        for variant in &data_enum.variants {
            if contains_deprecated_attrs(&variant.attrs)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Check whether a set of attributes contains one of:
///   - `#[deprecated]`
///   - `#[allow(deprecated)]`
fn contains_deprecated_attrs(attrs: &[syn::Attribute]) -> syn::Result<bool> {
    for attr in attrs {
        if let syn::Meta::Path(path) = &attr.meta {
            if path.is_ident("deprecated") {
                return Ok(true);
            }
        }
        if let syn::Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("allow") {
                let mut deprecated_allowed = false;
                meta_list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("deprecated") {
                        deprecated_allowed = true;
                    }
                    Ok(())
                })?;
                if deprecated_allowed {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
