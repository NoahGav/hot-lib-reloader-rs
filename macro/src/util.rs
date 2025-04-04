use proc_macro2::Span;
use std::path::PathBuf;
use syn::{Error, ForeignItemFn, LitStr, Result};

pub fn ident_from_pat(
    pat: &syn::Pat,
    func_name: &proc_macro2::Ident,
    span: proc_macro2::Span,
) -> syn::Result<syn::Ident> {
    match pat {
        syn::Pat::Ident(pat) => Ok(pat.ident.clone()),
        _ => Err(syn::Error::new(
            span,
            format!("generating call for library function: signature of function {func_name} cannot be converted"),
        )),
    }
}

/// Reads the contents of a Rust source file and finds the top-level functions that have
/// - visibility public
/// - #[no_mangle] attribute
/// It converts these functions into a [syn::ForeignItemFn] so that those can
/// serve as lib function declarations of the lib reloader.
pub fn read_functions_from_file(
    file_name: LitStr,
    ignore_no_mangle: bool,
) -> Result<Vec<(ForeignItemFn, Span)>> {
    let span = file_name.span();
    let path: PathBuf = file_name.value().into();

    if !path.exists() {
        return Err(Error::new(span, format!("Could not find Rust source file {path:?}. Please make sure that you specify the file path from the project root directory. Please not that this has been changed in hot-lib-reloader v0.5 -> v0.6. See https://github.com/rksm/hot-lib-reloader-rs/issues/13.")));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|err| Error::new(span, format!("Error reading file {path:?}: {err}")))?;

    let ast = syn::parse_file(&content)?;

    let mut functions = Vec::new();

    for item in ast.items {
        match item {
            syn::Item::Fn(fun) => {
                match fun.vis {
                    syn::Visibility::Public(_) => {}
                    _ => continue,
                };

                // we can optionally assume that the function will be unmangled
                // by other means than a direct attribute
                if !ignore_no_mangle {
                    let has_no_mangle = fun.attrs.iter().any(|attr| {
                        // // Check for #[no_mangle]
                        // if attr.path.is_ident("no_mangle") {
                        //     return true;
                        // }
                        todo!("HELLO?");
                        // Check for #[unsafe(no_mangle)]
                        if attr.path.is_ident("unsafe") {
                            if let Ok(nested) = attr.parse_args::<syn::Ident>() {
                                return nested == "no_mangle";
                            }
                        }
                        false
                    });

                    if !has_no_mangle {
                        continue;
                    };
                }

                let fun = ForeignItemFn {
                    attrs: Vec::new(),
                    vis: fun.vis,
                    sig: fun.sig,
                    semi_token: syn::token::Semi(span),
                };

                functions.push((fun, span));
            }
            _ => continue,
        }
    }

    Ok(functions)
}
