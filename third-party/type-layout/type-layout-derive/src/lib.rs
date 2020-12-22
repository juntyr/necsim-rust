extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{Literal};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Index};

#[proc_macro_derive(TypeLayout)]
pub fn derive_type_layout(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let name_str = Literal::string(&name.to_string());

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let layout = layout_of_type(&(quote! { #name #ty_generics }), &input.data);

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #impl_generics ::type_layout::TypeLayout for #name #ty_generics #where_clause {
            fn type_layout() -> ::type_layout::TypeLayoutInfo {
                #layout

                ::type_layout::TypeLayoutInfo {
                    name: ::type_layout::alloc::borrow::Cow::Borrowed(#name_str),
                    size: ::core::mem::size_of::<#name #ty_generics>(),
                    alignment: ::core::mem::align_of::<#name #ty_generics>(),
                    structure,
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

fn layout_of_type(struct_name: &proc_macro2::TokenStream, data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let fields = quote_fields(struct_name, quote_field_values(struct_name, &data.fields));
            
            quote! {
                #fields
                
                let structure = ::type_layout::TypeStructure::StructLike { fields };
            }
        },
        Data::Enum(_enum) => {
            unimplemented!("Calculating the type layout of enums is not yet supported.")
            
            /* IDEA: Getting the offsets this way could work
            
            enum Test {
                A {
                    a: u32
                },
                B {
                    e: (),
                    b: u8,
                    d: u16,
                    c: u32,
                }
            }

            fn main() {
                let uninit = unsafe { Test::B {
                    e: std::mem::MaybeUninit::<()>::uninit().assume_init(),
                    b: std::mem::MaybeUninit::<u8>::uninit().assume_init(),
                    d: std::mem::MaybeUninit::<u16>::uninit().assume_init(),
                    c: std::mem::MaybeUninit::<u32>::uninit().assume_init(),
                } };
                let base_ptr: *const Test = &uninit as *const Test;
                let field_ptr = {
                    match unsafe { &*base_ptr } {
                        Test::B { e: v, .. } => v as *const _,
                        _ => unreachable!(),
                    }
                };
                let offset = (field_ptr as usize) - (base_ptr as usize);

                println!("{}", offset);
            }
            
            */
            
            /*let variant_values = r#enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let variant_name_str = Literal::string(&variant_name.to_string());
                
                let fields = quote_fields(struct_name, quote_field_values(&(quote! { #struct_name :: #variant_name }), &variant.fields));

                quote! {
                    #fields
                    
                    variants.push((::type_layout::alloc::borrow::Cow::Borrowed(#variant_name_str), fields));
                }
            });

            quote! {
                let mut variants = ::type_layout::alloc::vec::Vec::new();

                #(#variant_values)*

                let structure = ::type_layout::TypeStructure::Enum { variants };
            }*/
        },
        Data::Union(union) => {
            let values = union.fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    #[allow(unused_assignments)]
                    {
                        fields.push(::type_layout::Field::Field {
                            name: ::type_layout::alloc::borrow::Cow::Borrowed(#field_name_str),
                            ty: ::type_layout::alloc::borrow::Cow::Owned(::type_layout::tynm::type_name::<#field_ty>()),
                            offset: ::type_layout::memoffset::offset_of_union!(#struct_name, #field_name),
                            size: ::core::mem::size_of::<#field_ty>(),
                            alignment: ::core::mem::align_of::<#field_ty>(),
                        });
                    }
                }
            }).collect();

            let fields = quote_fields(struct_name, values);

            quote! {
                #fields

                let structure = ::type_layout::TypeStructure::StructLike { fields };
            }
        },
    }
}

fn quote_field_values(struct_name: &proc_macro2::TokenStream, fields: &Fields) -> Vec<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(fields) => {
            fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    #[allow(unused_assignments)]
                    {
                        fields.push(::type_layout::Field::Field {
                            name: ::type_layout::alloc::borrow::Cow::Borrowed(#field_name_str),
                            ty: ::type_layout::alloc::borrow::Cow::Owned(::type_layout::tynm::type_name::<#field_ty>()),
                            offset: ::type_layout::memoffset::offset_of!(#struct_name, #field_name),
                            size: ::core::mem::size_of::<#field_ty>(),
                            alignment: ::core::mem::align_of::<#field_ty>(),
                        });
                    }
                }
            }).collect()
        },
        Fields::Unnamed(fields) => {
            fields.unnamed.iter().enumerate().map(|(field_index, field)| {
                let field_name = Index::from(field_index);
                let field_name_str = Literal::string(&field_index.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    #[allow(unused_assignments)]
                    {
                        fields.push(::type_layout::Field::Field {
                            name: ::type_layout::alloc::borrow::Cow::Borrowed(#field_name_str),
                            ty: ::type_layout::alloc::borrow::Cow::Owned(::type_layout::tynm::type_name::<#field_ty>()),
                            offset: ::type_layout::memoffset::offset_of!(#struct_name, #field_name),
                            size: ::core::mem::size_of::<#field_ty>(),
                            alignment: ::core::mem::align_of::<#field_ty>(),
                        });
                    }
                }
            }).collect()
        },
        Fields::Unit => vec![],
    }
}

fn quote_fields(struct_name: &proc_macro2::TokenStream, values: Vec<proc_macro2::TokenStream>) -> proc_macro2::TokenStream {
    quote! {
        let mut fields = ::type_layout::alloc::vec::Vec::new();

        #(#values)*

        fields.sort_by_key(|e| match e {
            ::type_layout::Field::Field { offset, size, ..} | ::type_layout::Field::Padding { offset, size, ..} => (*offset, *size),
        });

        let mut last_field_end = 0;
        let mut field_index = 0;

        while field_index < fields.len() {
            let (field_offset, field_size) = match &fields[field_index] {
                ::type_layout::Field::Field { offset, size, ..} | ::type_layout::Field::Padding { offset, size, ..} => (*offset, *size),
            };

            if field_offset > last_field_end {
                fields.insert(field_index, ::type_layout::Field::Padding {
                    offset: last_field_end,
                    size: field_offset - last_field_end
                });

                field_index += 2;
            } else {
                field_index += 1;
            }

            last_field_end = field_offset + field_size;
        }

        let struct_size = ::core::mem::size_of::<#struct_name>();

        if struct_size > last_field_end {
            fields.push(::type_layout::Field::Padding {
                offset: last_field_end,
                size: struct_size - last_field_end,
            });
        }
    }
}
