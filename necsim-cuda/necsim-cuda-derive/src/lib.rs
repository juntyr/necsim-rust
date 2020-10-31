#![deny(clippy::pedantic)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};

enum CudaBorrowType {
    BoxedSlice(proc_macro2::TokenStream),
    Recursive(proc_macro2::Ident),
}

fn get_cuda_repr_ident(rust_repr_ident: &proc_macro2::Ident) -> proc_macro2::Ident {
    format_ident!("{}CudaRepresentation", rust_repr_ident)
}

#[allow(clippy::too_many_lines)] // TODO: Refactor
fn impl_rust_to_cuda(ast: &syn::DeriveInput) -> TokenStream {
    let (mut fields, semi) = if let syn::Data::Struct(s) = &ast.data {
        (s.fields.clone(), s.semi_token)
    } else {
        panic!("You can only derive `RustToCuda` on structs for now.");
    };

    let attrs = &ast.attrs;
    let vis = &ast.vis;
    let name = &ast.ident;
    let generics = &ast.generics;

    let name_cuda = get_cuda_repr_ident(name);

    let mut alloc_combined_type: proc_macro2::TokenStream = quote! { necsim_cuda::NullCudaAlloc };
    let mut field_declare_streams: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut field_borrow_streams: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut field_as_rust_streams: Vec<proc_macro2::TokenStream> = Vec::new();

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

                            field_cuda_repr_ty = Some(syn::parse_quote! {
                                (rustacuda_core::DevicePointer<#slice_type>, usize)
                            });

                            recursive_cuda_borrow = Some(CudaBorrowType::BoxedSlice(slice_type));
                        } else if let Some(struct_type) = attribute_str
                            .strip_prefix("(")
                            .and_then(|rest| rest.strip_suffix(")"))
                        {
                            let field_type = format_ident!("{}", struct_type);

                            field_cuda_repr_ty = Some(syn::parse_quote! {
                                <#field_type as necsim_cuda::RustToCuda>::CudaRepresentation
                            });

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
                                    rustacuda::memory::DeviceBuffer::from_slice(
                                        &self.#field_accessor
                                    )?
                                );

                                (
                                    (device_buffer.as_device_ptr(), device_buffer.len()),
                                    necsim_cuda::ScopedCudaAlloc::new(device_buffer, alloc_front)
                                )
                            };
                        });

                        field_borrow_streams.push(quote! {
                            #optional_field_ident #field_ptr,
                        });

                        field_as_rust_streams.push(quote! {
                            #optional_field_ident unsafe {
                                // This is only safe because we will NOT expose mutability
                                let raw_mut_slice_ptr: *mut #slice_type =
                                    self.#field_accessor.0.as_raw() as *mut #slice_type;
                                let raw_mut_slice_len = self.#field_accessor.1;

                                let raw_slice: &mut [#slice_type] = core::slice::from_raw_parts_mut(
                                    raw_mut_slice_ptr, raw_mut_slice_len
                                );

                                alloc::boxed::Box::from_raw(raw_slice)
                            },
                        });
                    }
                    Some(CudaBorrowType::Recursive(field_type)) => {
                        alloc_combined_type = quote! {
                            necsim_cuda::ScopedCudaAlloc<
                                <#field_type as necsim_cuda::RustToCuda>::CudaAllocation,
                                #alloc_combined_type
                            >
                        };

                        field_declare_streams.push(quote! {
                            let (#field_ptr, alloc_front) = self.#field_accessor.borrow(
                                alloc_front
                            )?;
                        });

                        field_borrow_streams.push(quote! {
                            #optional_field_ident #field_ptr,
                        });

                        field_as_rust_streams.push(quote! {
                            #optional_field_ident self.#field_accessor.as_rust(),
                        });
                    }
                    None => {
                        field_borrow_streams.push(quote! {
                            #optional_field_ident self.#field_accessor,
                        });

                        field_as_rust_streams.push(quote! {
                            #optional_field_ident self.#field_accessor,
                        });
                    }
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

    let as_rust_impl = match fields {
        syn::Fields::Named(_) => quote! {
            #name {
                #(#field_as_rust_streams)*
            }
        },
        syn::Fields::Unnamed(_) => quote! {
            #name (
                #(#field_as_rust_streams)*
            )
        },
        syn::Fields::Unit => quote! { #name },
    };

    (quote! {
        //#[derive(DeviceCopy)]
        #[allow(dead_code)]
        #(#attrs)* #vis struct #name_cuda #generics #fields #semi

        // I would prefer #[derive(DeviceCopy)] on #name_cuda but this can interfer with
        // type parameters
        unsafe impl #impl_generics rustacuda_core::DeviceCopy for #name_cuda #ty_generics
            #where_clause {}

        unsafe impl #impl_generics necsim_cuda::RustToCuda for #name #ty_generics #where_clause {
            type CudaRepresentation = #name_cuda #ty_generics;

            #[cfg(not(target_os = "cuda"))]
            type CudaAllocation = #alloc_combined_type;

            #[cfg(not(target_os = "cuda"))]
            //#[allow(clippy::type_complexity)]
            unsafe fn borrow<A: necsim_cuda::CudaAlloc>(
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

        unsafe impl #impl_generics necsim_cuda::CudaAsRust for #name_cuda #ty_generics
            #where_clause
        {
            type RustRepresentation = #name #ty_generics;

            #[cfg(target_os = "cuda")]
            unsafe fn as_rust(&self) -> #name #ty_generics {
                #as_rust_impl
            }
        }
    })
    .into()
}

