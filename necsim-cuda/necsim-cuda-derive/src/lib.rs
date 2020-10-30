#![deny(clippy::pedantic)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn;

enum CudaBorrowType {
    BoxedSlice(proc_macro2::TokenStream),
    Recursive(proc_macro2::Ident),
}

#[allow(clippy::too_many_lines)] // TODO: Refactor
fn impl_cuda_borrow(ast: &syn::DeriveInput) -> TokenStream {
    let attrs = &ast.attrs;
    let vis = &ast.vis;
    let name = &ast.ident;
    let generics = &ast.generics;

    let name_cuda = format_ident!("{}CudaRepresentation", name);

    let (mut fields, semi) = if let syn::Data::Struct(s) = &ast.data {
        (s.fields.clone(), s.semi_token)
    } else {
        panic!("You can only derive `CudaBorrow` on structs for now.");
    };

    let mut alloc_combined_type: proc_macro2::TokenStream = quote! { necsim_cuda::NullCudaAlloc };
    let mut field_declare_streams: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut field_borrow_streams: Vec<proc_macro2::TokenStream> = Vec::new();

    match fields {
        syn::Fields::Named(syn::FieldsNamed {
            named: ref mut fields,
            ..
        })
        | syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: ref mut fields,
            ..
        }) => {
            for (i, mut field) in fields.iter_mut().enumerate() {
                let mut recursive_cuda_borrow: Option<CudaBorrowType> = None;
                let mut field_cuda_repr_ty: Option<syn::Type> = None;

                field.attrs.retain(|attr| match attr.path.get_ident() {
                    Some(ident)
                        if recursive_cuda_borrow.is_none()
                            && format!("{}", ident) == "repr_cuda" =>
                    {
                        let attribute_str = format!("{}", attr.tokens);

                        if let Some(slice_type) = attribute_str
                            .strip_prefix("(Box < [")
                            .and_then(|rest| rest.strip_suffix("] >)"))
                        {
                            let slice_type = slice_type.parse().unwrap();

                            field_cuda_repr_ty = Some(
                                syn::parse_quote! { rustacuda_core::DevicePointer<#slice_type> },
                            );

                            recursive_cuda_borrow = Some(CudaBorrowType::BoxedSlice(slice_type));
                        } else if let Some(struct_type) = attribute_str
                            .strip_prefix("(")
                            .and_then(|rest| rest.strip_suffix(")"))
                        {
                            let field_type = format_ident!("{}", struct_type);

                            field_cuda_repr_ty = Some(
                                syn::parse_quote! { <#field_type as necsim_cuda::CudaBorrow>::CudaRepresentation },
                            );

                            recursive_cuda_borrow = Some(CudaBorrowType::Recursive(field_type));
                        }

                        false
                    }
                    _ => true,
                });

                if let Some(ty) = field_cuda_repr_ty {
                    field.ty = ty;
                }

                let field_accessor = match &field.ident {
                    Some(ident) => ident.clone(),
                    None => format_ident!("{}", i),
                };
                let field_ptr = match &field.ident {
                    Some(ident) => format_ident!("{}_ptr", ident),
                    None => format_ident!("{}_ptr", i),
                };
                let optional_field_ident = field.ident.as_ref().map(|ident| quote! { #ident: });

                match recursive_cuda_borrow {
                    Some(CudaBorrowType::BoxedSlice(slice_type)) => {
                        alloc_combined_type = quote! {
                            necsim_cuda::ScopedCudaAlloc<
                                necsim_cuda::CudaDropWrapper<
                                    rustacuda::memory::DeviceBuffer<
                                        #slice_type
                                    >
                                >,
                                #alloc_combined_type
                            >
                        };

                        field_declare_streams.push(quote! {
                            let (#field_ptr, alloc_front) = {
                                let mut device_buffer = necsim_cuda::CudaDropWrapper::from(
                                    rustacuda::memory::DeviceBuffer::from_slice(&self.#field_accessor)?
                                );

                                (device_buffer.as_device_ptr(), necsim_cuda::ScopedCudaAlloc::new(
                                    device_buffer, alloc_front
                                ))
                            };
                        });

                        field_borrow_streams.push(quote! {
                            #optional_field_ident #field_ptr,
                        });
                    }
                    Some(CudaBorrowType::Recursive(field_type)) => {
                        alloc_combined_type = quote! {
                            necsim_cuda::ScopedCudaAlloc<
                                <#field_type as necsim_cuda::CudaBorrow>::CudaAllocation,
                                #alloc_combined_type
                            >
                        };

                        field_declare_streams.push(quote! {
                            let (#field_ptr, alloc_front) = self.#field_accessor.borrow(alloc_front)?;
                        });

                        field_borrow_streams.push(quote! {
                            #optional_field_ident #field_ptr,
                        });
                    }
                    None => field_borrow_streams.push(quote! {
                        #optional_field_ident self.#field_accessor,
                    }),
                }
            }
        }
        syn::Fields::Unit => (),
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let borrow_impl = match fields {
        syn::Fields::Named(_) => quote! {
            #name_cuda {
                #(#field_borrow_streams)*
            }
        },
        syn::Fields::Unnamed(_) => quote! {
            #name_cuda (
                #(#field_borrow_streams)*
            )
        },
        syn::Fields::Unit => quote! { #name_cuda },
    };

    (quote! {
        //#[derive(DeviceCopy)]
        #(#attrs)* #vis struct #name_cuda #generics #fields #semi

        // I would prefer #[derive(DeviceCopy)] on #name_cuda but this can interfer with type parameters
        unsafe impl #impl_generics rustacuda_core::DeviceCopy for #name_cuda #ty_generics #where_clause {}

        impl #impl_generics necsim_cuda::CudaBorrow for #name #ty_generics #where_clause {
            type CudaRepresentation = #name_cuda #ty_generics;
            type CudaAllocation = #alloc_combined_type;

            fn borrow<A: necsim_cuda::CudaAlloc>(
                &self, alloc: A
            ) -> rustacuda::error::CudaResult<(
                Self::CudaRepresentation,
                necsim_cuda::ScopedCudaAlloc<Self::CudaAllocation, A>
            )> {
                let alloc_front = necsim_cuda::NullCudaAlloc;
                let alloc_tail = alloc;

                #(#field_declare_streams)*

                let borrow = #borrow_impl;

                Ok((borrow, necsim_cuda::ScopedCudaAlloc::new(alloc_front, alloc_tail)))
            }
        }
    }).into()
}

#[proc_macro_derive(CudaBorrow, attributes(repr_cuda))]
pub fn cuda_borrow_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_cuda_borrow(&ast)
}
