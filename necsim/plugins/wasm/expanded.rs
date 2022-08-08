#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod host {
    #[allow(clippy::all)]
    pub mod serde_wasm_host {
        #[allow(unused_imports)]
        use wit_bindgen_wasmtime::{wasmtime, anyhow};
        #[repr(C)]
        pub struct Usize {
            pub size: u32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for Usize {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Usize {
            #[inline]
            fn clone(&self) -> Usize {
                {
                    let _: ::core::clone::AssertParamIsClone<u32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for Usize {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Usize").field("size", &self.size).finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for Usize {
            fn into_le(self) -> Self {
                Self {
                    size: self.size.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    size: self.size.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for Usize {}
        pub enum Unexpected<'a> {
            Bool(bool),
            Unsigned(u64),
            Signed(i64),
            Float(f64),
            Char(char),
            Str(&'a str),
            Bytes(&'a [u8]),
            Unit,
            Option,
            NewtypeStruct,
            Seq,
            Map,
            Enum,
            UnitVariant,
            NewtypeVariant,
            TupleVariant,
            StructVariant,
            Other(&'a str),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<'a> ::core::clone::Clone for Unexpected<'a> {
            #[inline]
            fn clone(&self) -> Unexpected<'a> {
                match (&*self,) {
                    (&Unexpected::Bool(ref __self_0),) => {
                        Unexpected::Bool(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Unexpected::Unsigned(ref __self_0),) => {
                        Unexpected::Unsigned(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Unexpected::Signed(ref __self_0),) => {
                        Unexpected::Signed(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Unexpected::Float(ref __self_0),) => {
                        Unexpected::Float(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Unexpected::Char(ref __self_0),) => {
                        Unexpected::Char(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Unexpected::Str(ref __self_0),) => {
                        Unexpected::Str(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Unexpected::Bytes(ref __self_0),) => {
                        Unexpected::Bytes(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Unexpected::Unit,) => Unexpected::Unit,
                    (&Unexpected::Option,) => Unexpected::Option,
                    (&Unexpected::NewtypeStruct,) => Unexpected::NewtypeStruct,
                    (&Unexpected::Seq,) => Unexpected::Seq,
                    (&Unexpected::Map,) => Unexpected::Map,
                    (&Unexpected::Enum,) => Unexpected::Enum,
                    (&Unexpected::UnitVariant,) => Unexpected::UnitVariant,
                    (&Unexpected::NewtypeVariant,) => Unexpected::NewtypeVariant,
                    (&Unexpected::TupleVariant,) => Unexpected::TupleVariant,
                    (&Unexpected::StructVariant,) => Unexpected::StructVariant,
                    (&Unexpected::Other(ref __self_0),) => {
                        Unexpected::Other(::core::clone::Clone::clone(&(*__self_0)))
                    }
                }
            }
        }
        impl<'a> core::fmt::Debug for Unexpected<'a> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Unexpected::Bool(e) => f.debug_tuple("Unexpected::Bool").field(e).finish(),
                    Unexpected::Unsigned(e) => {
                        f.debug_tuple("Unexpected::Unsigned").field(e).finish()
                    }
                    Unexpected::Signed(e) => f.debug_tuple("Unexpected::Signed").field(e).finish(),
                    Unexpected::Float(e) => f.debug_tuple("Unexpected::Float").field(e).finish(),
                    Unexpected::Char(e) => f.debug_tuple("Unexpected::Char").field(e).finish(),
                    Unexpected::Str(e) => f.debug_tuple("Unexpected::Str").field(e).finish(),
                    Unexpected::Bytes(e) => f.debug_tuple("Unexpected::Bytes").field(e).finish(),
                    Unexpected::Unit => f.debug_tuple("Unexpected::Unit").finish(),
                    Unexpected::Option => f.debug_tuple("Unexpected::Option").finish(),
                    Unexpected::NewtypeStruct => {
                        f.debug_tuple("Unexpected::NewtypeStruct").finish()
                    }
                    Unexpected::Seq => f.debug_tuple("Unexpected::Seq").finish(),
                    Unexpected::Map => f.debug_tuple("Unexpected::Map").finish(),
                    Unexpected::Enum => f.debug_tuple("Unexpected::Enum").finish(),
                    Unexpected::UnitVariant => f.debug_tuple("Unexpected::UnitVariant").finish(),
                    Unexpected::NewtypeVariant => {
                        f.debug_tuple("Unexpected::NewtypeVariant").finish()
                    }
                    Unexpected::TupleVariant => f.debug_tuple("Unexpected::TupleVariant").finish(),
                    Unexpected::StructVariant => {
                        f.debug_tuple("Unexpected::StructVariant").finish()
                    }
                    Unexpected::Other(e) => f.debug_tuple("Unexpected::Other").field(e).finish(),
                }
            }
        }
        #[repr(C)]
        pub struct VisitorHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for VisitorHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for VisitorHandle {
            #[inline]
            fn clone(&self) -> VisitorHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for VisitorHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("VisitorHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for VisitorHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for VisitorHandle {}
        #[repr(C)]
        pub struct DeValueHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for DeValueHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for DeValueHandle {
            #[inline]
            fn clone(&self) -> DeValueHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for DeValueHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("DeValueHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for DeValueHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for DeValueHandle {}
        #[repr(C)]
        pub struct DeserializeSeedHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for DeserializeSeedHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for DeserializeSeedHandle {
            #[inline]
            fn clone(&self) -> DeserializeSeedHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for DeserializeSeedHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("DeserializeSeedHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for DeserializeSeedHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for DeserializeSeedHandle {}
        pub trait SerdeWasmHost: Sized {
            type DeError: std::fmt::Debug;
            type Deserializer: std::fmt::Debug;
            type EnumAccess: std::fmt::Debug;
            type MapAccess: std::fmt::Debug;
            type SeqAccess: std::fmt::Debug;
            type VariantAccess: std::fmt::Debug;
            fn de_error_custom(&mut self, msg: &str) -> Self::DeError;
            fn de_error_invalid_type(&mut self, unexp: Unexpected<'_>, exp: &str) -> Self::DeError;
            fn de_error_invalid_value(&mut self, unexp: Unexpected<'_>, exp: &str)
                -> Self::DeError;
            fn de_error_invalid_length(&mut self, len: Usize, exp: &str) -> Self::DeError;
            fn de_error_unknown_variant(
                &mut self,
                variant: &str,
                expected: Vec<&str>,
            ) -> Self::DeError;
            fn de_error_unknown_field(&mut self, field: &str, expected: Vec<&str>)
                -> Self::DeError;
            fn de_error_missing_field(&mut self, field: &str) -> Self::DeError;
            fn de_error_duplicate_field(&mut self, field: &str) -> Self::DeError;
            fn deserializer_erased_deserialize_any(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_bool(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_u8(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_u16(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_u32(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_u64(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_i8(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_i16(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_i32(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_i64(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_i128(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_u128(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_f32(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_f64(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_char(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_str(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_string(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_bytes(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_byte_buf(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_option(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_unit(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_unit_struct(
                &mut self,
                self_: &Self::Deserializer,
                name: &str,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_newtype_struct(
                &mut self,
                self_: &Self::Deserializer,
                name: &str,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_seq(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_tuple(
                &mut self,
                self_: &Self::Deserializer,
                len: Usize,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_tuple_struct(
                &mut self,
                self_: &Self::Deserializer,
                name: &str,
                len: Usize,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_map(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_struct(
                &mut self,
                self_: &Self::Deserializer,
                name: &str,
                fields: Vec<&str>,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_identifier(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_enum(
                &mut self,
                self_: &Self::Deserializer,
                name: &str,
                variants: Vec<&str>,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_deserialize_ignored_any(
                &mut self,
                self_: &Self::Deserializer,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn deserializer_erased_is_human_readable(&mut self, self_: &Self::Deserializer)
                -> bool;
            fn seq_access_erased_next_element(
                &mut self,
                self_: &Self::SeqAccess,
                seed: DeserializeSeedHandle,
            ) -> Result<Option<DeValueHandle>, Self::DeError>;
            fn seq_access_erased_size_hint(&mut self, self_: &Self::SeqAccess) -> Option<Usize>;
            fn map_access_erased_next_key(
                &mut self,
                self_: &Self::MapAccess,
                seed: DeserializeSeedHandle,
            ) -> Result<Option<DeValueHandle>, Self::DeError>;
            fn map_access_erased_next_value(
                &mut self,
                self_: &Self::MapAccess,
                seed: DeserializeSeedHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn map_access_erased_next_entry(
                &mut self,
                self_: &Self::MapAccess,
                kseed: DeserializeSeedHandle,
                vseed: DeserializeSeedHandle,
            ) -> Result<Option<(DeValueHandle, DeValueHandle)>, Self::DeError>;
            fn map_access_erased_size_hint(&mut self, self_: &Self::MapAccess) -> Option<Usize>;
            fn enum_access_erased_variant_seed(
                &mut self,
                self_: &Self::EnumAccess,
                seed: DeserializeSeedHandle,
            ) -> Result<(DeValueHandle, Self::VariantAccess), Self::DeError>;
            fn variant_access_unit_variant(
                &mut self,
                self_: &Self::VariantAccess,
            ) -> Result<(), Self::DeError>;
            fn variant_access_newtype_variant_seed(
                &mut self,
                self_: &Self::VariantAccess,
                seed: DeserializeSeedHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn variant_access_tuple_variant(
                &mut self,
                self_: &Self::VariantAccess,
                len: Usize,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn variant_access_struct_variant(
                &mut self,
                self_: &Self::VariantAccess,
                fields: Vec<&str>,
                visitor: VisitorHandle,
            ) -> Result<DeValueHandle, Self::DeError>;
            fn drop_de_error(&mut self, state: Self::DeError) {
                drop(state);
            }
            fn drop_deserializer(&mut self, state: Self::Deserializer) {
                drop(state);
            }
            fn drop_enum_access(&mut self, state: Self::EnumAccess) {
                drop(state);
            }
            fn drop_map_access(&mut self, state: Self::MapAccess) {
                drop(state);
            }
            fn drop_seq_access(&mut self, state: Self::SeqAccess) {
                drop(state);
            }
            fn drop_variant_access(&mut self, state: Self::VariantAccess) {
                drop(state);
            }
        }
        pub struct SerdeWasmHostTables<T: SerdeWasmHost> {
            pub(crate) de_error_table: wit_bindgen_wasmtime::Table<T::DeError>,
            pub(crate) deserializer_table: wit_bindgen_wasmtime::Table<T::Deserializer>,
            pub(crate) enum_access_table: wit_bindgen_wasmtime::Table<T::EnumAccess>,
            pub(crate) map_access_table: wit_bindgen_wasmtime::Table<T::MapAccess>,
            pub(crate) seq_access_table: wit_bindgen_wasmtime::Table<T::SeqAccess>,
            pub(crate) variant_access_table: wit_bindgen_wasmtime::Table<T::VariantAccess>,
        }
        impl<T: SerdeWasmHost> Default for SerdeWasmHostTables<T> {
            fn default() -> Self {
                Self {
                    de_error_table: Default::default(),
                    deserializer_table: Default::default(),
                    enum_access_table: Default::default(),
                    map_access_table: Default::default(),
                    seq_access_table: Default::default(),
                    variant_access_table: Default::default(),
                }
            }
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::Linker<T>,
            get: impl Fn(&mut T) -> (&mut U, &mut SerdeWasmHostTables<U>) + Send + Sync + Copy + 'static,
        ) -> anyhow::Result<()>
        where
            U: SerdeWasmHost,
        {
            use wit_bindgen_wasmtime::rt::get_memory;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::custom",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg0;
                    let len0 = arg1;
                    let param0 = _bc.slice_str(ptr0, len0)?;
                    let result = host.de_error_custom(param0);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::invalid-type",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i64,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr3 = arg3;
                    let len3 = arg4;
                    let param0 = match arg0 {
                        0 => Unexpected::Bool(match arg1 as i32 {
                            0 => false,
                            1 => true,
                            _ => return Err(invalid_variant("bool")),
                        }),
                        1 => Unexpected::Unsigned(arg1 as u64),
                        2 => Unexpected::Signed(arg1),
                        3 => Unexpected::Float(f64::from_bits(arg1 as u64)),
                        4 => Unexpected::Char(char_from_i32(arg1 as i32)?),
                        5 => Unexpected::Str({
                            let ptr0 = arg1 as i32;
                            let len0 = arg2;
                            _bc.slice_str(ptr0, len0)?
                        }),
                        6 => Unexpected::Bytes({
                            let ptr1 = arg1 as i32;
                            let len1 = arg2;
                            _bc.slice(ptr1, len1)?
                        }),
                        7 => Unexpected::Unit,
                        8 => Unexpected::Option,
                        9 => Unexpected::NewtypeStruct,
                        10 => Unexpected::Seq,
                        11 => Unexpected::Map,
                        12 => Unexpected::Enum,
                        13 => Unexpected::UnitVariant,
                        14 => Unexpected::NewtypeVariant,
                        15 => Unexpected::TupleVariant,
                        16 => Unexpected::StructVariant,
                        17 => Unexpected::Other({
                            let ptr2 = arg1 as i32;
                            let len2 = arg2;
                            _bc.slice_str(ptr2, len2)?
                        }),
                        _ => return Err(invalid_variant("Unexpected")),
                    };
                    let param1 = _bc.slice_str(ptr3, len3)?;
                    let result = host.de_error_invalid_type(param0, param1);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::invalid-value",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i64,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr3 = arg3;
                    let len3 = arg4;
                    let param0 = match arg0 {
                        0 => Unexpected::Bool(match arg1 as i32 {
                            0 => false,
                            1 => true,
                            _ => return Err(invalid_variant("bool")),
                        }),
                        1 => Unexpected::Unsigned(arg1 as u64),
                        2 => Unexpected::Signed(arg1),
                        3 => Unexpected::Float(f64::from_bits(arg1 as u64)),
                        4 => Unexpected::Char(char_from_i32(arg1 as i32)?),
                        5 => Unexpected::Str({
                            let ptr0 = arg1 as i32;
                            let len0 = arg2;
                            _bc.slice_str(ptr0, len0)?
                        }),
                        6 => Unexpected::Bytes({
                            let ptr1 = arg1 as i32;
                            let len1 = arg2;
                            _bc.slice(ptr1, len1)?
                        }),
                        7 => Unexpected::Unit,
                        8 => Unexpected::Option,
                        9 => Unexpected::NewtypeStruct,
                        10 => Unexpected::Seq,
                        11 => Unexpected::Map,
                        12 => Unexpected::Enum,
                        13 => Unexpected::UnitVariant,
                        14 => Unexpected::NewtypeVariant,
                        15 => Unexpected::TupleVariant,
                        16 => Unexpected::StructVariant,
                        17 => Unexpected::Other({
                            let ptr2 = arg1 as i32;
                            let len2 = arg2;
                            _bc.slice_str(ptr2, len2)?
                        }),
                        _ => return Err(invalid_variant("Unexpected")),
                    };
                    let param1 = _bc.slice_str(ptr3, len3)?;
                    let result = host.de_error_invalid_value(param0, param1);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::invalid-length",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg1;
                    let len0 = arg2;
                    let param0 = Usize { size: arg0 as u32 };
                    let param1 = _bc.slice_str(ptr0, len0)?;
                    let result = host.de_error_invalid_length(param0, param1);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::unknown-variant",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg0;
                    let len0 = arg1;
                    let len4 = arg3;
                    let base4 = arg2;
                    let mut result4 = Vec::with_capacity(len4 as usize);
                    for i in 0..len4 {
                        let base = base4 + i * 8;
                        result4.push({
                            let load1 = _bc.load::<i32>(base + 0)?;
                            let load2 = _bc.load::<i32>(base + 4)?;
                            let ptr3 = load1;
                            let len3 = load2;
                            _bc.slice_str(ptr3, len3)?
                        });
                    }
                    let param0 = _bc.slice_str(ptr0, len0)?;
                    let param1 = result4;
                    let result = host.de_error_unknown_variant(param0, param1);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::unknown-field",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg0;
                    let len0 = arg1;
                    let len4 = arg3;
                    let base4 = arg2;
                    let mut result4 = Vec::with_capacity(len4 as usize);
                    for i in 0..len4 {
                        let base = base4 + i * 8;
                        result4.push({
                            let load1 = _bc.load::<i32>(base + 0)?;
                            let load2 = _bc.load::<i32>(base + 4)?;
                            let ptr3 = load1;
                            let len3 = load2;
                            _bc.slice_str(ptr3, len3)?
                        });
                    }
                    let param0 = _bc.slice_str(ptr0, len0)?;
                    let param1 = result4;
                    let result = host.de_error_unknown_field(param0, param1);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::missing-field",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg0;
                    let len0 = arg1;
                    let param0 = _bc.slice_str(ptr0, len0)?;
                    let result = host.de_error_missing_field(param0);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "de-error::duplicate-field",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg0;
                    let len0 = arg1;
                    let param0 = _bc.slice_str(ptr0, len0)?;
                    let result = host.de_error_duplicate_field(param0);
                    Ok(_tables.de_error_table.insert(result) as i32)
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-any",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_any(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-bool",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_bool(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-u8",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_u8(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-u16",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_u16(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-u32",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_u32(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-u64",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_u64(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-i8",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_i8(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-i16",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_i16(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-i32",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_i32(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-i64",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_i64(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-i128",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_i128(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-u128",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_u128(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-f32",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_f32(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-f64",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_f64(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-char",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_char(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-str",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_str(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-string",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_string(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-bytes",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_bytes(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-byte-buf",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_byte_buf(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-option",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_option(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-unit",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_unit(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-unit-struct",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg1;
                    let len0 = arg2;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = _bc.slice_str(ptr0, len0)?;
                    let param2 = VisitorHandle { handle: arg3 };
                    let result =
                        host.deserializer_erased_deserialize_unit_struct(param0, param1, param2);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg4 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle1 } = e;
                            caller_memory.store(
                                arg4 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle1,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg4 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg4 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-newtype-struct",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg1;
                    let len0 = arg2;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = _bc.slice_str(ptr0, len0)?;
                    let param2 = VisitorHandle { handle: arg3 };
                    let result =
                        host.deserializer_erased_deserialize_newtype_struct(param0, param1, param2);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg4 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle1 } = e;
                            caller_memory.store(
                                arg4 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle1,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg4 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg4 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-seq",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_seq(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-tuple",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = Usize { size: arg1 as u32 };
                    let param2 = VisitorHandle { handle: arg2 };
                    let result = host.deserializer_erased_deserialize_tuple(param0, param1, param2);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg3 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg3 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-tuple-struct",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32,
                      arg5: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg1;
                    let len0 = arg2;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = _bc.slice_str(ptr0, len0)?;
                    let param2 = Usize { size: arg3 as u32 };
                    let param3 = VisitorHandle { handle: arg4 };
                    let result = host.deserializer_erased_deserialize_tuple_struct(
                        param0, param1, param2, param3,
                    );
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle1 } = e;
                            caller_memory.store(
                                arg5 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle1,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg5 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-map",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_map(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-struct",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32,
                      arg5: i32,
                      arg6: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg1;
                    let len0 = arg2;
                    let len4 = arg4;
                    let base4 = arg3;
                    let mut result4 = Vec::with_capacity(len4 as usize);
                    for i in 0..len4 {
                        let base = base4 + i * 8;
                        result4.push({
                            let load1 = _bc.load::<i32>(base + 0)?;
                            let load2 = _bc.load::<i32>(base + 4)?;
                            let ptr3 = load1;
                            let len3 = load2;
                            _bc.slice_str(ptr3, len3)?
                        });
                    }
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = _bc.slice_str(ptr0, len0)?;
                    let param2 = result4;
                    let param3 = VisitorHandle { handle: arg5 };
                    let result =
                        host.deserializer_erased_deserialize_struct(param0, param1, param2, param3);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg6 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle5 } = e;
                            caller_memory.store(
                                arg6 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle5,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg6 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg6 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-identifier",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_identifier(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-enum",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32,
                      arg5: i32,
                      arg6: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let ptr0 = arg1;
                    let len0 = arg2;
                    let len4 = arg4;
                    let base4 = arg3;
                    let mut result4 = Vec::with_capacity(len4 as usize);
                    for i in 0..len4 {
                        let base = base4 + i * 8;
                        result4.push({
                            let load1 = _bc.load::<i32>(base + 0)?;
                            let load2 = _bc.load::<i32>(base + 4)?;
                            let ptr3 = load1;
                            let len3 = load2;
                            _bc.slice_str(ptr3, len3)?
                        });
                    }
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = _bc.slice_str(ptr0, len0)?;
                    let param2 = result4;
                    let param3 = VisitorHandle { handle: arg5 };
                    let result =
                        host.deserializer_erased_deserialize_enum(param0, param1, param2, param3);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg6 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle5 } = e;
                            caller_memory.store(
                                arg6 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle5,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg6 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg6 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-deserialize-ignored-any",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = VisitorHandle { handle: arg1 };
                    let result = host.deserializer_erased_deserialize_ignored_any(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "deserializer::erased-is-human-readable",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .deserializer_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let result = host.deserializer_erased_is_human_readable(param0);
                    Ok(match result {
                        true => 1,
                        false => 0,
                    })
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "seq-access::erased-next-element",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .seq_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = DeserializeSeedHandle { handle: arg1 };
                    let result = host.seq_access_erased_next_element(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            match e {
                                Some(e) => {
                                    let (caller_memory, data) =
                                        memory.data_and_store_mut(&mut caller);
                                    let (_, _tables) = get(data);
                                    caller_memory.store(
                                        arg2 + 4,
                                        wit_bindgen_wasmtime::rt::as_i32(1i32) as u8,
                                    )?;
                                    let DeValueHandle { handle: handle0 } = e;
                                    caller_memory.store(
                                        arg2 + 8,
                                        wit_bindgen_wasmtime::rt::as_i32(
                                            wit_bindgen_wasmtime::rt::as_i32(handle0),
                                        ),
                                    )?;
                                }
                                None => {
                                    let e = ();
                                    {
                                        caller_memory.store(
                                            arg2 + 4,
                                            wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                        )?;
                                        let () = e;
                                    }
                                }
                            };
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "seq-access::erased-size-hint",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .seq_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let result = host.seq_access_erased_size_hint(param0);
                    match result {
                        Some(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg1 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            let Usize { size: size0 } = e;
                            caller_memory.store(
                                arg1 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    size0,
                                )),
                            )?;
                        }
                        None => {
                            let e = ();
                            {
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store(
                                    arg1 + 0,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let () = e;
                            }
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "map-access::erased-next-key",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .map_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = DeserializeSeedHandle { handle: arg1 };
                    let result = host.map_access_erased_next_key(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            match e {
                                Some(e) => {
                                    let (caller_memory, data) =
                                        memory.data_and_store_mut(&mut caller);
                                    let (_, _tables) = get(data);
                                    caller_memory.store(
                                        arg2 + 4,
                                        wit_bindgen_wasmtime::rt::as_i32(1i32) as u8,
                                    )?;
                                    let DeValueHandle { handle: handle0 } = e;
                                    caller_memory.store(
                                        arg2 + 8,
                                        wit_bindgen_wasmtime::rt::as_i32(
                                            wit_bindgen_wasmtime::rt::as_i32(handle0),
                                        ),
                                    )?;
                                }
                                None => {
                                    let e = ();
                                    {
                                        caller_memory.store(
                                            arg2 + 4,
                                            wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                        )?;
                                        let () = e;
                                    }
                                }
                            };
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "map-access::erased-next-value",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .map_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = DeserializeSeedHandle { handle: arg1 };
                    let result = host.map_access_erased_next_value(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "map-access::erased-next-entry",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .map_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = DeserializeSeedHandle { handle: arg1 };
                    let param2 = DeserializeSeedHandle { handle: arg2 };
                    let result = host.map_access_erased_next_entry(param0, param1, param2);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            match e {
                                Some(e) => {
                                    let (caller_memory, data) =
                                        memory.data_and_store_mut(&mut caller);
                                    let (_, _tables) = get(data);
                                    caller_memory.store(
                                        arg3 + 4,
                                        wit_bindgen_wasmtime::rt::as_i32(1i32) as u8,
                                    )?;
                                    let (t0_0, t0_1) = e;
                                    let DeValueHandle { handle: handle1 } = t0_0;
                                    caller_memory.store(
                                        arg3 + 8,
                                        wit_bindgen_wasmtime::rt::as_i32(
                                            wit_bindgen_wasmtime::rt::as_i32(handle1),
                                        ),
                                    )?;
                                    let DeValueHandle { handle: handle2 } = t0_1;
                                    caller_memory.store(
                                        arg3 + 12,
                                        wit_bindgen_wasmtime::rt::as_i32(
                                            wit_bindgen_wasmtime::rt::as_i32(handle2),
                                        ),
                                    )?;
                                }
                                None => {
                                    let e = ();
                                    {
                                        caller_memory.store(
                                            arg3 + 4,
                                            wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                        )?;
                                        let () = e;
                                    }
                                }
                            };
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg3 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "map-access::erased-size-hint",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .map_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let result = host.map_access_erased_size_hint(param0);
                    match result {
                        Some(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg1 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            let Usize { size: size0 } = e;
                            caller_memory.store(
                                arg1 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    size0,
                                )),
                            )?;
                        }
                        None => {
                            let e = ();
                            {
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store(
                                    arg1 + 0,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let () = e;
                            }
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "enum-access::erased-variant-seed",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .enum_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = DeserializeSeedHandle { handle: arg1 };
                    let result = host.enum_access_erased_variant_seed(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let (t0_0, t0_1) = e;
                            let DeValueHandle { handle: handle1 } = t0_0;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle1,
                                )),
                            )?;
                            caller_memory.store(
                                arg2 + 8,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.variant_access_table.insert(t0_1) as i32,
                                ),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "variant-access::unit-variant",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .variant_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let result = host.variant_access_unit_variant(param0);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg1 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let () = e;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg1 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg1 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "variant-access::newtype-variant-seed",
                move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .variant_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = DeserializeSeedHandle { handle: arg1 };
                    let result = host.variant_access_newtype_variant_seed(param0, param1);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg2 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "variant-access::tuple-variant",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let host = get(caller.data_mut());
                    let (host, _tables) = host;
                    let param0 = _tables
                        .variant_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = Usize { size: arg1 as u32 };
                    let param2 = VisitorHandle { handle: arg2 };
                    let result = host.variant_access_tuple_variant(param0, param1, param2);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle0 } = e;
                            caller_memory.store(
                                arg3 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle0,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg3 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "serde-wasm-host",
                "variant-access::struct-variant",
                move |mut caller: wasmtime::Caller<'_, T>,
                      arg0: i32,
                      arg1: i32,
                      arg2: i32,
                      arg3: i32,
                      arg4: i32| {
                    let memory = &get_memory(&mut caller, "memory")?;
                    let (mem, data) = memory.data_and_store_mut(&mut caller);
                    let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                    let host = get(data);
                    let (host, _tables) = host;
                    let len3 = arg2;
                    let base3 = arg1;
                    let mut result3 = Vec::with_capacity(len3 as usize);
                    for i in 0..len3 {
                        let base = base3 + i * 8;
                        result3.push({
                            let load0 = _bc.load::<i32>(base + 0)?;
                            let load1 = _bc.load::<i32>(base + 4)?;
                            let ptr2 = load0;
                            let len2 = load1;
                            _bc.slice_str(ptr2, len2)?
                        });
                    }
                    let param0 = _tables
                        .variant_access_table
                        .get((arg0) as u32)
                        .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                    let param1 = result3;
                    let param2 = VisitorHandle { handle: arg3 };
                    let result = host.variant_access_struct_variant(param0, param1, param2);
                    match result {
                        Ok(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg4 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                            let DeValueHandle { handle: handle4 } = e;
                            caller_memory.store(
                                arg4 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(wit_bindgen_wasmtime::rt::as_i32(
                                    handle4,
                                )),
                            )?;
                        }
                        Err(e) => {
                            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                            let (_, _tables) = get(data);
                            caller_memory
                                .store(arg4 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                            caller_memory.store(
                                arg4 + 4,
                                wit_bindgen_wasmtime::rt::as_i32(
                                    _tables.de_error_table.insert(e) as i32
                                ),
                            )?;
                        }
                    };
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_drop_de-error",
                move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                    let (host, tables) = get(caller.data_mut());
                    let handle = tables.de_error_table.remove(handle).map_err(|e| {
                        wasmtime::Trap::new({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["failed to remove handle: "],
                                &[::core::fmt::ArgumentV1::new_display(&e)],
                            ));
                            res
                        })
                    })?;
                    host.drop_de_error(handle);
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_drop_deserializer",
                move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                    let (host, tables) = get(caller.data_mut());
                    let handle = tables.deserializer_table.remove(handle).map_err(|e| {
                        wasmtime::Trap::new({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["failed to remove handle: "],
                                &[::core::fmt::ArgumentV1::new_display(&e)],
                            ));
                            res
                        })
                    })?;
                    host.drop_deserializer(handle);
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_drop_enum-access",
                move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                    let (host, tables) = get(caller.data_mut());
                    let handle = tables.enum_access_table.remove(handle).map_err(|e| {
                        wasmtime::Trap::new({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["failed to remove handle: "],
                                &[::core::fmt::ArgumentV1::new_display(&e)],
                            ));
                            res
                        })
                    })?;
                    host.drop_enum_access(handle);
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_drop_map-access",
                move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                    let (host, tables) = get(caller.data_mut());
                    let handle = tables.map_access_table.remove(handle).map_err(|e| {
                        wasmtime::Trap::new({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["failed to remove handle: "],
                                &[::core::fmt::ArgumentV1::new_display(&e)],
                            ));
                            res
                        })
                    })?;
                    host.drop_map_access(handle);
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_drop_seq-access",
                move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                    let (host, tables) = get(caller.data_mut());
                    let handle = tables.seq_access_table.remove(handle).map_err(|e| {
                        wasmtime::Trap::new({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["failed to remove handle: "],
                                &[::core::fmt::ArgumentV1::new_display(&e)],
                            ));
                            res
                        })
                    })?;
                    host.drop_seq_access(handle);
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_drop_variant-access",
                move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                    let (host, tables) = get(caller.data_mut());
                    let handle = tables.variant_access_table.remove(handle).map_err(|e| {
                        wasmtime::Trap::new({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["failed to remove handle: "],
                                &[::core::fmt::ArgumentV1::new_display(&e)],
                            ));
                            res
                        })
                    })?;
                    host.drop_variant_access(handle);
                    Ok(())
                },
            )?;
            Ok(())
        }
        use wit_bindgen_wasmtime::rt::RawMem;
        use wit_bindgen_wasmtime::rt::char_from_i32;
        use wit_bindgen_wasmtime::rt::invalid_variant;
    }
    const _ : & str = "record usize {\n  size: u32,\n}\n\nvariant unexpected {\n  %bool(bool),\n  unsigned(u64),\n  signed(s64),\n  float(float64),\n  %char(char),\n  str(string),\n  bytes(list<u8>),\n  %unit,\n  %option,\n  newtype-struct,\n  seq,\n  map,\n  %enum,\n  unit-variant,\n  newtype-variant,\n  tuple-variant,\n  struct-variant,\n  other(string),\n}\n\nresource de-error {\n  static custom: func(msg: string) -> de-error\n  static invalid-type: func(unexp: unexpected, exp: string) -> de-error\n  static invalid-value: func(unexp: unexpected, exp: string) -> de-error\n  static invalid-length: func(len: usize, exp: string) -> de-error\n  static unknown-variant: func(%variant: string, %expected: list<string>) -> de-error\n  static unknown-field: func(field: string, %expected: list<string>) -> de-error\n  static missing-field: func(field: string) -> de-error\n  static duplicate-field: func(field: string) -> de-error\n}\n\nresource deserializer {\n  erased-deserialize-any: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-bool: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u8: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u16: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u32: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u64: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i8: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i16: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i32: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i64: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i128: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u128: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-f32: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-f64: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-char: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-str: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-string: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-bytes: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-byte-buf: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-option: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-unit: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-unit-struct: func(name: string, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-newtype-struct: func(name: string, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-seq: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-tuple: func(len: usize, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-tuple-struct: func(name: string, len: usize, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-map: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-struct: func(name: string, fields: list<string>, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-identifier: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-enum: func(name: string, variants: list<string>, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-ignored-any: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-is-human-readable: func() -> bool\n}\n\nresource seq-access {\n  erased-next-element: func(seed: deserialize-seed-handle) -> expected<option<de-value-handle>, de-error>\n  erased-size-hint: func() -> option<usize>\n}\n\nresource map-access {\n  erased-next-key: func(seed: deserialize-seed-handle) -> expected<option<de-value-handle>, de-error>\n  erased-next-value: func(seed: deserialize-seed-handle) -> expected<de-value-handle, de-error>\n  erased-next-entry: func(kseed: deserialize-seed-handle, vseed: deserialize-seed-handle) -> expected<option<tuple<de-value-handle, de-value-handle>>, de-error>\n  erased-size-hint: func() -> option<usize>\n}\n\nresource enum-access {\n  erased-variant-seed: func(seed: deserialize-seed-handle) -> expected<tuple<de-value-handle, variant-access>, de-error>\n}\n\nresource variant-access {\n  unit-variant: func() -> expected<unit, de-error>\n  newtype-variant-seed: func(seed: deserialize-seed-handle) -> expected<de-value-handle, de-error>\n  tuple-variant: func(len: usize, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  struct-variant: func(fields: list<string>, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n}\n\nrecord visitor-handle {\n  %handle: s32\n}\n\nrecord de-value-handle {\n  %handle: s32\n}\n\nrecord deserialize-seed-handle {\n  %handle: s32\n}\n" ;
    #[allow(clippy::all)]
    pub mod serde_wasm_guest {
        #[allow(unused_imports)]
        use wit_bindgen_wasmtime::{wasmtime, anyhow};
        #[repr(C)]
        pub struct I128 {
            pub hi: u64,
            pub lo: u64,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for I128 {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for I128 {
            #[inline]
            fn clone(&self) -> I128 {
                {
                    let _: ::core::clone::AssertParamIsClone<u64>;
                    let _: ::core::clone::AssertParamIsClone<u64>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for I128 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("I128")
                    .field("hi", &self.hi)
                    .field("lo", &self.lo)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for I128 {
            fn into_le(self) -> Self {
                Self {
                    hi: self.hi.into_le(),
                    lo: self.lo.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    hi: self.hi.from_le(),
                    lo: self.lo.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for I128 {}
        #[repr(C)]
        pub struct U128 {
            pub hi: u64,
            pub lo: u64,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for U128 {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for U128 {
            #[inline]
            fn clone(&self) -> U128 {
                {
                    let _: ::core::clone::AssertParamIsClone<u64>;
                    let _: ::core::clone::AssertParamIsClone<u64>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for U128 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("U128")
                    .field("hi", &self.hi)
                    .field("lo", &self.lo)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for U128 {
            fn into_le(self) -> Self {
                Self {
                    hi: self.hi.into_le(),
                    lo: self.lo.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    hi: self.hi.from_le(),
                    lo: self.lo.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for U128 {}
        #[repr(C)]
        pub struct DeErrorHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for DeErrorHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for DeErrorHandle {
            #[inline]
            fn clone(&self) -> DeErrorHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for DeErrorHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("DeErrorHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for DeErrorHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for DeErrorHandle {}
        #[repr(C)]
        pub struct DeserializerHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for DeserializerHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for DeserializerHandle {
            #[inline]
            fn clone(&self) -> DeserializerHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for DeserializerHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("DeserializerHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for DeserializerHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for DeserializerHandle {}
        #[repr(C)]
        pub struct SeqAccessHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for SeqAccessHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for SeqAccessHandle {
            #[inline]
            fn clone(&self) -> SeqAccessHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for SeqAccessHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("SeqAccessHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for SeqAccessHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for SeqAccessHandle {}
        #[repr(C)]
        pub struct MapAccessHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for MapAccessHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for MapAccessHandle {
            #[inline]
            fn clone(&self) -> MapAccessHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for MapAccessHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("MapAccessHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for MapAccessHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for MapAccessHandle {}
        #[repr(C)]
        pub struct EnumAccessHandle {
            pub handle: i32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for EnumAccessHandle {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for EnumAccessHandle {
            #[inline]
            fn clone(&self) -> EnumAccessHandle {
                {
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    *self
                }
            }
        }
        impl core::fmt::Debug for EnumAccessHandle {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("EnumAccessHandle")
                    .field("handle", &self.handle)
                    .finish()
            }
        }
        impl wit_bindgen_wasmtime::Endian for EnumAccessHandle {
            fn into_le(self) -> Self {
                Self {
                    handle: self.handle.into_le(),
                }
            }
            fn from_le(self) -> Self {
                Self {
                    handle: self.handle.from_le(),
                }
            }
        }
        unsafe impl wit_bindgen_wasmtime::AllBytesValid for EnumAccessHandle {}
        pub struct DeValue(wit_bindgen_wasmtime::rt::ResourceIndex);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for DeValue {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    DeValue(ref __self_0_0) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "DeValue");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        pub struct DeserializeSeed(wit_bindgen_wasmtime::rt::ResourceIndex);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for DeserializeSeed {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    DeserializeSeed(ref __self_0_0) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "DeserializeSeed");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        pub struct Visitor(wit_bindgen_wasmtime::rt::ResourceIndex);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Visitor {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Visitor(ref __self_0_0) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "Visitor");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        /// Auxiliary data associated with the wasm exports.
        ///
        /// This is required to be stored within the data of a
        /// `Store<T>` itself so lifting/lowering state can be managed
        /// when translating between the host and wasm.
        pub struct SerdeWasmGuestData {
            index_slab0: wit_bindgen_wasmtime::rt::IndexSlab,
            resource_slab0: wit_bindgen_wasmtime::rt::ResourceSlab,
            dtor0: Option<wasmtime::TypedFunc<i32, ()>>,
            index_slab1: wit_bindgen_wasmtime::rt::IndexSlab,
            resource_slab1: wit_bindgen_wasmtime::rt::ResourceSlab,
            dtor1: Option<wasmtime::TypedFunc<i32, ()>>,
            index_slab2: wit_bindgen_wasmtime::rt::IndexSlab,
            resource_slab2: wit_bindgen_wasmtime::rt::ResourceSlab,
            dtor2: Option<wasmtime::TypedFunc<i32, ()>>,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for SerdeWasmGuestData {
            #[inline]
            fn default() -> SerdeWasmGuestData {
                SerdeWasmGuestData {
                    index_slab0: ::core::default::Default::default(),
                    resource_slab0: ::core::default::Default::default(),
                    dtor0: ::core::default::Default::default(),
                    index_slab1: ::core::default::Default::default(),
                    resource_slab1: ::core::default::Default::default(),
                    dtor1: ::core::default::Default::default(),
                    index_slab2: ::core::default::Default::default(),
                    resource_slab2: ::core::default::Default::default(),
                    dtor2: ::core::default::Default::default(),
                }
            }
        }
        pub struct SerdeWasmGuest<T> {
            get_state: Box<dyn Fn(&mut T) -> &mut SerdeWasmGuestData + Send + Sync>,
            canonical_abi_free: wasmtime::TypedFunc<(i32, i32, i32), ()>,
            canonical_abi_realloc: wasmtime::TypedFunc<(i32, i32, i32, i32), i32>,
            deserialize_seed_erased_deserialize: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            memory: wasmtime::Memory,
            visitor_erased_expecting: wasmtime::TypedFunc<(i32,), (i32,)>,
            visitor_erased_visit_bool: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_byte_buf: wasmtime::TypedFunc<(i32, i32, i32), (i32,)>,
            visitor_erased_visit_bytes: wasmtime::TypedFunc<(i32, i32, i32), (i32,)>,
            visitor_erased_visit_char: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_enum: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_f32: wasmtime::TypedFunc<(i32, f32), (i32,)>,
            visitor_erased_visit_f64: wasmtime::TypedFunc<(i32, f64), (i32,)>,
            visitor_erased_visit_i128: wasmtime::TypedFunc<(i32, i64, i64), (i32,)>,
            visitor_erased_visit_i16: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_i32: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_i64: wasmtime::TypedFunc<(i32, i64), (i32,)>,
            visitor_erased_visit_i8: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_map: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_newtype_struct: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_none: wasmtime::TypedFunc<(i32,), (i32,)>,
            visitor_erased_visit_seq: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_some: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_str: wasmtime::TypedFunc<(i32, i32, i32), (i32,)>,
            visitor_erased_visit_string: wasmtime::TypedFunc<(i32, i32, i32), (i32,)>,
            visitor_erased_visit_u128: wasmtime::TypedFunc<(i32, i64, i64), (i32,)>,
            visitor_erased_visit_u16: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_u32: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_u64: wasmtime::TypedFunc<(i32, i64), (i32,)>,
            visitor_erased_visit_u8: wasmtime::TypedFunc<(i32, i32), (i32,)>,
            visitor_erased_visit_unit: wasmtime::TypedFunc<(i32,), (i32,)>,
        }
        impl<T> SerdeWasmGuest<T> {
            /// Adds any intrinsics, if necessary for this exported wasm
            /// functionality to the `linker` provided.
            ///
            /// The `get_state` closure is required to access the
            /// auxiliary data necessary for these wasm exports from
            /// the general store's state.
            pub fn add_to_linker(
                linker: &mut wasmtime::Linker<T>,
                get_state: impl Fn(&mut T) -> &mut SerdeWasmGuestData + Send + Sync + Copy + 'static,
            ) -> anyhow::Result<()> {
                linker.func_wrap(
                    "canonical_abi",
                    "resource_drop_de-value",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab0.remove(idx)?;
                        let wasm = match state.resource_slab0.drop(resource_idx) {
                            Some(wasm) => wasm,
                            None => return Ok(()),
                        };
                        let dtor = state.dtor0.expect("destructor not set yet");
                        dtor.call(&mut caller, wasm)?;
                        Ok(())
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_clone_de-value",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab0.get(idx)?;
                        state.resource_slab0.clone(resource_idx)?;
                        Ok(state.index_slab0.insert(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_get_de-value",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab0.get(idx)?;
                        Ok(state.resource_slab0.get(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_new_de-value",
                    move |mut caller: wasmtime::Caller<'_, T>, val: i32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.resource_slab0.insert(val);
                        Ok(state.index_slab0.insert(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_drop_deserialize-seed",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab1.remove(idx)?;
                        let wasm = match state.resource_slab1.drop(resource_idx) {
                            Some(wasm) => wasm,
                            None => return Ok(()),
                        };
                        let dtor = state.dtor1.expect("destructor not set yet");
                        dtor.call(&mut caller, wasm)?;
                        Ok(())
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_clone_deserialize-seed",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab1.get(idx)?;
                        state.resource_slab1.clone(resource_idx)?;
                        Ok(state.index_slab1.insert(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_get_deserialize-seed",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab1.get(idx)?;
                        Ok(state.resource_slab1.get(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_new_deserialize-seed",
                    move |mut caller: wasmtime::Caller<'_, T>, val: i32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.resource_slab1.insert(val);
                        Ok(state.index_slab1.insert(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_drop_visitor",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab2.remove(idx)?;
                        let wasm = match state.resource_slab2.drop(resource_idx) {
                            Some(wasm) => wasm,
                            None => return Ok(()),
                        };
                        let dtor = state.dtor2.expect("destructor not set yet");
                        dtor.call(&mut caller, wasm)?;
                        Ok(())
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_clone_visitor",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab2.get(idx)?;
                        state.resource_slab2.clone(resource_idx)?;
                        Ok(state.index_slab2.insert(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_get_visitor",
                    move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.index_slab2.get(idx)?;
                        Ok(state.resource_slab2.get(resource_idx))
                    },
                )?;
                linker.func_wrap(
                    "canonical_abi",
                    "resource_new_visitor",
                    move |mut caller: wasmtime::Caller<'_, T>, val: i32| {
                        let state = get_state(caller.data_mut());
                        let resource_idx = state.resource_slab2.insert(val);
                        Ok(state.index_slab2.insert(resource_idx))
                    },
                )?;
                Ok(())
            }
            /// Instantiates the provided `module` using the specified
            /// parameters, wrapping up the result in a structure that
            /// translates between wasm and the host.
            ///
            /// The `linker` provided will have intrinsics added to it
            /// automatically, so it's not necessary to call
            /// `add_to_linker` beforehand. This function will
            /// instantiate the `module` otherwise using `linker`, and
            /// both an instance of this structure and the underlying
            /// `wasmtime::Instance` will be returned.
            ///
            /// The `get_state` parameter is used to access the
            /// auxiliary state necessary for these wasm exports from
            /// the general store state `T`.
            pub fn instantiate(
                mut store: impl wasmtime::AsContextMut<Data = T>,
                module: &wasmtime::Module,
                linker: &mut wasmtime::Linker<T>,
                get_state: impl Fn(&mut T) -> &mut SerdeWasmGuestData + Send + Sync + Copy + 'static,
            ) -> anyhow::Result<(Self, wasmtime::Instance)> {
                Self::add_to_linker(linker, get_state)?;
                let instance = linker.instantiate(&mut store, module)?;
                Ok((Self::new(store, &instance, get_state)?, instance))
            }
            /// Low-level creation wrapper for wrapping up the exports
            /// of the `instance` provided in this structure of wasm
            /// exports.
            ///
            /// This function will extract exports from the `instance`
            /// defined within `store` and wrap them all up in the
            /// returned structure which can be used to interact with
            /// the wasm module.
            pub fn new(
                mut store: impl wasmtime::AsContextMut<Data = T>,
                instance: &wasmtime::Instance,
                get_state: impl Fn(&mut T) -> &mut SerdeWasmGuestData + Send + Sync + Copy + 'static,
            ) -> anyhow::Result<Self> {
                let mut store = store.as_context_mut();
                let canonical_abi_free = instance
                    .get_typed_func::<(i32, i32, i32), (), _>(&mut store, "canonical_abi_free")?;
                let canonical_abi_realloc = instance
                    .get_typed_func::<(i32, i32, i32, i32), i32, _>(
                        &mut store,
                        "canonical_abi_realloc",
                    )?;
                let deserialize_seed_erased_deserialize = instance
                    .get_typed_func::<(i32, i32), (i32,), _>(
                        &mut store,
                        "deserialize-seed::erased-deserialize",
                    )?;
                let memory = instance.get_memory(&mut store, "memory").ok_or_else(|| {
                    ::anyhow::private::must_use({
                        let error = ::anyhow::private::format_err(::core::fmt::Arguments::new_v1(
                            &["`memory` export not a memory"],
                            &[],
                        ));
                        error
                    })
                })?;
                let visitor_erased_expecting = instance
                    .get_typed_func::<(i32,), (i32,), _>(&mut store, "visitor::erased-expecting")?;
                let visitor_erased_visit_bool = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-bool",
                )?;
                let visitor_erased_visit_byte_buf = instance
                    .get_typed_func::<(i32, i32, i32), (i32,), _>(
                        &mut store,
                        "visitor::erased-visit-byte-buf",
                    )?;
                let visitor_erased_visit_bytes = instance
                    .get_typed_func::<(i32, i32, i32), (i32,), _>(
                        &mut store,
                        "visitor::erased-visit-bytes",
                    )?;
                let visitor_erased_visit_char = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-char",
                )?;
                let visitor_erased_visit_enum = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-enum",
                )?;
                let visitor_erased_visit_f32 = instance.get_typed_func::<(i32, f32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-f32",
                )?;
                let visitor_erased_visit_f64 = instance.get_typed_func::<(i32, f64), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-f64",
                )?;
                let visitor_erased_visit_i128 = instance
                    .get_typed_func::<(i32, i64, i64), (i32,), _>(
                        &mut store,
                        "visitor::erased-visit-i128",
                    )?;
                let visitor_erased_visit_i16 = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-i16",
                )?;
                let visitor_erased_visit_i32 = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-i32",
                )?;
                let visitor_erased_visit_i64 = instance.get_typed_func::<(i32, i64), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-i64",
                )?;
                let visitor_erased_visit_i8 = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-i8",
                )?;
                let visitor_erased_visit_map = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-map",
                )?;
                let visitor_erased_visit_newtype_struct = instance
                    .get_typed_func::<(i32, i32), (i32,), _>(
                        &mut store,
                        "visitor::erased-visit-newtype-struct",
                    )?;
                let visitor_erased_visit_none = instance.get_typed_func::<(i32,), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-none",
                )?;
                let visitor_erased_visit_seq = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-seq",
                )?;
                let visitor_erased_visit_some = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-some",
                )?;
                let visitor_erased_visit_str = instance
                    .get_typed_func::<(i32, i32, i32), (i32,), _>(
                        &mut store,
                        "visitor::erased-visit-str",
                    )?;
                let visitor_erased_visit_string = instance
                    .get_typed_func::<(i32, i32, i32), (i32,), _>(
                        &mut store,
                        "visitor::erased-visit-string",
                    )?;
                let visitor_erased_visit_u128 = instance
                    .get_typed_func::<(i32, i64, i64), (i32,), _>(
                        &mut store,
                        "visitor::erased-visit-u128",
                    )?;
                let visitor_erased_visit_u16 = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-u16",
                )?;
                let visitor_erased_visit_u32 = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-u32",
                )?;
                let visitor_erased_visit_u64 = instance.get_typed_func::<(i32, i64), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-u64",
                )?;
                let visitor_erased_visit_u8 = instance.get_typed_func::<(i32, i32), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-u8",
                )?;
                let visitor_erased_visit_unit = instance.get_typed_func::<(i32,), (i32,), _>(
                    &mut store,
                    "visitor::erased-visit-unit",
                )?;
                get_state(store.data_mut()).dtor0 = Some(
                    instance
                        .get_typed_func::<i32, (), _>(&mut store, "canonical_abi_drop_de-value")?,
                );
                get_state(store.data_mut()).dtor1 = Some(instance.get_typed_func::<i32, (), _>(
                    &mut store,
                    "canonical_abi_drop_deserialize-seed",
                )?);
                get_state(store.data_mut()).dtor2 = Some(
                    instance
                        .get_typed_func::<i32, (), _>(&mut store, "canonical_abi_drop_visitor")?,
                );
                Ok(SerdeWasmGuest {
                    canonical_abi_free,
                    canonical_abi_realloc,
                    deserialize_seed_erased_deserialize,
                    memory,
                    visitor_erased_expecting,
                    visitor_erased_visit_bool,
                    visitor_erased_visit_byte_buf,
                    visitor_erased_visit_bytes,
                    visitor_erased_visit_char,
                    visitor_erased_visit_enum,
                    visitor_erased_visit_f32,
                    visitor_erased_visit_f64,
                    visitor_erased_visit_i128,
                    visitor_erased_visit_i16,
                    visitor_erased_visit_i32,
                    visitor_erased_visit_i64,
                    visitor_erased_visit_i8,
                    visitor_erased_visit_map,
                    visitor_erased_visit_newtype_struct,
                    visitor_erased_visit_none,
                    visitor_erased_visit_seq,
                    visitor_erased_visit_some,
                    visitor_erased_visit_str,
                    visitor_erased_visit_string,
                    visitor_erased_visit_u128,
                    visitor_erased_visit_u16,
                    visitor_erased_visit_u32,
                    visitor_erased_visit_u64,
                    visitor_erased_visit_u8,
                    visitor_erased_visit_unit,
                    get_state: Box::new(get_state),
                })
            }
            pub fn deserialize_seed_erased_deserialize(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &DeserializeSeed,
                deserializer: DeserializerHandle,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab1
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab1
                    .insert(obj0.0);
                let DeserializerHandle { handle: handle1 } = deserializer;
                let (result2_0,) = self.deserialize_seed_erased_deserialize.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(handle1)),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_expecting(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
            ) -> Result<Option<String>, wasmtime::Trap> {
                let func_canonical_abi_free = &self.canonical_abi_free;
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self
                    .visitor_erased_expecting
                    .call(&mut caller, (handle0 as i32,))?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => None,
                    1 => Some({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 8)?;
                        let ptr5 = load3;
                        let len5 = load4;
                        let data5 = copy_slice(&mut caller, memory, ptr5, len5, 1)?;
                        func_canonical_abi_free.call(&mut caller, (ptr5, len5, 1))?;
                        String::from_utf8(data5)
                            .map_err(|_| wasmtime::Trap::new("invalid utf-8"))?
                    }),
                    _ => return Err(invalid_variant("option")),
                })
            }
            pub fn visitor_erased_visit_bool(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: bool,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_bool.call(
                    &mut caller,
                    (
                        handle0 as i32,
                        match v {
                            true => 1,
                            false => 0,
                        },
                    ),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_i8(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: i8,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_i8.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_i16(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: i16,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_i16.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_i32(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: i32,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_i32.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_i64(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: i64,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_i64.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i64(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_i128(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: I128,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let I128 { hi: hi1, lo: lo1 } = v;
                let (result2_0,) = self.visitor_erased_visit_i128.call(
                    &mut caller,
                    (
                        handle0 as i32,
                        wit_bindgen_wasmtime::rt::as_i64(hi1),
                        wit_bindgen_wasmtime::rt::as_i64(lo1),
                    ),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_u8(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: u8,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_u8.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_u16(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: u16,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_u16.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_u32(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: u32,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_u32.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_u64(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: u64,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_u64.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i64(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_u128(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: U128,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let U128 { hi: hi1, lo: lo1 } = v;
                let (result2_0,) = self.visitor_erased_visit_u128.call(
                    &mut caller,
                    (
                        handle0 as i32,
                        wit_bindgen_wasmtime::rt::as_i64(hi1),
                        wit_bindgen_wasmtime::rt::as_i64(lo1),
                    ),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_f32(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: f32,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self
                    .visitor_erased_visit_f32
                    .call(&mut caller, (handle0 as i32, v))?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_f64(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: f64,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self
                    .visitor_erased_visit_f64
                    .call(&mut caller, (handle0 as i32, v))?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_char(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: char,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self.visitor_erased_visit_char.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(v)),
                )?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_str(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: &str,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let func_canonical_abi_realloc = &self.canonical_abi_realloc;
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let vec1 = v;
                let ptr1 =
                    func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, vec1.len() as i32))?;
                memory
                    .data_mut(&mut caller)
                    .store_many(ptr1, vec1.as_bytes())?;
                let (result2_0,) = self
                    .visitor_erased_visit_str
                    .call(&mut caller, (handle0 as i32, ptr1, vec1.len() as i32))?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_string(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: &str,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let func_canonical_abi_realloc = &self.canonical_abi_realloc;
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let vec1 = v;
                let ptr1 =
                    func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, vec1.len() as i32))?;
                memory
                    .data_mut(&mut caller)
                    .store_many(ptr1, vec1.as_bytes())?;
                let (result2_0,) = self
                    .visitor_erased_visit_string
                    .call(&mut caller, (handle0 as i32, ptr1, vec1.len() as i32))?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_bytes(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: &[u8],
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let func_canonical_abi_realloc = &self.canonical_abi_realloc;
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let vec1 = v;
                let ptr1 = func_canonical_abi_realloc
                    .call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
                memory.data_mut(&mut caller).store_many(ptr1, &vec1)?;
                let (result2_0,) = self
                    .visitor_erased_visit_bytes
                    .call(&mut caller, (handle0 as i32, ptr1, vec1.len() as i32))?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_byte_buf(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                v: &[u8],
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let func_canonical_abi_realloc = &self.canonical_abi_realloc;
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let vec1 = v;
                let ptr1 = func_canonical_abi_realloc
                    .call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
                memory.data_mut(&mut caller).store_many(ptr1, &vec1)?;
                let (result2_0,) = self
                    .visitor_erased_visit_byte_buf
                    .call(&mut caller, (handle0 as i32, ptr1, vec1.len() as i32))?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_none(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self
                    .visitor_erased_visit_none
                    .call(&mut caller, (handle0 as i32,))?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_some(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                deserializer: DeserializerHandle,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let DeserializerHandle { handle: handle1 } = deserializer;
                let (result2_0,) = self.visitor_erased_visit_some.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(handle1)),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_unit(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let (result1_0,) = self
                    .visitor_erased_visit_unit
                    .call(&mut caller, (handle0 as i32,))?;
                let load2 = memory.data_mut(&mut caller).load::<u8>(result1_0 + 0)?;
                Ok(match i32::from(load2) {
                    0 => Ok({
                        let load3 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        let handle4 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load3 as u32)?;
                        DeValue(handle4)
                    }),
                    1 => Err({
                        let load5 = memory.data_mut(&mut caller).load::<i32>(result1_0 + 4)?;
                        DeErrorHandle { handle: load5 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_newtype_struct(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                deserializer: DeserializerHandle,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let DeserializerHandle { handle: handle1 } = deserializer;
                let (result2_0,) = self.visitor_erased_visit_newtype_struct.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(handle1)),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_seq(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                seq: SeqAccessHandle,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let SeqAccessHandle { handle: handle1 } = seq;
                let (result2_0,) = self.visitor_erased_visit_seq.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(handle1)),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_map(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                map: MapAccessHandle,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let MapAccessHandle { handle: handle1 } = map;
                let (result2_0,) = self.visitor_erased_visit_map.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(handle1)),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            pub fn visitor_erased_visit_enum(
                &self,
                mut caller: impl wasmtime::AsContextMut<Data = T>,
                self_: &Visitor,
                data: EnumAccessHandle,
            ) -> Result<Result<DeValue, DeErrorHandle>, wasmtime::Trap> {
                let memory = &self.memory;
                let obj0 = self_;
                (self.get_state)(caller.as_context_mut().data_mut())
                    .resource_slab2
                    .clone(obj0.0)?;
                let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                    .index_slab2
                    .insert(obj0.0);
                let EnumAccessHandle { handle: handle1 } = data;
                let (result2_0,) = self.visitor_erased_visit_enum.call(
                    &mut caller,
                    (handle0 as i32, wit_bindgen_wasmtime::rt::as_i32(handle1)),
                )?;
                let load3 = memory.data_mut(&mut caller).load::<u8>(result2_0 + 0)?;
                Ok(match i32::from(load3) {
                    0 => Ok({
                        let load4 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        let handle5 = (self.get_state)(caller.as_context_mut().data_mut())
                            .index_slab0
                            .remove(load4 as u32)?;
                        DeValue(handle5)
                    }),
                    1 => Err({
                        let load6 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 4)?;
                        DeErrorHandle { handle: load6 }
                    }),
                    _ => return Err(invalid_variant("expected")),
                })
            }
            /// Drops the host-owned handle to the resource
            /// specified.
            ///
            /// Note that this may execute the WebAssembly-defined
            /// destructor for this type. This also may not run
            /// the destructor if there are still other references
            /// to this type.
            pub fn drop_de_value(
                &self,
                mut store: impl wasmtime::AsContextMut<Data = T>,
                val: DeValue,
            ) -> Result<(), wasmtime::Trap> {
                let mut store = store.as_context_mut();
                let data = (self.get_state)(store.data_mut());
                let wasm = match data.resource_slab0.drop(val.0) {
                    Some(val) => val,
                    None => return Ok(()),
                };
                data.dtor0.unwrap().call(&mut store, wasm)?;
                Ok(())
            }
            /// Drops the host-owned handle to the resource
            /// specified.
            ///
            /// Note that this may execute the WebAssembly-defined
            /// destructor for this type. This also may not run
            /// the destructor if there are still other references
            /// to this type.
            pub fn drop_deserialize_seed(
                &self,
                mut store: impl wasmtime::AsContextMut<Data = T>,
                val: DeserializeSeed,
            ) -> Result<(), wasmtime::Trap> {
                let mut store = store.as_context_mut();
                let data = (self.get_state)(store.data_mut());
                let wasm = match data.resource_slab1.drop(val.0) {
                    Some(val) => val,
                    None => return Ok(()),
                };
                data.dtor1.unwrap().call(&mut store, wasm)?;
                Ok(())
            }
            /// Drops the host-owned handle to the resource
            /// specified.
            ///
            /// Note that this may execute the WebAssembly-defined
            /// destructor for this type. This also may not run
            /// the destructor if there are still other references
            /// to this type.
            pub fn drop_visitor(
                &self,
                mut store: impl wasmtime::AsContextMut<Data = T>,
                val: Visitor,
            ) -> Result<(), wasmtime::Trap> {
                let mut store = store.as_context_mut();
                let data = (self.get_state)(store.data_mut());
                let wasm = match data.resource_slab2.drop(val.0) {
                    Some(val) => val,
                    None => return Ok(()),
                };
                data.dtor2.unwrap().call(&mut store, wasm)?;
                Ok(())
            }
        }
        use wit_bindgen_wasmtime::rt::RawMem;
        use wit_bindgen_wasmtime::rt::invalid_variant;
        use wit_bindgen_wasmtime::rt::copy_slice;
    }
    const _ : & str = "record i128 {\n  hi: u64,\n  lo: u64,\n}\n\nrecord u128 {\n  hi: u64,\n  lo: u64,\n}\n\nresource de-value {}\n\nresource deserialize-seed {\n  erased-deserialize: func(deserializer: deserializer-handle) -> expected<de-value, de-error-handle>\n}\n\nresource visitor {\n  erased-expecting: func() -> option<string>\n  \n  erased-visit-bool: func(v: bool) -> expected<de-value, de-error-handle>\n  erased-visit-i8: func(v: s8) -> expected<de-value, de-error-handle>\n  erased-visit-i16: func(v: s16) -> expected<de-value, de-error-handle>\n  erased-visit-i32: func(v: s32) -> expected<de-value, de-error-handle>\n  erased-visit-i64: func(v: s64) -> expected<de-value, de-error-handle>\n  erased-visit-i128: func(v: i128) -> expected<de-value, de-error-handle>\n  erased-visit-u8: func(v: u8) -> expected<de-value, de-error-handle>\n  erased-visit-u16: func(v: u16) -> expected<de-value, de-error-handle>\n  erased-visit-u32: func(v: u32) -> expected<de-value, de-error-handle>\n  erased-visit-u64: func(v: u64) -> expected<de-value, de-error-handle>\n  erased-visit-u128: func(v: u128) -> expected<de-value, de-error-handle>\n  erased-visit-f32: func(v: float32) -> expected<de-value, de-error-handle>\n  erased-visit-f64: func(v: float64) -> expected<de-value, de-error-handle>\n  erased-visit-char: func(v: char) -> expected<de-value, de-error-handle>\n  erased-visit-str: func(v: string) -> expected<de-value, de-error-handle>\n  erased-visit-string: func(v: string) -> expected<de-value, de-error-handle>\n  erased-visit-bytes: func(v: list<u8>) -> expected<de-value, de-error-handle>\n  erased-visit-byte-buf: func(v: list<u8>) -> expected<de-value, de-error-handle>\n  erased-visit-none: func() -> expected<de-value, de-error-handle>\n  erased-visit-some: func(deserializer: deserializer-handle) -> expected<de-value, de-error-handle>\n  erased-visit-unit: func() -> expected<de-value, de-error-handle>\n  erased-visit-newtype-struct: func(deserializer: deserializer-handle) -> expected<de-value, de-error-handle>\n  erased-visit-seq: func(seq: seq-access-handle) -> expected<de-value, de-error-handle>\n  erased-visit-map: func(map: map-access-handle) -> expected<de-value, de-error-handle>\n  erased-visit-enum: func(data: enum-access-handle) -> expected<de-value, de-error-handle>\n}\n\nrecord de-error-handle {\n  %handle: s32\n}\n\nrecord deserializer-handle {\n  %handle: s32\n}\n\nrecord seq-access-handle {\n  %handle: s32\n}\n\nrecord map-access-handle {\n  %handle: s32\n}\n\nrecord enum-access-handle {\n  %handle: s32\n}\n" ;
}
