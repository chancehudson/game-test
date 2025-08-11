use proc_macro::TokenStream;

#[proc_macro_derive(Entity)]
pub fn derive_entity_fn(_item: TokenStream) -> TokenStream {}
