use std::{
    cell::RefCell,
    fmt::{self, Write},
};

mod any;
mod de;
mod error;
mod map;

wit_bindgen_rust::import!("src/serde-wasm-host.wit");
wit_bindgen_rust::export!("src/serde-wasm-guest.wit");

#[doc(hidden)]
pub struct DeValue {
    inner: RefCell<Option<de::Out>>,
}

#[doc(hidden)]
pub struct DeserializeSeed {
    inner: RefCell<&'static mut dyn de::DeserializeSeed<'static>>,
}

impl serde_wasm_guest::DeserializeSeed for DeserializeSeed {
    fn erased_deserialize(
        &self,
        deserializer: serde_wasm_guest::DeserializerHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(
            self.inner
                .borrow_mut()
                .erased_deserialize_seed(&mut Deserializer::from(deserializer)),
        )
    }
}

#[doc(hidden)]
pub struct Visitor {
    inner: RefCell<&'static mut dyn de::Visitor<'static>>,
}

fn map_erased_result(
    result: Result<de::Out, Error>,
) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
    match result {
        Ok(out) => Ok(wit_bindgen_rust::Handle::new(DeValue {
            inner: RefCell::new(Some(out)),
        })),
        Err(err) => Err(err.into()),
    }
}

fn map_bridge_result(
    result: Result<serde_wasm_host::DeValueHandle, serde_wasm_host::DeError>,
) -> Result<de::Out, Error> {
    match result {
        Ok(handle) => {
            let handle: wit_bindgen_rust::Handle<DeValue> =
                unsafe { wit_bindgen_rust::Handle::from_raw(handle.handle) };
            let out = handle.inner.borrow_mut().take().unwrap();
            Ok(out)
        },
        Err(err) => Err(Error { inner: err }),
    }
}

fn map_bridge_option_result(
    result: Result<Option<serde_wasm_host::DeValueHandle>, serde_wasm_host::DeError>,
) -> Result<Option<de::Out>, Error> {
    match result {
        Ok(Some(handle)) => {
            let handle: wit_bindgen_rust::Handle<DeValue> =
                unsafe { wit_bindgen_rust::Handle::from_raw(handle.handle) };
            let out = handle.inner.borrow_mut().take().unwrap();
            Ok(Some(out))
        },
        Ok(None) => Ok(None),
        Err(err) => Err(Error { inner: err }),
    }
}

fn map_bridge_option_pair_result(
    result: Result<
        Option<(
            serde_wasm_host::DeValueHandle,
            serde_wasm_host::DeValueHandle,
        )>,
        serde_wasm_host::DeError,
    >,
) -> Result<Option<(de::Out, de::Out)>, Error> {
    match result {
        Ok(Some((handle_a, handle_b))) => {
            let handle_a: wit_bindgen_rust::Handle<DeValue> =
                unsafe { wit_bindgen_rust::Handle::from_raw(handle_a.handle) };
            let out_a = handle_a.inner.borrow_mut().take().unwrap();

            let handle_b: wit_bindgen_rust::Handle<DeValue> =
                unsafe { wit_bindgen_rust::Handle::from_raw(handle_b.handle) };
            let out_b = handle_b.inner.borrow_mut().take().unwrap();

            Ok(Some((out_a, out_b)))
        },
        Ok(None) => Ok(None),
        Err(err) => Err(Error { inner: err }),
    }
}

fn map_bridge_enum_result(
    result: Result<
        (
            serde_wasm_host::DeValueHandle,
            serde_wasm_host::VariantAccess,
        ),
        serde_wasm_host::DeError,
    >,
) -> Result<(de::Out, VariantAccess), Error> {
    match result {
        Ok((handle, variant)) => {
            let handle: wit_bindgen_rust::Handle<DeValue> =
                unsafe { wit_bindgen_rust::Handle::from_raw(handle.handle) };
            let out = handle.inner.borrow_mut().take().unwrap();

            let variant = VariantAccess { inner: variant };

            Ok((out, variant))
        },
        Err(err) => Err(Error { inner: err }),
    }
}

impl From<Error> for serde_wasm_guest::DeErrorHandle {
    fn from(err: Error) -> Self {
        Self {
            handle: err.inner.into_raw(),
        }
    }
}

