mod engine_entity;

use proc_macro::TokenStream;

/// A wrapper enum around all potential types of
/// entites in the game. This allows polymorphism
/// in the engine.
#[proc_macro_derive(EngineEntity)]
pub fn derive_engine_entity(input: TokenStream) -> TokenStream {
    engine_entity::derive_engine_entity(input)
}