#[proc_macro_derive(RustToCuda, attributes(repr_cuda))]
pub fn rust_to_cuda_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_rust_to_cuda(&ast)
}

fn impl_lend_to_cuda(ast: &syn::DeriveInput) -> TokenStream {
    if !matches!(ast.data, syn::Data::Struct(_)) {
        panic!("You can only derive `LendToCuda` on structs for now.");
    };

    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    (quote! {
        #[cfg(not(target_os = "cuda"))]
        unsafe impl #impl_generics necsim_cuda::LendToCuda for #name #ty_generics #where_clause {
            fn lend_to_cuda<
                O,
                F: FnOnce(
                    rustacuda_core::DevicePointer<
                        <Self as necsim_cuda::RustToCuda>::CudaRepresentation
                    >
                ) -> rustacuda::error::CudaResult<O>,
            >(
                &self,
                inner: F,
            ) -> rustacuda::error::CudaResult<O> {
                use necsim_cuda::RustToCuda;

                let (cuda_repr, tail_alloc) = unsafe { self.borrow(necsim_cuda::NullCudaAlloc) }?;

                let mut device_box = necsim_cuda::CudaDropWrapper::from(
                    rustacuda::memory::DeviceBox::new(&cuda_repr)?
                );
                let cuda_ptr = device_box.as_device_ptr();

                let alloc = necsim_cuda::ScopedCudaAlloc::new(device_box, tail_alloc);

                let result = inner(cuda_ptr);

                core::mem::drop(alloc);

                result
            }
        }

        #[cfg(target_os = "cuda")]
        unsafe impl #impl_generics necsim_cuda::BorrowFromRust for #name #ty_generics
            #where_clause
        {
            unsafe fn with_borrow_from_rust<O, F: FnOnce(
                &Self
            ) -> O>(
                this: *const <Self as necsim_cuda::RustToCuda>::CudaRepresentation,
                inner: F,
            ) -> O {
                use necsim_cuda::CudaAsRust;

                let cuda_repr_ref: &<Self as necsim_cuda::RustToCuda>::CudaRepresentation = &*this;

                let rust_repr = cuda_repr_ref.as_rust();

                let result = inner(&rust_repr);

                // MUST forget about rust_repr as we do NOT own any of the heap memory
                // it might reference
                core::mem::forget(rust_repr);

                result
            }
        }
    })
    .into()
}

#[proc_macro_derive(LendToCuda)]
pub fn lend_to_cuda_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_lend_to_cuda(&ast)
}
