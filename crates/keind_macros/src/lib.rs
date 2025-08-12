mod engine_entity;
mod entity_system;

use proc_macro::TokenStream;

/// A wrapper enum around all potential types of
/// entites in the game. This allows polymorphism
/// in the engine.
#[proc_macro_derive(EngineEntity)]
pub fn derive_engine_entity(input: TokenStream) -> TokenStream {
    engine_entity::derive_engine_entity(input)
}

#[proc_macro_derive(EntitySystem)]
pub fn derive_entity_system(input: TokenStream) -> TokenStream {
    entity_system::derive_entity_system(input)
}
