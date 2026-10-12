use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result, parse_macro_input};

#[proc_macro_attribute]
pub fn bytes(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);

    if let Err(err) = check_repr(&item) {
        return err.into_compile_error().into();
    }

    quote! {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq,
            ::zerocopy::TryFromBytes, ::zerocopy::IntoBytes, ::zerocopy::Immutable,
            ::zerocopy::KnownLayout, ::zerocopy::Unaligned,
        )]
        #item
    }
    .into()
}

#[proc_macro_derive(ToBytes)]
pub fn derive_to_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Err(err) = check_repr(&input) {
        return err.into_compile_error().into();
    }

    let name = &input.ident;
    let size = quote!(::core::mem::size_of::<Self>());
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::aurora_protocol::traits::ToBytes for #name #ty_generics
            #where_clause
        {
            const SIZE: usize = #size;
            type Output = [u8; #size];

            fn to_bytes(&self) -> Self::Output {
                let mut buf = [0u8; #size];
                buf.copy_from_slice(::zerocopy::IntoBytes::as_bytes(self));
                buf
            }
        }
    }
    .into()
}

#[proc_macro_derive(FromBytes)]
pub fn derive_from_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Err(err) = check_repr(&input) {
        return err.into_compile_error().into();
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::aurora_protocol::traits::FromBytes for #name #ty_generics
            #where_clause
        {
            const SIZE: usize = ::core::mem::size_of::<Self>();

            fn from_bytes(raw: &[u8]) -> ::core::option::Option<Self> {
                <Self as ::zerocopy::TryFromBytes>::try_read_from_bytes(raw).ok()
            }
        }
    }
    .into()
}

fn check_repr(input: &DeriveInput) -> Result<()> {
    let mut has_c = false;
    let mut has_u8 = false;

    for attr in input.attrs.iter().filter(|a| a.path().is_ident("repr")) {
        let _ = attr.parse_nested_meta(|meta| {
            has_c |= meta.path.is_ident("C");
            has_u8 |= meta.path.is_ident("u8");
            Ok(())
        });
    }

    let (is_valid, message) = match input.data {
        Data::Enum(_) => (has_u8, "enums must be #[repr(u8)]"),
        _ => (has_c, "structs must be #[repr(C)]"),
    };

    if is_valid { Ok(()) } else { Err(Error::new_spanned(input, message)) }
}
