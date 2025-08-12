use proc_macro::TokenStream;
use proc_macro_crate::FoundCrate;
use proc_macro_crate::crate_name;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::Ident;
use syn::parse_macro_input;

pub fn derive_entity_system(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("EntitySystem can only be derived for enums"),
    };

    let mut variant_names = Vec::new();
    let mut variant_types = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;
        let variant_type = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => panic!("Each variant must have exactly one unnamed field"),
        };
        variant_names.push(variant_name);
        variant_types.push(variant_type);
    }
    let crate_name = match crate_name("keind") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, input.ident.span());
            quote! { ::#ident }
        }
        Err(_) => quote! { ::keind }, // fallback to global path
    };

    let expanded = quote! {
        impl #crate_name::prelude::KPoly for #enum_name {
            fn type_id(&self) -> ::std::any::TypeId {
                match self {
                    #(
                        #enum_name::#variant_names(_) => ::std::any::TypeId::of::<#variant_types>(),
                    )*
                }
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                match self {
                    #(
                        #enum_name::#variant_names(entity) => entity,
                    )*
                }
            }

            fn extract_ref<T: 'static>(&self) -> ::std::option::Option<&T> {
                self.as_any().downcast_ref::<T>()
            }

            fn extract_mut<T: 'static>(&mut self) -> ::std::option::Option<&mut T> {
                match self {
                    #(
                        #enum_name::#variant_names(entity) => {
                            (entity as &mut dyn ::std::any::Any).downcast_mut::<T>()
                        },
                    )*
                }
            }
        }

        #(
            impl ::std::convert::From<#variant_types> for #enum_name {
                fn from(value: #variant_types) -> Self {
                    #enum_name::#variant_names(value)
                }
            }
        )*

        impl<GL> #crate_name::prelude::EEntitySystem<GL> for #enum_name
        where
            GL: #crate_name::prelude::GameLogic,
            #(
                #variant_types: #crate_name::prelude::EEntitySystem<GL>,
            )*
        {
            fn prestep(&self, engine: &#crate_name::prelude::GameEngine<GL>, entity: &<GL as #crate_name::prelude::GameLogic>::Entity) -> bool {
                match self {
                    #(
                        #enum_name::#variant_names(system) => system.prestep(engine, entity),
                    )*
                }
            }

            fn step(
                &self,
                engine: &#crate_name::prelude::GameEngine<GL>,
                entity: &<GL as #crate_name::prelude::GameLogic>::Entity,
                next_entity: &mut GL::Entity
            ) -> ::std::option::Option<Self> {
                match self {
                    #(
                        #enum_name::#variant_names(system) => system.step(engine, entity, next_entity).map(|v| #enum_name::from(v)),
                    )*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