impl From<serde_wasm_guest::DeserializerHandle> for Deserializer {
    fn from(deserializer: serde_wasm_guest::DeserializerHandle) -> Self {
        Self {
            inner: unsafe { serde_wasm_host::Deserializer::from_raw(deserializer.handle) },
        }
    }
}

impl From<serde_wasm_guest::SeqAccessHandle> for SeqAccess {
    fn from(seq: serde_wasm_guest::SeqAccessHandle) -> Self {
        Self {
            inner: unsafe { serde_wasm_host::SeqAccess::from_raw(seq.handle) },
        }
    }
}

impl From<serde_wasm_guest::MapAccessHandle> for MapAccess {
    fn from(map: serde_wasm_guest::MapAccessHandle) -> Self {
        Self {
            inner: unsafe { serde_wasm_host::MapAccess::from_raw(map.handle) },
        }
    }
}

impl From<serde_wasm_guest::EnumAccessHandle> for EnumAccess {
    fn from(r#enum: serde_wasm_guest::EnumAccessHandle) -> Self {
        Self {
            inner: unsafe { serde_wasm_host::EnumAccess::from_raw(r#enum.handle) },
        }
    }
}

impl<'a, 'de> From<&'a mut dyn de::Visitor<'de>> for serde_wasm_host::VisitorHandle {
    fn from(visitor: &'a mut dyn de::Visitor<'de>) -> Self {
        let visitor: &'static mut dyn de::Visitor<'static> =
            unsafe { std::mem::transmute(visitor) };
        let visitor = Visitor {
            inner: RefCell::new(visitor),
        };

        let handle = wit_bindgen_rust::Handle::new(visitor);

        Self {
            handle: wit_bindgen_rust::Handle::into_raw(handle),
        }
    }
}

impl<'a, 'de> From<&'a mut dyn de::DeserializeSeed<'de>>
    for serde_wasm_host::DeserializeSeedHandle
{
    fn from(deserialize_seed: &'a mut dyn de::DeserializeSeed<'de>) -> Self {
        let deserialize_seed: &'static mut dyn de::DeserializeSeed<'static> =
            unsafe { std::mem::transmute(deserialize_seed) };
        let deserialize_seed = DeserializeSeed {
            inner: RefCell::new(deserialize_seed),
        };

        let handle = wit_bindgen_rust::Handle::new(deserialize_seed);

        Self {
            handle: wit_bindgen_rust::Handle::into_raw(handle),
        }
    }
}

impl serde_wasm_guest::Visitor for Visitor {
    fn erased_expecting(&self) -> Option<String> {
        struct Expecting<'a>(&'a Visitor);

        impl<'a> fmt::Display for Expecting<'a> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                self.0.inner.borrow().erased_expecting(fmt)
            }
        }

        let mut buffer = String::new();

