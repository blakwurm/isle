use proc_macro::{TokenStream};
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Event)]
pub fn event_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  let expanded = quote! {
    impl isle_traits::event::Event for #name {
      fn as_any(&self) -> &dyn std::any::Any {
        self
      }
    }
  };

  TokenStream::from(expanded)
}