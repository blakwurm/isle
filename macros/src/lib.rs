use proc_macro::{TokenStream};
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Event)]
pub fn event_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let expanded = quote! {
    impl #impl_generics isle_traits::event::Event for #name #ty_generics #where_clause {
      fn as_any(&self) -> &dyn std::any::Any {
        self
      }
    }
  };

  TokenStream::from(expanded)
}

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let mut state_queue_filed = None;

  if let Data::Struct(ref data_struct) = input.data {
    if let Fields::Named(ref fields_named) = data_struct.fields {
      state_queue_filed = Some(
        fields_named.named.iter().find(|f| f.ident == Some(Ident::new("staged")))
      )
    }
  }

  let expanded = quote! {
    impl #impl_generics isle_traits::Anyable for #name #ty_generics #where_clause {
      fn as_any(&self) -> &dyn std::any::Any {
        self
      }
      fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
      }
    }

    impl #impl_generics isle_traits::StateQueue for #name #ty_generics #where_clause {

    }
  };

  TokenStream::from(expanded)
}