        match buffer.write_fmt(format_args!("{}", Expecting(self))) {
            Ok(()) => Some(buffer),
            Err(_) => None,
        }
    }

    fn erased_visit_bool(
        &self,
        v: bool,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_bool(v))
    }

    fn erased_visit_i8(
        &self,
        v: i8,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_i8(v))
    }

    fn erased_visit_i16(
        &self,
        v: i16,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_i16(v))
    }

    fn erased_visit_i32(
        &self,
        v: i32,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_i32(v))
    }

    fn erased_visit_i64(
        &self,
        v: i64,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_i64(v))
    }

    fn erased_visit_i128(
        &self,
        v: serde_wasm_guest::I128,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        let v = unsafe {
            std::mem::transmute::<u128, i128>((u128::from(v.hi) << 64) | u128::from(v.lo))
        };

        map_erased_result(self.inner.borrow_mut().erased_visit_i128(v))
    }

    fn erased_visit_u8(
        &self,
        v: u8,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_u8(v))
    }

    fn erased_visit_u16(
        &self,
        v: u16,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_u16(v))
    }

    fn erased_visit_u32(
        &self,
        v: u32,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_u32(v))
    }

    fn erased_visit_u64(
        &self,
        v: u64,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_u64(v))
    }

    fn erased_visit_u128(
        &self,
        v: serde_wasm_guest::U128,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        let v = (u128::from(v.hi) << 64) | u128::from(v.lo);

        map_erased_result(self.inner.borrow_mut().erased_visit_u128(v))
    }

    fn erased_visit_f32(
        &self,
        v: f32,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_f32(v))
    }

    fn erased_visit_f64(
        &self,
        v: f64,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_f64(v))
    }

    fn erased_visit_char(
        &self,
        v: char,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_char(v))
    }

    fn erased_visit_str(
        &self,
        v: String,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_str(&v))
    }

    fn erased_visit_string(
        &self,
        v: String,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_string(v))
    }

    fn erased_visit_bytes(
        &self,
        v: Vec<u8>,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_bytes(&v))
    }

    fn erased_visit_byte_buf(
        &self,
        v: Vec<u8>,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_byte_buf(v))
    }

    fn erased_visit_none(
        &self,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_none())
    }

    fn erased_visit_some(
        &self,
        deserializer: serde_wasm_guest::DeserializerHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(
            self.inner
                .borrow_mut()
                .erased_visit_some(&mut Deserializer::from(deserializer)),
        )
    }

    fn erased_visit_unit(
        &self,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(self.inner.borrow_mut().erased_visit_unit())
    }

    fn erased_visit_newtype_struct(
        &self,
        deserializer: serde_wasm_guest::DeserializerHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(
            self.inner
                .borrow_mut()
                .erased_visit_newtype_struct(&mut Deserializer::from(deserializer)),
        )
    }

    fn erased_visit_seq(
        &self,
        seq: serde_wasm_guest::SeqAccessHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(
            self.inner
                .borrow_mut()
                .erased_visit_seq(&mut SeqAccess::from(seq)),
        )
    }

    fn erased_visit_map(
        &self,
        map: serde_wasm_guest::MapAccessHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(
            self.inner
                .borrow_mut()
                .erased_visit_map(&mut MapAccess::from(map)),
        )
    }

    fn erased_visit_enum(
        &self,
        data: serde_wasm_guest::EnumAccessHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        map_erased_result(
            self.inner
                .borrow_mut()
                .erased_visit_enum(&mut EnumAccess::from(data)),
        )
    }
}

struct SerdeWasmGuest {}

impl serde_wasm_guest::SerdeWasmGuest for SerdeWasmGuest {}

#[cfg(target_pointer_width = "32")]
impl From<usize> for serde_wasm_host::Usize {
    fn from(size: usize) -> Self {
        Self { size: size as _ }
    }
}

#[cfg(not(target_pointer_width = "32"))]
impl From<usize> for serde_wasm_host::Usize {
    fn from(size: usize) -> Self {
        extern "C" {
            fn usize_must_be_u32(size: usize) -> !;
        }

        unsafe { usize_must_be_u32(size) }
    }
}

impl From<serde_wasm_host::Usize> for usize {
    fn from(size: serde_wasm_host::Usize) -> Self {
        size.size as usize
    }
}

pub struct Error {
    inner: serde_wasm_host::DeError,
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Error").finish_non_exhaustive()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Error").finish_non_exhaustive()
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    #[cold]
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self {
            inner: serde_wasm_host::DeError::custom(&msg.to_string()),
        }
    }

    #[cold]
    fn invalid_type(unexp: serde::de::Unexpected, exp: &dyn serde::de::Expected) -> Self {
        Self {
            inner: serde_wasm_host::DeError::invalid_type(unexp.into(), &exp.to_string()),
        }
    }

    #[cold]
    fn invalid_value(unexp: serde::de::Unexpected, exp: &dyn serde::de::Expected) -> Self {
        Self {
            inner: serde_wasm_host::DeError::invalid_value(unexp.into(), &exp.to_string()),
        }
    }

    #[cold]
    fn invalid_length(len: usize, exp: &dyn serde::de::Expected) -> Self {
        Self {
            inner: serde_wasm_host::DeError::invalid_length(len.into(), &exp.to_string()),
        }
    }

    #[cold]
    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Self {
            inner: serde_wasm_host::DeError::unknown_variant(variant, expected),
        }
    }

    #[cold]
    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Self {
            inner: serde_wasm_host::DeError::unknown_field(field, expected),
        }
    }

    #[cold]
    fn missing_field(field: &'static str) -> Self {
        Self {
            inner: serde_wasm_host::DeError::missing_field(field),
        }
    }

    #[cold]
    fn duplicate_field(field: &'static str) -> Self {
        Self {
            inner: serde_wasm_host::DeError::duplicate_field(field),
        }
    }
}

