use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Implements the message trait for some message.
#[proc_macro_derive(Message)]
pub fn derive_answer_fn(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(item).unwrap();

    let name = ast.ident;
    let out = quote! {
        impl Message for #name {
            // Currently, nothing needs to be done here.
            // It is useful to have in the future for backwards compatability.
        }
    };
    out.into()
}