impl<'a> From<serde::de::Unexpected<'a>> for serde_wasm_host::Unexpected<'a> {
    fn from(unexpected: serde::de::Unexpected<'a>) -> Self {
        use serde::de::Unexpected::*;

        match unexpected {
            Bool(b) => Self::Bool(b),
            Unsigned(u) => Self::Unsigned(u),
            Signed(s) => Self::Signed(s),
            Float(f) => Self::Float(f),
            Char(c) => Self::Char(c),
            Str(s) => Self::Str(s),
            Bytes(b) => Self::Bytes(b),
            Unit => Self::Unit,
            Option => Self::Option,
            NewtypeStruct => Self::NewtypeStruct,
            Seq => Self::Seq,
            Map => Self::Map,
            Enum => Self::Enum,
            UnitVariant => Self::UnitVariant,
            NewtypeVariant => Self::NewtypeVariant,
            TupleVariant => Self::TupleVariant,
            StructVariant => Self::StructVariant,
            Other(o) => Self::Other(o),
        }
    }
}

struct SeqAccess {
    inner: serde_wasm_host::SeqAccess,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    fn erased_next_element(
        &mut self,
        d: &mut dyn de::DeserializeSeed<'de>,
    ) -> Result<Option<de::Out>, Error> {
        map_bridge_option_result(self.inner.erased_next_element(d.into()))
    }

    fn erased_size_hint(&self) -> Option<usize> {
        self.inner.erased_size_hint().map(Into::into)
    }
}

struct MapAccess {
    inner: serde_wasm_host::MapAccess,
}

impl<'de> de::MapAccess<'de> for MapAccess {
    fn erased_next_key(
        &mut self,
        d: &mut dyn de::DeserializeSeed<'de>,
    ) -> Result<Option<de::Out>, Error> {
        map_bridge_option_result(self.inner.erased_next_key(d.into()))
    }

    fn erased_next_value(
        &mut self,
        d: &mut dyn de::DeserializeSeed<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_next_value(d.into()))
    }

    fn erased_next_entry(
        &mut self,
        key: &mut dyn de::DeserializeSeed<'de>,
        value: &mut dyn de::DeserializeSeed<'de>,
    ) -> Result<Option<(de::Out, de::Out)>, Error> {
        map_bridge_option_pair_result(self.inner.erased_next_entry(key.into(), value.into()))
    }

    fn erased_size_hint(&self) -> Option<usize> {
        self.inner.erased_size_hint().map(Into::into)
    }
}

struct EnumAccess {
    inner: serde_wasm_host::EnumAccess,
}

impl<'de> de::EnumAccess<'de> for EnumAccess {
    fn erased_variant_seed(
        &mut self,
        d: &mut dyn de::DeserializeSeed<'de>,
    ) -> Result<(de::Out, de::Variant<'de>), Error> {
        let (value, variant) = map_bridge_enum_result(self.inner.erased_variant_seed(d.into()))?;

        let erased_variant = de::Variant {
            data: unsafe { any::Any::new(variant) },
            unit_variant: {
                unsafe fn unit_variant(a: any::Any) -> Result<(), Error> {
                    let variant = a.take::<VariantAccess>();

                    variant
                        .inner
                        .unit_variant()
                        .map_err(|err| Error { inner: err })
                }
                unit_variant
            },
            visit_newtype: {
                unsafe fn visit_newtype(
                    a: any::Any,
                    seed: &mut dyn de::DeserializeSeed,
                ) -> Result<de::Out, Error> {
                    let variant = a.take::<VariantAccess>();

                    map_bridge_result(variant.inner.newtype_variant_seed(seed.into()))
                }
                visit_newtype
            },
            tuple_variant: {
                unsafe fn tuple_variant(
                    a: any::Any,
                    len: usize,
                    visitor: &mut dyn de::Visitor,
                ) -> Result<de::Out, Error> {
                    let variant = a.take::<VariantAccess>();

                    map_bridge_result(variant.inner.tuple_variant(len.into(), visitor.into()))
                }
                tuple_variant
            },
            struct_variant: {
                unsafe fn struct_variant(
                    a: any::Any,
                    fields: &'static [&'static str],
                    visitor: &mut dyn de::Visitor,
                ) -> Result<de::Out, Error> {
                    let variant = a.take::<VariantAccess>();

                    map_bridge_result(variant.inner.struct_variant(fields, visitor.into()))
                }
                struct_variant
            },
        };

        Ok((value, erased_variant))
    }
}

struct VariantAccess {
    inner: serde_wasm_host::VariantAccess,
}

struct Deserializer {
    inner: serde_wasm_host::Deserializer,
}

impl<'de> de::Deserializer<'de> for Deserializer {
    fn erased_deserialize_any(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_any(v.into()))
    }

    fn erased_deserialize_bool(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_bool(v.into()))
    }

    fn erased_deserialize_u8(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_u8(v.into()))
    }

    fn erased_deserialize_u16(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_u16(v.into()))
    }

    fn erased_deserialize_u32(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_u32(v.into()))
    }

    fn erased_deserialize_u64(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_u64(v.into()))
    }

    fn erased_deserialize_i8(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_i8(v.into()))
    }

    fn erased_deserialize_i16(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_i16(v.into()))
    }

    fn erased_deserialize_i32(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_i32(v.into()))
    }

    fn erased_deserialize_i64(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_i64(v.into()))
    }

    fn erased_deserialize_i128(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_i128(v.into()))
    }

    fn erased_deserialize_u128(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_u128(v.into()))
    }

    fn erased_deserialize_f32(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_f32(v.into()))
    }

    fn erased_deserialize_f64(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_f64(v.into()))
    }

    fn erased_deserialize_char(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_char(v.into()))
    }

    fn erased_deserialize_str(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_str(v.into()))
    }

    fn erased_deserialize_string(
        &mut self,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_string(v.into()))
    }

    fn erased_deserialize_bytes(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_bytes(v.into()))
    }

    fn erased_deserialize_byte_buf(
        &mut self,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_byte_buf(v.into()))
    }

    fn erased_deserialize_option(
        &mut self,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_option(v.into()))
    }

    fn erased_deserialize_unit(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_unit(v.into()))
    }

    fn erased_deserialize_unit_struct(
        &mut self,
        name: &'static str,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_unit_struct(name, v.into()))
    }

    fn erased_deserialize_newtype_struct(
        &mut self,
        name: &'static str,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_newtype_struct(name, v.into()))
    }

    fn erased_deserialize_seq(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_seq(v.into()))
    }

    fn erased_deserialize_tuple(
        &mut self,
        len: usize,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_tuple(len.into(), v.into()))
    }

    fn erased_deserialize_tuple_struct(
        &mut self,
        name: &'static str,
        len: usize,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(
            self.inner
                .erased_deserialize_tuple_struct(name, len.into(), v.into()),
        )
    }

    fn erased_deserialize_map(&mut self, v: &mut dyn de::Visitor<'de>) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_map(v.into()))
    }

    fn erased_deserialize_struct(
        &mut self,
        name: &'static str,
        fields: &'static [&'static str],
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_struct(name, fields, v.into()))
    }

    fn erased_deserialize_identifier(
        &mut self,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_identifier(v.into()))
    }

    fn erased_deserialize_enum(
        &mut self,
        name: &'static str,
        variants: &'static [&'static str],
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_enum(name, variants, v.into()))
    }

    fn erased_deserialize_ignored_any(
        &mut self,
        v: &mut dyn de::Visitor<'de>,
    ) -> Result<de::Out, Error> {
        map_bridge_result(self.inner.erased_deserialize_ignored_any(v.into()))
    }

    fn erased_is_human_readable(&self) -> bool {
        self.inner.erased_is_human_readable()
    }
}
