#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::{
    cell::RefCell,
    fmt::{self, Write},
};
mod any {
    use core::{any, mem, mem::MaybeUninit, ptr};
    pub struct Any {
        value: Value,
        drop: unsafe fn(&mut Value),
        fingerprint: Fingerprint,
    }
    union Value {
        ptr: *mut (),
        inline: [MaybeUninit<usize>; 2],
    }
    fn is_small<T>() -> bool {
        true && mem::size_of::<T>() <= mem::size_of::<Value>()
            && mem::align_of::<T>() <= mem::align_of::<Value>()
    }
    impl Any {
        pub(crate) unsafe fn new<T>(t: T) -> Self {
            let value: Value;
            let drop: unsafe fn(&mut Value);
            let fingerprint = Fingerprint::of::<T>();
            if is_small::<T>() {
                let mut inline = [MaybeUninit::uninit(); 2];
                unsafe { ptr::write(inline.as_mut_ptr() as *mut T, t) };
                value = Value { inline };
                unsafe fn inline_drop<T>(value: &mut Value) {
                    unsafe { ptr::drop_in_place(value.inline.as_mut_ptr() as *mut T) }
                }
                drop = inline_drop::<T>;
            } else {
                let ptr = Box::into_raw(Box::new(t)) as *mut ();
                value = Value { ptr };
                unsafe fn ptr_drop<T>(value: &mut Value) {
                    mem::drop(unsafe { Box::from_raw(value.ptr as *mut T) });
                }
                drop = ptr_drop::<T>;
            };
            Any {
                value,
                drop,
                fingerprint,
            }
        }
        pub(crate) unsafe fn view<T>(&mut self) -> &mut T {
            if true && self.fingerprint != Fingerprint::of::<T>() {
                self.invalid_cast_to::<T>();
            }
            let ptr = if is_small::<T>() {
                unsafe { self.value.inline.as_mut_ptr() as *mut T }
            } else {
                unsafe { self.value.ptr as *mut T }
            };
            unsafe { &mut *ptr }
        }
        pub(crate) unsafe fn take<T>(mut self) -> T {
            if true && self.fingerprint != Fingerprint::of::<T>() {
                self.invalid_cast_to::<T>();
            }
            if is_small::<T>() {
                let ptr = unsafe { self.value.inline.as_mut_ptr() as *mut T };
                let value = unsafe { ptr::read(ptr) };
                mem::forget(self);
                value
            } else {
                let ptr = unsafe { self.value.ptr as *mut T };
                let box_t = unsafe { Box::from_raw(ptr) };
                mem::forget(self);
                *box_t
            }
        }
        fn invalid_cast_to<T>(&self) -> ! {
            let from = self.fingerprint.type_name;
            let to = any::type_name::<T>();
            ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                &["invalid cast: ", " to "],
                &[
                    ::core::fmt::ArgumentV1::new_display(&from),
                    ::core::fmt::ArgumentV1::new_display(&to),
                ],
            ));
        }
    }
    impl Drop for Any {
        fn drop(&mut self) {
            unsafe { (self.drop)(&mut self.value) }
        }
    }
    struct Fingerprint {
        size: usize,
        align: usize,
        type_name: &'static str,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Fingerprint {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Fingerprint {
                    size: ref __self_0_0,
                    align: ref __self_0_1,
                    type_name: ref __self_0_2,
                } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f, "Fingerprint");
                    let _ = ::core::fmt::DebugStruct::field(
                        debug_trait_builder,
                        "size",
                        &&(*__self_0_0),
                    );
                    let _ = ::core::fmt::DebugStruct::field(
                        debug_trait_builder,
                        "align",
                        &&(*__self_0_1),
                    );
                    let _ = ::core::fmt::DebugStruct::field(
                        debug_trait_builder,
                        "type_name",
                        &&(*__self_0_2),
                    );
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Fingerprint {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Fingerprint {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<&'static str>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Fingerprint {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Fingerprint {
        #[inline]
        fn eq(&self, other: &Fingerprint) -> bool {
            match *other {
                Fingerprint {
                    size: ref __self_1_0,
                    align: ref __self_1_1,
                    type_name: ref __self_1_2,
                } => match *self {
                    Fingerprint {
                        size: ref __self_0_0,
                        align: ref __self_0_1,
                        type_name: ref __self_0_2,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &Fingerprint) -> bool {
            match *other {
                Fingerprint {
                    size: ref __self_1_0,
                    align: ref __self_1_1,
                    type_name: ref __self_1_2,
                } => match *self {
                    Fingerprint {
                        size: ref __self_0_0,
                        align: ref __self_0_1,
                        type_name: ref __self_0_2,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                    }
                },
            }
        }
    }
    impl Fingerprint {
        fn of<T>() -> Fingerprint {
            Fingerprint {
                size: mem::size_of::<T>(),
                align: mem::align_of::<T>(),
                type_name: any::type_name::<T>(),
            }
        }
    }
}
mod de {
    use crate::{
        any::Any,
        error::Error,
        map::{OptionExt, ResultExt},
    };
    use core::fmt::{self, Display};
    use serde::serde_if_integer128;
    /// Deserialize a value of type `T` from the given trait object.
    ///
    /// ```rust
    /// use erased_serde::Deserializer;
    /// use std::collections::BTreeMap as Map;
    ///
    /// fn main() {
    ///     static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
    ///     static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];
    ///
    ///     // Construct some deserializers.
    ///     let json = &mut serde_json::Deserializer::from_slice(JSON);
    ///     let cbor = &mut serde_cbor::Deserializer::from_slice(CBOR);
    ///
    ///     // The values in this map are boxed trait objects, which is not possible
    ///     // with the normal serde::Deserializer because of object safety.
    ///     let mut formats: Map<&str, Box<dyn Deserializer>> = Map::new();
    ///     formats.insert("json", Box::new(<dyn Deserializer>::erase(json)));
    ///     formats.insert("cbor", Box::new(<dyn Deserializer>::erase(cbor)));
    ///
    ///     // Pick a Deserializer out of the formats map.
    ///     let format = formats.get_mut("json").unwrap();
    ///
    ///     let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();
    ///
    ///     println!("{}", data["A"] + data["B"]);
    /// }
    /// ```
    pub fn deserialize<'de, T>(deserializer: &mut dyn Deserializer<'de>) -> Result<T, Error>
    where
        T: serde::Deserialize<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
    pub trait DeserializeSeed<'de> {
        fn erased_deserialize_seed(&mut self, d: &mut dyn Deserializer<'de>) -> Result<Out, Error>;
    }
    /// An object-safe equivalent of Serde's `Deserializer` trait.
    ///
    /// Any implementation of Serde's `Deserializer` can be converted to an
    /// `&erased_serde::Deserializer` or `Box<erased_serde::Deserializer>` trait
    /// object using `erased_serde::Deserializer::erase`.
    ///
    /// ```rust
    /// use erased_serde::Deserializer;
    /// use std::collections::BTreeMap as Map;
    ///
    /// fn main() {
    ///     static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
    ///     static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];
    ///
    ///     // Construct some deserializers.
    ///     let json = &mut serde_json::Deserializer::from_slice(JSON);
    ///     let cbor = &mut serde_cbor::Deserializer::from_slice(CBOR);
    ///
    ///     // The values in this map are boxed trait objects, which is not possible
    ///     // with the normal serde::Deserializer because of object safety.
    ///     let mut formats: Map<&str, Box<dyn Deserializer>> = Map::new();
    ///     formats.insert("json", Box::new(<dyn Deserializer>::erase(json)));
    ///     formats.insert("cbor", Box::new(<dyn Deserializer>::erase(cbor)));
    ///
    ///     // Pick a Deserializer out of the formats map.
    ///     let format = formats.get_mut("json").unwrap();
    ///
    ///     let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();
    ///
    ///     println!("{}", data["A"] + data["B"]);
    /// }
    /// ```
    pub trait Deserializer<'de> {
        fn erased_deserialize_any(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_bool(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_u8(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_u16(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_u32(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_u64(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_i8(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_i16(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_i32(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_i64(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_i128(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_u128(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_f32(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_f64(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_char(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_str(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_string(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_bytes(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_byte_buf(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_option(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_unit(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_unit_struct(
            &mut self,
            name: &'static str,
            v: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>;
        fn erased_deserialize_newtype_struct(
            &mut self,
            name: &'static str,
            v: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>;
        fn erased_deserialize_seq(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_tuple(
            &mut self,
            len: usize,
            v: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>;
        fn erased_deserialize_tuple_struct(
            &mut self,
            name: &'static str,
            len: usize,
            v: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>;
        fn erased_deserialize_map(&mut self, v: &mut dyn Visitor<'de>) -> Result<Out, Error>;
        fn erased_deserialize_struct(
            &mut self,
            name: &'static str,
            fields: &'static [&'static str],
            v: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>;
        fn erased_deserialize_identifier(&mut self, v: &mut dyn Visitor<'de>)
            -> Result<Out, Error>;
        fn erased_deserialize_enum(
            &mut self,
            name: &'static str,
            variants: &'static [&'static str],
            v: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>;
        fn erased_deserialize_ignored_any(
            &mut self,
            v: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>;
        fn erased_is_human_readable(&self) -> bool;
    }
    pub trait Visitor<'de> {
        fn erased_expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result;
        fn erased_visit_bool(&mut self, v: bool) -> Result<Out, Error>;
        fn erased_visit_i8(&mut self, v: i8) -> Result<Out, Error>;
        fn erased_visit_i16(&mut self, v: i16) -> Result<Out, Error>;
        fn erased_visit_i32(&mut self, v: i32) -> Result<Out, Error>;
        fn erased_visit_i64(&mut self, v: i64) -> Result<Out, Error>;
        fn erased_visit_u8(&mut self, v: u8) -> Result<Out, Error>;
        fn erased_visit_u16(&mut self, v: u16) -> Result<Out, Error>;
        fn erased_visit_u32(&mut self, v: u32) -> Result<Out, Error>;
        fn erased_visit_u64(&mut self, v: u64) -> Result<Out, Error>;
        fn erased_visit_i128(&mut self, v: i128) -> Result<Out, Error>;
        fn erased_visit_u128(&mut self, v: u128) -> Result<Out, Error>;
        fn erased_visit_f32(&mut self, v: f32) -> Result<Out, Error>;
        fn erased_visit_f64(&mut self, v: f64) -> Result<Out, Error>;
        fn erased_visit_char(&mut self, v: char) -> Result<Out, Error>;
        fn erased_visit_str(&mut self, v: &str) -> Result<Out, Error>;
        fn erased_visit_borrowed_str(&mut self, v: &'de str) -> Result<Out, Error>;
        fn erased_visit_string(&mut self, v: String) -> Result<Out, Error>;
        fn erased_visit_bytes(&mut self, v: &[u8]) -> Result<Out, Error>;
        fn erased_visit_borrowed_bytes(&mut self, v: &'de [u8]) -> Result<Out, Error>;
        fn erased_visit_byte_buf(&mut self, v: Vec<u8>) -> Result<Out, Error>;
        fn erased_visit_none(&mut self) -> Result<Out, Error>;
        fn erased_visit_some(&mut self, d: &mut dyn Deserializer<'de>) -> Result<Out, Error>;
        fn erased_visit_unit(&mut self) -> Result<Out, Error>;
        fn erased_visit_newtype_struct(
            &mut self,
            d: &mut dyn Deserializer<'de>,
        ) -> Result<Out, Error>;
        fn erased_visit_seq(&mut self, s: &mut dyn SeqAccess<'de>) -> Result<Out, Error>;
        fn erased_visit_map(&mut self, m: &mut dyn MapAccess<'de>) -> Result<Out, Error>;
        fn erased_visit_enum(&mut self, e: &mut dyn EnumAccess<'de>) -> Result<Out, Error>;
    }
    pub trait SeqAccess<'de> {
        fn erased_next_element(
            &mut self,
            d: &mut dyn DeserializeSeed<'de>,
        ) -> Result<Option<Out>, Error>;
        fn erased_size_hint(&self) -> Option<usize>;
    }
    pub trait MapAccess<'de> {
        fn erased_next_key(
            &mut self,
            d: &mut dyn DeserializeSeed<'de>,
        ) -> Result<Option<Out>, Error>;
        fn erased_next_value(&mut self, d: &mut dyn DeserializeSeed<'de>) -> Result<Out, Error>;
        fn erased_next_entry(
            &mut self,
            key: &mut dyn DeserializeSeed<'de>,
            value: &mut dyn DeserializeSeed<'de>,
        ) -> Result<Option<(Out, Out)>, Error>;
        fn erased_size_hint(&self) -> Option<usize>;
    }
    pub trait EnumAccess<'de> {
        fn erased_variant_seed(
            &mut self,
            d: &mut dyn DeserializeSeed<'de>,
        ) -> Result<(Out, Variant<'de>), Error>;
    }
    impl<'de> dyn Deserializer<'de> {
        /// Convert any Serde `Deserializer` to a trait object.
        ///
        /// ```rust
        /// use erased_serde::Deserializer;
        /// use std::collections::BTreeMap as Map;
        ///
        /// fn main() {
        ///     static JSON: &'static [u8] = br#"{"A": 65, "B": 66}"#;
        ///     static CBOR: &'static [u8] = &[162, 97, 65, 24, 65, 97, 66, 24, 66];
        ///
        ///     // Construct some deserializers.
        ///     let json = &mut serde_json::Deserializer::from_slice(JSON);
        ///     let cbor = &mut serde_cbor::Deserializer::from_slice(CBOR);
        ///
        ///     // The values in this map are boxed trait objects, which is not possible
        ///     // with the normal serde::Deserializer because of object safety.
        ///     let mut formats: Map<&str, Box<dyn Deserializer>> = Map::new();
        ///     formats.insert("json", Box::new(<dyn Deserializer>::erase(json)));
        ///     formats.insert("cbor", Box::new(<dyn Deserializer>::erase(cbor)));
        ///
        ///     // Pick a Deserializer out of the formats map.
        ///     let format = formats.get_mut("json").unwrap();
        ///
        ///     let data: Map<String, usize> = erased_serde::deserialize(format).unwrap();
        ///
        ///     println!("{}", data["A"] + data["B"]);
        /// }
        /// ```
        pub fn erase<D>(deserializer: D) -> erase::Deserializer<D>
        where
            D: serde::Deserializer<'de>,
        {
            erase::Deserializer {
                state: Some(deserializer),
            }
        }
    }
    pub struct Out(Any);
    impl Out {
        unsafe fn new<T>(t: T) -> Self {
            Out(unsafe { Any::new(t) })
        }
        unsafe fn take<T>(self) -> T {
            unsafe { self.0.take() }
        }
    }
    mod erase {
        pub struct DeserializeSeed<D> {
            pub(crate) state: Option<D>,
        }
        impl<D> DeserializeSeed<D> {
            pub(crate) fn take(&mut self) -> D {
                self.state.take().unwrap()
            }
        }
        pub struct Deserializer<D> {
            pub(crate) state: Option<D>,
        }
        impl<D> Deserializer<D> {
            pub(crate) fn take(&mut self) -> D {
                self.state.take().unwrap()
            }
            pub(crate) fn as_ref(&self) -> &D {
                self.state.as_ref().unwrap()
            }
        }
        pub struct Visitor<D> {
            pub(crate) state: Option<D>,
        }
        impl<D> Visitor<D> {
            pub(crate) fn take(&mut self) -> D {
                self.state.take().unwrap()
            }
            pub(crate) fn as_ref(&self) -> &D {
                self.state.as_ref().unwrap()
            }
        }
        pub struct SeqAccess<D> {
            pub(crate) state: D,
        }
        impl<D> SeqAccess<D> {
            pub(crate) fn as_ref(&self) -> &D {
                &self.state
            }
            pub(crate) fn as_mut(&mut self) -> &mut D {
                &mut self.state
            }
        }
        pub struct MapAccess<D> {
            pub(crate) state: D,
        }
        impl<D> MapAccess<D> {
            pub(crate) fn as_ref(&self) -> &D {
                &self.state
            }
            pub(crate) fn as_mut(&mut self) -> &mut D {
                &mut self.state
            }
        }
        pub struct EnumAccess<D> {
            pub(crate) state: Option<D>,
        }
        impl<D> EnumAccess<D> {
            pub(crate) fn take(&mut self) -> D {
                self.state.take().unwrap()
            }
        }
    }
    impl<'de, T> DeserializeSeed<'de> for erase::DeserializeSeed<T>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        fn erased_deserialize_seed(
            &mut self,
            deserializer: &mut dyn Deserializer<'de>,
        ) -> Result<Out, Error> {
            unsafe { self.take().deserialize(deserializer).unsafe_map(Out::new) }
        }
    }
    impl<'de, T> Deserializer<'de> for erase::Deserializer<T>
    where
        T: serde::Deserializer<'de>,
    {
        fn erased_deserialize_i128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_i128(visitor).map_err(erase)
        }
        fn erased_deserialize_u128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_u128(visitor).map_err(erase)
        }
        fn erased_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_any(visitor).map_err(erase)
        }
        fn erased_deserialize_bool(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_bool(visitor).map_err(erase)
        }
        fn erased_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_u8(visitor).map_err(erase)
        }
        fn erased_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_u16(visitor).map_err(erase)
        }
        fn erased_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_u32(visitor).map_err(erase)
        }
        fn erased_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_u64(visitor).map_err(erase)
        }
        fn erased_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_i8(visitor).map_err(erase)
        }
        fn erased_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_i16(visitor).map_err(erase)
        }
        fn erased_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_i32(visitor).map_err(erase)
        }
        fn erased_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_i64(visitor).map_err(erase)
        }
        fn erased_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_f32(visitor).map_err(erase)
        }
        fn erased_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_f64(visitor).map_err(erase)
        }
        fn erased_deserialize_char(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_char(visitor).map_err(erase)
        }
        fn erased_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_str(visitor).map_err(erase)
        }
        fn erased_deserialize_string(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_string(visitor).map_err(erase)
        }
        fn erased_deserialize_bytes(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_bytes(visitor).map_err(erase)
        }
        fn erased_deserialize_byte_buf(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_byte_buf(visitor).map_err(erase)
        }
        fn erased_deserialize_option(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_option(visitor).map_err(erase)
        }
        fn erased_deserialize_unit(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_unit(visitor).map_err(erase)
        }
        fn erased_deserialize_unit_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take()
                .deserialize_unit_struct(name, visitor)
                .map_err(erase)
        }
        fn erased_deserialize_newtype_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take()
                .deserialize_newtype_struct(name, visitor)
                .map_err(erase)
        }
        fn erased_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_seq(visitor).map_err(erase)
        }
        fn erased_deserialize_tuple(
            &mut self,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_tuple(len, visitor).map_err(erase)
        }
        fn erased_deserialize_tuple_struct(
            &mut self,
            name: &'static str,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take()
                .deserialize_tuple_struct(name, len, visitor)
                .map_err(erase)
        }
        fn erased_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            self.take().deserialize_map(visitor).map_err(erase)
        }
        fn erased_deserialize_struct(
            &mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take()
                .deserialize_struct(name, fields, visitor)
                .map_err(erase)
        }
        fn erased_deserialize_identifier(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_identifier(visitor).map_err(erase)
        }
        fn erased_deserialize_enum(
            &mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take()
                .deserialize_enum(name, variants, visitor)
                .map_err(erase)
        }
        fn erased_deserialize_ignored_any(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            self.take().deserialize_ignored_any(visitor).map_err(erase)
        }
        fn erased_is_human_readable(&self) -> bool {
            self.as_ref().is_human_readable()
        }
    }
    impl<'de, T> Visitor<'de> for erase::Visitor<T>
    where
        T: serde::de::Visitor<'de>,
    {
        fn erased_visit_i128(&mut self, v: i128) -> Result<Out, Error> {
            unsafe { self.take().visit_i128(v).unsafe_map(Out::new) }
        }
        fn erased_visit_u128(&mut self, v: u128) -> Result<Out, Error> {
            unsafe { self.take().visit_u128(v).unsafe_map(Out::new) }
        }
        fn erased_expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            self.as_ref().expecting(formatter)
        }
        fn erased_visit_bool(&mut self, v: bool) -> Result<Out, Error> {
            unsafe { self.take().visit_bool(v).unsafe_map(Out::new) }
        }
        fn erased_visit_i8(&mut self, v: i8) -> Result<Out, Error> {
            unsafe { self.take().visit_i8(v).unsafe_map(Out::new) }
        }
        fn erased_visit_i16(&mut self, v: i16) -> Result<Out, Error> {
            unsafe { self.take().visit_i16(v).unsafe_map(Out::new) }
        }
        fn erased_visit_i32(&mut self, v: i32) -> Result<Out, Error> {
            unsafe { self.take().visit_i32(v).unsafe_map(Out::new) }
        }
        fn erased_visit_i64(&mut self, v: i64) -> Result<Out, Error> {
            unsafe { self.take().visit_i64(v).unsafe_map(Out::new) }
        }
        fn erased_visit_u8(&mut self, v: u8) -> Result<Out, Error> {
            unsafe { self.take().visit_u8(v).unsafe_map(Out::new) }
        }
        fn erased_visit_u16(&mut self, v: u16) -> Result<Out, Error> {
            unsafe { self.take().visit_u16(v).unsafe_map(Out::new) }
        }
        fn erased_visit_u32(&mut self, v: u32) -> Result<Out, Error> {
            unsafe { self.take().visit_u32(v).unsafe_map(Out::new) }
        }
        fn erased_visit_u64(&mut self, v: u64) -> Result<Out, Error> {
            unsafe { self.take().visit_u64(v).unsafe_map(Out::new) }
        }
        fn erased_visit_f32(&mut self, v: f32) -> Result<Out, Error> {
            unsafe { self.take().visit_f32(v).unsafe_map(Out::new) }
        }
        fn erased_visit_f64(&mut self, v: f64) -> Result<Out, Error> {
            unsafe { self.take().visit_f64(v).unsafe_map(Out::new) }
        }
        fn erased_visit_char(&mut self, v: char) -> Result<Out, Error> {
            unsafe { self.take().visit_char(v).unsafe_map(Out::new) }
        }
        fn erased_visit_str(&mut self, v: &str) -> Result<Out, Error> {
            unsafe { self.take().visit_str(v).unsafe_map(Out::new) }
        }
        fn erased_visit_borrowed_str(&mut self, v: &'de str) -> Result<Out, Error> {
            unsafe { self.take().visit_borrowed_str(v).unsafe_map(Out::new) }
        }
        fn erased_visit_string(&mut self, v: String) -> Result<Out, Error> {
            unsafe { self.take().visit_string(v).unsafe_map(Out::new) }
        }
        fn erased_visit_bytes(&mut self, v: &[u8]) -> Result<Out, Error> {
            unsafe { self.take().visit_bytes(v).unsafe_map(Out::new) }
        }
        fn erased_visit_borrowed_bytes(&mut self, v: &'de [u8]) -> Result<Out, Error> {
            unsafe { self.take().visit_borrowed_bytes(v).unsafe_map(Out::new) }
        }
        fn erased_visit_byte_buf(&mut self, v: Vec<u8>) -> Result<Out, Error> {
            unsafe { self.take().visit_byte_buf(v).unsafe_map(Out::new) }
        }
        fn erased_visit_none(&mut self) -> Result<Out, Error> {
            unsafe { self.take().visit_none().unsafe_map(Out::new) }
        }
        fn erased_visit_some(
            &mut self,
            deserializer: &mut dyn Deserializer<'de>,
        ) -> Result<Out, Error> {
            unsafe { self.take().visit_some(deserializer).unsafe_map(Out::new) }
        }
        fn erased_visit_unit(&mut self) -> Result<Out, Error> {
            unsafe { self.take().visit_unit().unsafe_map(Out::new) }
        }
        fn erased_visit_newtype_struct(
            &mut self,
            deserializer: &mut dyn Deserializer<'de>,
        ) -> Result<Out, Error> {
            unsafe {
                self.take()
                    .visit_newtype_struct(deserializer)
                    .unsafe_map(Out::new)
            }
        }
        fn erased_visit_seq(&mut self, seq: &mut dyn SeqAccess<'de>) -> Result<Out, Error> {
            unsafe { self.take().visit_seq(seq).unsafe_map(Out::new) }
        }
        fn erased_visit_map(&mut self, map: &mut dyn MapAccess<'de>) -> Result<Out, Error> {
            unsafe { self.take().visit_map(map).unsafe_map(Out::new) }
        }
        fn erased_visit_enum(&mut self, data: &mut dyn EnumAccess<'de>) -> Result<Out, Error> {
            unsafe { self.take().visit_enum(data).unsafe_map(Out::new) }
        }
    }
    impl<'de, T> SeqAccess<'de> for erase::SeqAccess<T>
    where
        T: serde::de::SeqAccess<'de>,
    {
        fn erased_next_element(
            &mut self,
            seed: &mut dyn DeserializeSeed<'de>,
        ) -> Result<Option<Out>, Error> {
            self.as_mut().next_element_seed(seed).map_err(erase)
        }
        fn erased_size_hint(&self) -> Option<usize> {
            self.as_ref().size_hint()
        }
    }
    impl<'de, T> MapAccess<'de> for erase::MapAccess<T>
    where
        T: serde::de::MapAccess<'de>,
    {
        fn erased_next_key(
            &mut self,
            seed: &mut dyn DeserializeSeed<'de>,
        ) -> Result<Option<Out>, Error> {
            self.as_mut().next_key_seed(seed).map_err(erase)
        }
        fn erased_next_value(&mut self, seed: &mut dyn DeserializeSeed<'de>) -> Result<Out, Error> {
            self.as_mut().next_value_seed(seed).map_err(erase)
        }
        fn erased_next_entry(
            &mut self,
            k: &mut dyn DeserializeSeed<'de>,
            v: &mut dyn DeserializeSeed<'de>,
        ) -> Result<Option<(Out, Out)>, Error> {
            self.as_mut().next_entry_seed(k, v).map_err(erase)
        }
        fn erased_size_hint(&self) -> Option<usize> {
            self.as_ref().size_hint()
        }
    }
    impl<'de, T> EnumAccess<'de> for erase::EnumAccess<T>
    where
        T: serde::de::EnumAccess<'de>,
    {
        fn erased_variant_seed(
            &mut self,
            seed: &mut dyn DeserializeSeed<'de>,
        ) -> Result<(Out, Variant<'de>), Error> {
            self.take()
                .variant_seed(seed)
                .map(|(out, variant)| {
                    use serde::de::VariantAccess;
                    let erased = Variant {
                        data: unsafe { Any::new(variant) },
                        unit_variant: {
                            unsafe fn unit_variant<'de, T>(a: Any) -> Result<(), Error>
                            where
                                T: serde::de::EnumAccess<'de>,
                            {
                                unsafe { a.take::<T::Variant>().unit_variant().map_err(erase) }
                            }
                            unit_variant::<T>
                        },
                        visit_newtype: {
                            unsafe fn visit_newtype<'de, T>(
                                a: Any,
                                seed: &mut dyn DeserializeSeed<'de>,
                            ) -> Result<Out, Error>
                            where
                                T: serde::de::EnumAccess<'de>,
                            {
                                unsafe {
                                    a.take::<T::Variant>()
                                        .newtype_variant_seed(seed)
                                        .map_err(erase)
                                }
                            }
                            visit_newtype::<T>
                        },
                        tuple_variant: {
                            unsafe fn tuple_variant<'de, T>(
                                a: Any,
                                len: usize,
                                visitor: &mut dyn Visitor<'de>,
                            ) -> Result<Out, Error>
                            where
                                T: serde::de::EnumAccess<'de>,
                            {
                                unsafe {
                                    a.take::<T::Variant>()
                                        .tuple_variant(len, visitor)
                                        .map_err(erase)
                                }
                            }
                            tuple_variant::<T>
                        },
                        struct_variant: {
                            unsafe fn struct_variant<'de, T>(
                                a: Any,
                                fields: &'static [&'static str],
                                visitor: &mut dyn Visitor<'de>,
                            ) -> Result<Out, Error>
                            where
                                T: serde::de::EnumAccess<'de>,
                            {
                                unsafe {
                                    a.take::<T::Variant>()
                                        .struct_variant(fields, visitor)
                                        .map_err(erase)
                                }
                            }
                            struct_variant::<T>
                        },
                    };
                    (out, erased)
                })
                .map_err(erase)
        }
    }
    impl<'de, 'a> serde::de::DeserializeSeed<'de> for &'a mut dyn DeserializeSeed<'de> {
        type Value = Out;
        fn deserialize<D>(self, deserializer: D) -> Result<Out, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let mut erased = erase::Deserializer {
                state: Some(deserializer),
            };
            self.erased_deserialize_seed(&mut erased).map_err(unerase)
        }
    }
    impl<'de, 'a> serde::Deserializer<'de> for &'a mut dyn Deserializer<'de> {
        type Error = Error;
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de, 'a> serde::Deserializer<'de> for &'a mut (dyn Deserializer<'de> + Send) {
        type Error = Error;
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de, 'a> serde::Deserializer<'de> for &'a mut (dyn Deserializer<'de> + Sync) {
        type Error = Error;
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de, 'a> serde::Deserializer<'de> for &'a mut (dyn Deserializer<'de> + Send + Sync) {
        type Error = Error;
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de> serde::Deserializer<'de> for Box<dyn Deserializer<'de>> {
        type Error = Error;
        fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            mut self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de> serde::Deserializer<'de> for Box<dyn Deserializer<'de> + Send> {
        type Error = Error;
        fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            mut self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de> serde::Deserializer<'de> for Box<dyn Deserializer<'de> + Sync> {
        type Error = Error;
        fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            mut self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de> serde::Deserializer<'de> for Box<dyn Deserializer<'de> + Send + Sync> {
        type Error = Error;
        fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bool(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i8(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i16(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_i128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_u128(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f32(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_f64(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_char(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_str(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_string(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_bytes(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_byte_buf(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_option(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_unit_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_unit_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_newtype_struct<V>(
            mut self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_newtype_struct(name, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_seq(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple(len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_tuple_struct<V>(
            mut self,
            name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_tuple_struct(name, len, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_map(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_struct<V>(
            mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_struct(name, fields, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_identifier(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_enum<V>(
            mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_enum(name, variants, &mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe {
                self.erased_deserialize_ignored_any(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn is_human_readable(&self) -> bool {
            self.erased_is_human_readable()
        }
    }
    impl<'de, 'a> serde::de::Visitor<'de> for &'a mut dyn Visitor<'de> {
        type Value = Out;
        fn visit_i128<E>(self, v: i128) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_i128(v).map_err(unerase)
        }
        fn visit_u128<E>(self, v: u128) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_u128(v).map_err(unerase)
        }
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            (**self).erased_expecting(formatter)
        }
        fn visit_bool<E>(self, v: bool) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_bool(v).map_err(unerase)
        }
        fn visit_i8<E>(self, v: i8) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_i8(v).map_err(unerase)
        }
        fn visit_i16<E>(self, v: i16) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_i16(v).map_err(unerase)
        }
        fn visit_i32<E>(self, v: i32) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_i32(v).map_err(unerase)
        }
        fn visit_i64<E>(self, v: i64) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_i64(v).map_err(unerase)
        }
        fn visit_u8<E>(self, v: u8) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_u8(v).map_err(unerase)
        }
        fn visit_u16<E>(self, v: u16) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_u16(v).map_err(unerase)
        }
        fn visit_u32<E>(self, v: u32) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_u32(v).map_err(unerase)
        }
        fn visit_u64<E>(self, v: u64) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_u64(v).map_err(unerase)
        }
        fn visit_f32<E>(self, v: f32) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_f32(v).map_err(unerase)
        }
        fn visit_f64<E>(self, v: f64) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_f64(v).map_err(unerase)
        }
        fn visit_char<E>(self, v: char) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_char(v).map_err(unerase)
        }
        fn visit_str<E>(self, v: &str) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_str(v).map_err(unerase)
        }
        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_borrowed_str(v).map_err(unerase)
        }
        fn visit_string<E>(self, v: String) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_string(v).map_err(unerase)
        }
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_bytes(v).map_err(unerase)
        }
        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_borrowed_bytes(v).map_err(unerase)
        }
        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_byte_buf(v).map_err(unerase)
        }
        fn visit_none<E>(self) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_none().map_err(unerase)
        }
        fn visit_some<D>(self, deserializer: D) -> Result<Out, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let mut erased = erase::Deserializer {
                state: Some(deserializer),
            };
            self.erased_visit_some(&mut erased).map_err(unerase)
        }
        fn visit_unit<E>(self) -> Result<Out, E>
        where
            E: serde::de::Error,
        {
            self.erased_visit_unit().map_err(unerase)
        }
        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Out, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let mut erased = erase::Deserializer {
                state: Some(deserializer),
            };
            self.erased_visit_newtype_struct(&mut erased)
                .map_err(unerase)
        }
        fn visit_seq<V>(self, seq: V) -> Result<Out, V::Error>
        where
            V: serde::de::SeqAccess<'de>,
        {
            let mut erased = erase::SeqAccess { state: seq };
            self.erased_visit_seq(&mut erased).map_err(unerase)
        }
        fn visit_map<V>(self, map: V) -> Result<Out, V::Error>
        where
            V: serde::de::MapAccess<'de>,
        {
            let mut erased = erase::MapAccess { state: map };
            self.erased_visit_map(&mut erased).map_err(unerase)
        }
        fn visit_enum<V>(self, data: V) -> Result<Out, V::Error>
        where
            V: serde::de::EnumAccess<'de>,
        {
            let mut erased = erase::EnumAccess { state: Some(data) };
            self.erased_visit_enum(&mut erased).map_err(unerase)
        }
    }
    impl<'de, 'a> serde::de::SeqAccess<'de> for &'a mut dyn SeqAccess<'de> {
        type Error = Error;
        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where
            T: serde::de::DeserializeSeed<'de>,
        {
            let mut seed = erase::DeserializeSeed { state: Some(seed) };
            unsafe {
                (**self)
                    .erased_next_element(&mut seed)
                    .map(|opt| opt.unsafe_map(Out::take))
            }
        }
        fn size_hint(&self) -> Option<usize> {
            (**self).erased_size_hint()
        }
    }
    impl<'de, 'a> serde::de::MapAccess<'de> for &'a mut dyn MapAccess<'de> {
        type Error = Error;
        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where
            K: serde::de::DeserializeSeed<'de>,
        {
            let mut erased = erase::DeserializeSeed { state: Some(seed) };
            unsafe {
                (**self)
                    .erased_next_key(&mut erased)
                    .map(|opt| opt.unsafe_map(Out::take))
            }
        }
        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where
            V: serde::de::DeserializeSeed<'de>,
        {
            let mut erased = erase::DeserializeSeed { state: Some(seed) };
            unsafe {
                (**self)
                    .erased_next_value(&mut erased)
                    .unsafe_map(Out::take)
            }
        }
        fn size_hint(&self) -> Option<usize> {
            (**self).erased_size_hint()
        }
    }
    impl<'de, 'a> serde::de::EnumAccess<'de> for &'a mut dyn EnumAccess<'de> {
        type Error = Error;
        type Variant = Variant<'de>;
        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where
            V: serde::de::DeserializeSeed<'de>,
        {
            let mut erased = erase::DeserializeSeed { state: Some(seed) };
            match self.erased_variant_seed(&mut erased) {
                Ok((out, variant)) => Ok((unsafe { out.take() }, variant)),
                Err(err) => Err(err),
            }
        }
    }
    pub struct Variant<'de> {
        data: Any,
        unit_variant: unsafe fn(Any) -> Result<(), Error>,
        visit_newtype: unsafe fn(Any, seed: &mut dyn DeserializeSeed<'de>) -> Result<Out, Error>,
        tuple_variant:
            unsafe fn(Any, len: usize, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error>,
        struct_variant: unsafe fn(
            Any,
            fields: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error>,
    }
    impl<'de> serde::de::VariantAccess<'de> for Variant<'de> {
        type Error = Error;
        fn unit_variant(self) -> Result<(), Error> {
            unsafe { (self.unit_variant)(self.data) }
        }
        fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where
            T: serde::de::DeserializeSeed<'de>,
        {
            let mut erased = erase::DeserializeSeed { state: Some(seed) };
            unsafe { (self.visit_newtype)(self.data, &mut erased).unsafe_map(Out::take) }
        }
        fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe { (self.tuple_variant)(self.data, len, &mut erased).unsafe_map(Out::take) }
        }
        fn struct_variant<V>(
            self,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let mut erased = erase::Visitor {
                state: Some(visitor),
            };
            unsafe { (self.struct_variant)(self.data, fields, &mut erased).unsafe_map(Out::take) }
        }
    }
    impl<'de, 'a> Deserializer<'de> for Box<dyn Deserializer<'de> + 'a> {
        fn erased_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_any(visitor)
        }
        fn erased_deserialize_bool(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bool(visitor)
        }
        fn erased_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u8(visitor)
        }
        fn erased_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u16(visitor)
        }
        fn erased_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u32(visitor)
        }
        fn erased_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u64(visitor)
        }
        fn erased_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i8(visitor)
        }
        fn erased_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i16(visitor)
        }
        fn erased_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i32(visitor)
        }
        fn erased_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i64(visitor)
        }
        fn erased_deserialize_i128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_i128(visitor)
        }
        fn erased_deserialize_u128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_u128(visitor)
        }
        fn erased_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f32(visitor)
        }
        fn erased_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f64(visitor)
        }
        fn erased_deserialize_char(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_char(visitor)
        }
        fn erased_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_str(visitor)
        }
        fn erased_deserialize_string(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_string(visitor)
        }
        fn erased_deserialize_bytes(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bytes(visitor)
        }
        fn erased_deserialize_byte_buf(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_byte_buf(visitor)
        }
        fn erased_deserialize_option(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_option(visitor)
        }
        fn erased_deserialize_unit(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit(visitor)
        }
        fn erased_deserialize_unit_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit_struct(name, visitor)
        }
        fn erased_deserialize_newtype_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_newtype_struct(name, visitor)
        }
        fn erased_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_seq(visitor)
        }
        fn erased_deserialize_tuple(
            &mut self,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple(len, visitor)
        }
        fn erased_deserialize_tuple_struct(
            &mut self,
            name: &'static str,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple_struct(name, len, visitor)
        }
        fn erased_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_map(visitor)
        }
        fn erased_deserialize_struct(
            &mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_struct(name, fields, visitor)
        }
        fn erased_deserialize_identifier(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_identifier(visitor)
        }
        fn erased_deserialize_enum(
            &mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_enum(name, variants, visitor)
        }
        fn erased_deserialize_ignored_any(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_ignored_any(visitor)
        }
        fn erased_is_human_readable(&self) -> bool {
            (**self).erased_is_human_readable()
        }
    }
    impl<'de, 'a> Deserializer<'de> for Box<dyn Deserializer<'de> + Send + 'a> {
        fn erased_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_any(visitor)
        }
        fn erased_deserialize_bool(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bool(visitor)
        }
        fn erased_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u8(visitor)
        }
        fn erased_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u16(visitor)
        }
        fn erased_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u32(visitor)
        }
        fn erased_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u64(visitor)
        }
        fn erased_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i8(visitor)
        }
        fn erased_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i16(visitor)
        }
        fn erased_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i32(visitor)
        }
        fn erased_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i64(visitor)
        }
        fn erased_deserialize_i128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_i128(visitor)
        }
        fn erased_deserialize_u128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_u128(visitor)
        }
        fn erased_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f32(visitor)
        }
        fn erased_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f64(visitor)
        }
        fn erased_deserialize_char(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_char(visitor)
        }
        fn erased_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_str(visitor)
        }
        fn erased_deserialize_string(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_string(visitor)
        }
        fn erased_deserialize_bytes(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bytes(visitor)
        }
        fn erased_deserialize_byte_buf(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_byte_buf(visitor)
        }
        fn erased_deserialize_option(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_option(visitor)
        }
        fn erased_deserialize_unit(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit(visitor)
        }
        fn erased_deserialize_unit_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit_struct(name, visitor)
        }
        fn erased_deserialize_newtype_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_newtype_struct(name, visitor)
        }
        fn erased_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_seq(visitor)
        }
        fn erased_deserialize_tuple(
            &mut self,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple(len, visitor)
        }
        fn erased_deserialize_tuple_struct(
            &mut self,
            name: &'static str,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple_struct(name, len, visitor)
        }
        fn erased_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_map(visitor)
        }
        fn erased_deserialize_struct(
            &mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_struct(name, fields, visitor)
        }
        fn erased_deserialize_identifier(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_identifier(visitor)
        }
        fn erased_deserialize_enum(
            &mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_enum(name, variants, visitor)
        }
        fn erased_deserialize_ignored_any(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_ignored_any(visitor)
        }
        fn erased_is_human_readable(&self) -> bool {
            (**self).erased_is_human_readable()
        }
    }
    impl<'de, 'a> Deserializer<'de> for Box<dyn Deserializer<'de> + Sync + 'a> {
        fn erased_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_any(visitor)
        }
        fn erased_deserialize_bool(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bool(visitor)
        }
        fn erased_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u8(visitor)
        }
        fn erased_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u16(visitor)
        }
        fn erased_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u32(visitor)
        }
        fn erased_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u64(visitor)
        }
        fn erased_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i8(visitor)
        }
        fn erased_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i16(visitor)
        }
        fn erased_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i32(visitor)
        }
        fn erased_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i64(visitor)
        }
        fn erased_deserialize_i128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_i128(visitor)
        }
        fn erased_deserialize_u128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_u128(visitor)
        }
        fn erased_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f32(visitor)
        }
        fn erased_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f64(visitor)
        }
        fn erased_deserialize_char(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_char(visitor)
        }
        fn erased_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_str(visitor)
        }
        fn erased_deserialize_string(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_string(visitor)
        }
        fn erased_deserialize_bytes(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bytes(visitor)
        }
        fn erased_deserialize_byte_buf(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_byte_buf(visitor)
        }
        fn erased_deserialize_option(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_option(visitor)
        }
        fn erased_deserialize_unit(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit(visitor)
        }
        fn erased_deserialize_unit_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit_struct(name, visitor)
        }
        fn erased_deserialize_newtype_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_newtype_struct(name, visitor)
        }
        fn erased_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_seq(visitor)
        }
        fn erased_deserialize_tuple(
            &mut self,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple(len, visitor)
        }
        fn erased_deserialize_tuple_struct(
            &mut self,
            name: &'static str,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple_struct(name, len, visitor)
        }
        fn erased_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_map(visitor)
        }
        fn erased_deserialize_struct(
            &mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_struct(name, fields, visitor)
        }
        fn erased_deserialize_identifier(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_identifier(visitor)
        }
        fn erased_deserialize_enum(
            &mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_enum(name, variants, visitor)
        }
        fn erased_deserialize_ignored_any(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_ignored_any(visitor)
        }
        fn erased_is_human_readable(&self) -> bool {
            (**self).erased_is_human_readable()
        }
    }
    impl<'de, 'a> Deserializer<'de> for Box<dyn Deserializer<'de> + Send + Sync + 'a> {
        fn erased_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_any(visitor)
        }
        fn erased_deserialize_bool(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bool(visitor)
        }
        fn erased_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u8(visitor)
        }
        fn erased_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u16(visitor)
        }
        fn erased_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u32(visitor)
        }
        fn erased_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u64(visitor)
        }
        fn erased_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i8(visitor)
        }
        fn erased_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i16(visitor)
        }
        fn erased_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i32(visitor)
        }
        fn erased_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i64(visitor)
        }
        fn erased_deserialize_i128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_i128(visitor)
        }
        fn erased_deserialize_u128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_u128(visitor)
        }
        fn erased_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f32(visitor)
        }
        fn erased_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f64(visitor)
        }
        fn erased_deserialize_char(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_char(visitor)
        }
        fn erased_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_str(visitor)
        }
        fn erased_deserialize_string(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_string(visitor)
        }
        fn erased_deserialize_bytes(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bytes(visitor)
        }
        fn erased_deserialize_byte_buf(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_byte_buf(visitor)
        }
        fn erased_deserialize_option(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_option(visitor)
        }
        fn erased_deserialize_unit(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit(visitor)
        }
        fn erased_deserialize_unit_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit_struct(name, visitor)
        }
        fn erased_deserialize_newtype_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_newtype_struct(name, visitor)
        }
        fn erased_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_seq(visitor)
        }
        fn erased_deserialize_tuple(
            &mut self,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple(len, visitor)
        }
        fn erased_deserialize_tuple_struct(
            &mut self,
            name: &'static str,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple_struct(name, len, visitor)
        }
        fn erased_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_map(visitor)
        }
        fn erased_deserialize_struct(
            &mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_struct(name, fields, visitor)
        }
        fn erased_deserialize_identifier(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_identifier(visitor)
        }
        fn erased_deserialize_enum(
            &mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_enum(name, variants, visitor)
        }
        fn erased_deserialize_ignored_any(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_ignored_any(visitor)
        }
        fn erased_is_human_readable(&self) -> bool {
            (**self).erased_is_human_readable()
        }
    }
    impl<'de, 'a, T: ?Sized + Deserializer<'de>> Deserializer<'de> for &'a mut T {
        fn erased_deserialize_any(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_any(visitor)
        }
        fn erased_deserialize_bool(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bool(visitor)
        }
        fn erased_deserialize_u8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u8(visitor)
        }
        fn erased_deserialize_u16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u16(visitor)
        }
        fn erased_deserialize_u32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u32(visitor)
        }
        fn erased_deserialize_u64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_u64(visitor)
        }
        fn erased_deserialize_i8(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i8(visitor)
        }
        fn erased_deserialize_i16(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i16(visitor)
        }
        fn erased_deserialize_i32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i32(visitor)
        }
        fn erased_deserialize_i64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_i64(visitor)
        }
        fn erased_deserialize_i128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_i128(visitor)
        }
        fn erased_deserialize_u128(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_u128(visitor)
        }
        fn erased_deserialize_f32(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f32(visitor)
        }
        fn erased_deserialize_f64(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_f64(visitor)
        }
        fn erased_deserialize_char(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_char(visitor)
        }
        fn erased_deserialize_str(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_str(visitor)
        }
        fn erased_deserialize_string(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_string(visitor)
        }
        fn erased_deserialize_bytes(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_bytes(visitor)
        }
        fn erased_deserialize_byte_buf(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_byte_buf(visitor)
        }
        fn erased_deserialize_option(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_option(visitor)
        }
        fn erased_deserialize_unit(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit(visitor)
        }
        fn erased_deserialize_unit_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_unit_struct(name, visitor)
        }
        fn erased_deserialize_newtype_struct(
            &mut self,
            name: &'static str,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_newtype_struct(name, visitor)
        }
        fn erased_deserialize_seq(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_seq(visitor)
        }
        fn erased_deserialize_tuple(
            &mut self,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple(len, visitor)
        }
        fn erased_deserialize_tuple_struct(
            &mut self,
            name: &'static str,
            len: usize,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_tuple_struct(name, len, visitor)
        }
        fn erased_deserialize_map(&mut self, visitor: &mut dyn Visitor<'de>) -> Result<Out, Error> {
            (**self).erased_deserialize_map(visitor)
        }
        fn erased_deserialize_struct(
            &mut self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_struct(name, fields, visitor)
        }
        fn erased_deserialize_identifier(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_identifier(visitor)
        }
        fn erased_deserialize_enum(
            &mut self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_enum(name, variants, visitor)
        }
        fn erased_deserialize_ignored_any(
            &mut self,
            visitor: &mut dyn Visitor<'de>,
        ) -> Result<Out, Error> {
            (**self).erased_deserialize_ignored_any(visitor)
        }
        fn erased_is_human_readable(&self) -> bool {
            (**self).erased_is_human_readable()
        }
    }
    fn erase<E>(e: E) -> Error
    where
        E: Display,
    {
        serde::de::Error::custom(e)
    }
    fn unerase<E>(e: Error) -> E
    where
        E: serde::de::Error,
    {
        E::custom(e)
    }
}
mod error {
    pub(crate) use crate::Error;
}
mod map {
    pub(crate) trait ResultExt<T, E> {
        unsafe fn unsafe_map<U>(self, op: unsafe fn(T) -> U) -> Result<U, E>;
    }
    impl<T, E> ResultExt<T, E> for Result<T, E> {
        unsafe fn unsafe_map<U>(self, op: unsafe fn(T) -> U) -> Result<U, E> {
            match self {
                Ok(t) => Ok(unsafe { op(t) }),
                Err(e) => Err(e),
            }
        }
    }
    pub(crate) trait OptionExt<T> {
        unsafe fn unsafe_map<U>(self, op: unsafe fn(T) -> U) -> Option<U>;
    }
    impl<T> OptionExt<T> for Option<T> {
        unsafe fn unsafe_map<U>(self, op: unsafe fn(T) -> U) -> Option<U> {
            match self {
                Some(t) => Some(unsafe { op(t) }),
                None => None,
            }
        }
    }
}
#[allow(clippy::all)]
mod serde_wasm_host {
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
                Unexpected::Unsigned(e) => f.debug_tuple("Unexpected::Unsigned").field(e).finish(),
                Unexpected::Signed(e) => f.debug_tuple("Unexpected::Signed").field(e).finish(),
                Unexpected::Float(e) => f.debug_tuple("Unexpected::Float").field(e).finish(),
                Unexpected::Char(e) => f.debug_tuple("Unexpected::Char").field(e).finish(),
                Unexpected::Str(e) => f.debug_tuple("Unexpected::Str").field(e).finish(),
                Unexpected::Bytes(e) => f.debug_tuple("Unexpected::Bytes").field(e).finish(),
                Unexpected::Unit => f.debug_tuple("Unexpected::Unit").finish(),
                Unexpected::Option => f.debug_tuple("Unexpected::Option").finish(),
                Unexpected::NewtypeStruct => f.debug_tuple("Unexpected::NewtypeStruct").finish(),
                Unexpected::Seq => f.debug_tuple("Unexpected::Seq").finish(),
                Unexpected::Map => f.debug_tuple("Unexpected::Map").finish(),
                Unexpected::Enum => f.debug_tuple("Unexpected::Enum").finish(),
                Unexpected::UnitVariant => f.debug_tuple("Unexpected::UnitVariant").finish(),
                Unexpected::NewtypeVariant => f.debug_tuple("Unexpected::NewtypeVariant").finish(),
                Unexpected::TupleVariant => f.debug_tuple("Unexpected::TupleVariant").finish(),
                Unexpected::StructVariant => f.debug_tuple("Unexpected::StructVariant").finish(),
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
    #[repr(transparent)]
    pub struct DeError(i32);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for DeError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                DeError(ref __self_0_0) => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_tuple(f, "DeError");
                    let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                    ::core::fmt::DebugTuple::finish(debug_trait_builder)
                }
            }
        }
    }
    impl DeError {
        pub unsafe fn from_raw(raw: i32) -> Self {
            Self(raw)
        }
        pub fn into_raw(self) -> i32 {
            let ret = self.0;
            core::mem::forget(self);
            return ret;
        }
        pub fn as_raw(&self) -> i32 {
            self.0
        }
    }
    impl Drop for DeError {
        fn drop(&mut self) {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_drop_de-error"]
                fn close(fd: i32);
            }
            unsafe {
                close(self.0);
            }
        }
    }
    impl Clone for DeError {
        fn clone(&self) -> Self {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_clone_de-error"]
                fn clone(val: i32) -> i32;
            }
            unsafe { Self(clone(self.0)) }
        }
    }
    #[repr(transparent)]
    pub struct Deserializer(i32);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Deserializer {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Deserializer(ref __self_0_0) => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_tuple(f, "Deserializer");
                    let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                    ::core::fmt::DebugTuple::finish(debug_trait_builder)
                }
            }
        }
    }
    impl Deserializer {
        pub unsafe fn from_raw(raw: i32) -> Self {
            Self(raw)
        }
        pub fn into_raw(self) -> i32 {
            let ret = self.0;
            core::mem::forget(self);
            return ret;
        }
        pub fn as_raw(&self) -> i32 {
            self.0
        }
    }
    impl Drop for Deserializer {
        fn drop(&mut self) {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_drop_deserializer"]
                fn close(fd: i32);
            }
            unsafe {
                close(self.0);
            }
        }
    }
    impl Clone for Deserializer {
        fn clone(&self) -> Self {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_clone_deserializer"]
                fn clone(val: i32) -> i32;
            }
            unsafe { Self(clone(self.0)) }
        }
    }
    #[repr(transparent)]
    pub struct SeqAccess(i32);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for SeqAccess {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                SeqAccess(ref __self_0_0) => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_tuple(f, "SeqAccess");
                    let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                    ::core::fmt::DebugTuple::finish(debug_trait_builder)
                }
            }
        }
    }
    impl SeqAccess {
        pub unsafe fn from_raw(raw: i32) -> Self {
            Self(raw)
        }
        pub fn into_raw(self) -> i32 {
            let ret = self.0;
            core::mem::forget(self);
            return ret;
        }
        pub fn as_raw(&self) -> i32 {
            self.0
        }
    }
    impl Drop for SeqAccess {
        fn drop(&mut self) {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_drop_seq-access"]
                fn close(fd: i32);
            }
            unsafe {
                close(self.0);
            }
        }
    }
    impl Clone for SeqAccess {
        fn clone(&self) -> Self {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_clone_seq-access"]
                fn clone(val: i32) -> i32;
            }
            unsafe { Self(clone(self.0)) }
        }
    }
    #[repr(transparent)]
    pub struct MapAccess(i32);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for MapAccess {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                MapAccess(ref __self_0_0) => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_tuple(f, "MapAccess");
                    let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                    ::core::fmt::DebugTuple::finish(debug_trait_builder)
                }
            }
        }
    }
    impl MapAccess {
        pub unsafe fn from_raw(raw: i32) -> Self {
            Self(raw)
        }
        pub fn into_raw(self) -> i32 {
            let ret = self.0;
            core::mem::forget(self);
            return ret;
        }
        pub fn as_raw(&self) -> i32 {
            self.0
        }
    }
    impl Drop for MapAccess {
        fn drop(&mut self) {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_drop_map-access"]
                fn close(fd: i32);
            }
            unsafe {
                close(self.0);
            }
        }
    }
    impl Clone for MapAccess {
        fn clone(&self) -> Self {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_clone_map-access"]
                fn clone(val: i32) -> i32;
            }
            unsafe { Self(clone(self.0)) }
        }
    }
    #[repr(transparent)]
    pub struct EnumAccess(i32);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for EnumAccess {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                EnumAccess(ref __self_0_0) => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_tuple(f, "EnumAccess");
                    let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                    ::core::fmt::DebugTuple::finish(debug_trait_builder)
                }
            }
        }
    }
    impl EnumAccess {
        pub unsafe fn from_raw(raw: i32) -> Self {
            Self(raw)
        }
        pub fn into_raw(self) -> i32 {
            let ret = self.0;
            core::mem::forget(self);
            return ret;
        }
        pub fn as_raw(&self) -> i32 {
            self.0
        }
    }
    impl Drop for EnumAccess {
        fn drop(&mut self) {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_drop_enum-access"]
                fn close(fd: i32);
            }
            unsafe {
                close(self.0);
            }
        }
    }
    impl Clone for EnumAccess {
        fn clone(&self) -> Self {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_clone_enum-access"]
                fn clone(val: i32) -> i32;
            }
            unsafe { Self(clone(self.0)) }
        }
    }
    #[repr(transparent)]
    pub struct VariantAccess(i32);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for VariantAccess {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                VariantAccess(ref __self_0_0) => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_tuple(f, "VariantAccess");
                    let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                    ::core::fmt::DebugTuple::finish(debug_trait_builder)
                }
            }
        }
    }
    impl VariantAccess {
        pub unsafe fn from_raw(raw: i32) -> Self {
            Self(raw)
        }
        pub fn into_raw(self) -> i32 {
            let ret = self.0;
            core::mem::forget(self);
            return ret;
        }
        pub fn as_raw(&self) -> i32 {
            self.0
        }
    }
    impl Drop for VariantAccess {
        fn drop(&mut self) {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_drop_variant-access"]
                fn close(fd: i32);
            }
            unsafe {
                close(self.0);
            }
        }
    }
    impl Clone for VariantAccess {
        fn clone(&self) -> Self {
            #[link(wasm_import_module = "canonical_abi")]
            extern "C" {
                #[link_name = "resource_clone_variant-access"]
                fn clone(val: i32) -> i32;
            }
            unsafe { Self(clone(self.0)) }
        }
    }
    impl DeError {
        pub fn custom(msg: &str) -> DeError {
            unsafe {
                let vec0 = msg;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::custom"]
                    fn wit_import(_: i32, _: i32) -> i32;
                }
                let ret = wit_import(ptr0, len0);
                DeError(ret)
            }
        }
    }
    impl DeError {
        pub fn invalid_type(unexp: Unexpected<'_>, exp: &str) -> DeError {
            unsafe {
                let (result3_0, result3_1, result3_2) = match unexp {
                    Unexpected::Bool(e) => (
                        0i32,
                        i64::from(match e {
                            true => 1,
                            false => 0,
                        }),
                        0i32,
                    ),
                    Unexpected::Unsigned(e) => (1i32, wit_bindgen_rust::rt::as_i64(e), 0i32),
                    Unexpected::Signed(e) => (2i32, wit_bindgen_rust::rt::as_i64(e), 0i32),
                    Unexpected::Float(e) => (
                        3i32,
                        (wit_bindgen_rust::rt::as_f64(e)).to_bits() as i64,
                        0i32,
                    ),
                    Unexpected::Char(e) => (4i32, i64::from(wit_bindgen_rust::rt::as_i32(e)), 0i32),
                    Unexpected::Str(e) => {
                        let vec0 = e;
                        let ptr0 = vec0.as_ptr() as i32;
                        let len0 = vec0.len() as i32;
                        (5i32, i64::from(ptr0), len0)
                    }
                    Unexpected::Bytes(e) => {
                        let vec1 = e;
                        let ptr1 = vec1.as_ptr() as i32;
                        let len1 = vec1.len() as i32;
                        (6i32, i64::from(ptr1), len1)
                    }
                    Unexpected::Unit => {
                        let e = ();
                        {
                            let () = e;
                            (7i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Option => {
                        let e = ();
                        {
                            let () = e;
                            (8i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::NewtypeStruct => {
                        let e = ();
                        {
                            let () = e;
                            (9i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Seq => {
                        let e = ();
                        {
                            let () = e;
                            (10i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Map => {
                        let e = ();
                        {
                            let () = e;
                            (11i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Enum => {
                        let e = ();
                        {
                            let () = e;
                            (12i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::UnitVariant => {
                        let e = ();
                        {
                            let () = e;
                            (13i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::NewtypeVariant => {
                        let e = ();
                        {
                            let () = e;
                            (14i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::TupleVariant => {
                        let e = ();
                        {
                            let () = e;
                            (15i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::StructVariant => {
                        let e = ();
                        {
                            let () = e;
                            (16i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Other(e) => {
                        let vec2 = e;
                        let ptr2 = vec2.as_ptr() as i32;
                        let len2 = vec2.len() as i32;
                        (17i32, i64::from(ptr2), len2)
                    }
                };
                let vec4 = exp;
                let ptr4 = vec4.as_ptr() as i32;
                let len4 = vec4.len() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::invalid-type"]
                    fn wit_import(_: i32, _: i64, _: i32, _: i32, _: i32) -> i32;
                }
                let ret = wit_import(result3_0, result3_1, result3_2, ptr4, len4);
                DeError(ret)
            }
        }
    }
    impl DeError {
        pub fn invalid_value(unexp: Unexpected<'_>, exp: &str) -> DeError {
            unsafe {
                let (result3_0, result3_1, result3_2) = match unexp {
                    Unexpected::Bool(e) => (
                        0i32,
                        i64::from(match e {
                            true => 1,
                            false => 0,
                        }),
                        0i32,
                    ),
                    Unexpected::Unsigned(e) => (1i32, wit_bindgen_rust::rt::as_i64(e), 0i32),
                    Unexpected::Signed(e) => (2i32, wit_bindgen_rust::rt::as_i64(e), 0i32),
                    Unexpected::Float(e) => (
                        3i32,
                        (wit_bindgen_rust::rt::as_f64(e)).to_bits() as i64,
                        0i32,
                    ),
                    Unexpected::Char(e) => (4i32, i64::from(wit_bindgen_rust::rt::as_i32(e)), 0i32),
                    Unexpected::Str(e) => {
                        let vec0 = e;
                        let ptr0 = vec0.as_ptr() as i32;
                        let len0 = vec0.len() as i32;
                        (5i32, i64::from(ptr0), len0)
                    }
                    Unexpected::Bytes(e) => {
                        let vec1 = e;
                        let ptr1 = vec1.as_ptr() as i32;
                        let len1 = vec1.len() as i32;
                        (6i32, i64::from(ptr1), len1)
                    }
                    Unexpected::Unit => {
                        let e = ();
                        {
                            let () = e;
                            (7i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Option => {
                        let e = ();
                        {
                            let () = e;
                            (8i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::NewtypeStruct => {
                        let e = ();
                        {
                            let () = e;
                            (9i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Seq => {
                        let e = ();
                        {
                            let () = e;
                            (10i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Map => {
                        let e = ();
                        {
                            let () = e;
                            (11i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Enum => {
                        let e = ();
                        {
                            let () = e;
                            (12i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::UnitVariant => {
                        let e = ();
                        {
                            let () = e;
                            (13i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::NewtypeVariant => {
                        let e = ();
                        {
                            let () = e;
                            (14i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::TupleVariant => {
                        let e = ();
                        {
                            let () = e;
                            (15i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::StructVariant => {
                        let e = ();
                        {
                            let () = e;
                            (16i32, 0i64, 0i32)
                        }
                    }
                    Unexpected::Other(e) => {
                        let vec2 = e;
                        let ptr2 = vec2.as_ptr() as i32;
                        let len2 = vec2.len() as i32;
                        (17i32, i64::from(ptr2), len2)
                    }
                };
                let vec4 = exp;
                let ptr4 = vec4.as_ptr() as i32;
                let len4 = vec4.len() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::invalid-value"]
                    fn wit_import(_: i32, _: i64, _: i32, _: i32, _: i32) -> i32;
                }
                let ret = wit_import(result3_0, result3_1, result3_2, ptr4, len4);
                DeError(ret)
            }
        }
    }
    impl DeError {
        pub fn invalid_length(len: Usize, exp: &str) -> DeError {
            unsafe {
                let Usize { size: size0 } = len;
                let vec1 = exp;
                let ptr1 = vec1.as_ptr() as i32;
                let len1 = vec1.len() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::invalid-length"]
                    fn wit_import(_: i32, _: i32, _: i32) -> i32;
                }
                let ret = wit_import(wit_bindgen_rust::rt::as_i32(size0), ptr1, len1);
                DeError(ret)
            }
        }
    }
    impl DeError {
        pub fn unknown_variant(variant: &str, expected: &[&str]) -> DeError {
            unsafe {
                let vec0 = variant;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                let vec2 = expected;
                let len2 = vec2.len() as i32;
                let layout2 = core::alloc::Layout::from_size_align_unchecked(vec2.len() * 8, 4);
                let result2 = std::alloc::alloc(layout2);
                if result2.is_null() {
                    std::alloc::handle_alloc_error(layout2);
                }
                for (i, e) in vec2.into_iter().enumerate() {
                    let base = result2 as i32 + (i as i32) * 8;
                    {
                        let vec1 = e;
                        let ptr1 = vec1.as_ptr() as i32;
                        let len1 = vec1.len() as i32;
                        *((base + 4) as *mut i32) = len1;
                        *((base + 0) as *mut i32) = ptr1;
                    }
                }
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::unknown-variant"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32) -> i32;
                }
                let ret = wit_import(ptr0, len0, result2 as i32, len2);
                std::alloc::dealloc(result2, layout2);
                DeError(ret)
            }
        }
    }
    impl DeError {
        pub fn unknown_field(field: &str, expected: &[&str]) -> DeError {
            unsafe {
                let vec0 = field;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                let vec2 = expected;
                let len2 = vec2.len() as i32;
                let layout2 = core::alloc::Layout::from_size_align_unchecked(vec2.len() * 8, 4);
                let result2 = std::alloc::alloc(layout2);
                if result2.is_null() {
                    std::alloc::handle_alloc_error(layout2);
                }
                for (i, e) in vec2.into_iter().enumerate() {
                    let base = result2 as i32 + (i as i32) * 8;
                    {
                        let vec1 = e;
                        let ptr1 = vec1.as_ptr() as i32;
                        let len1 = vec1.len() as i32;
                        *((base + 4) as *mut i32) = len1;
                        *((base + 0) as *mut i32) = ptr1;
                    }
                }
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::unknown-field"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32) -> i32;
                }
                let ret = wit_import(ptr0, len0, result2 as i32, len2);
                std::alloc::dealloc(result2, layout2);
                DeError(ret)
            }
        }
    }
    impl DeError {
        pub fn missing_field(field: &str) -> DeError {
            unsafe {
                let vec0 = field;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::missing-field"]
                    fn wit_import(_: i32, _: i32) -> i32;
                }
                let ret = wit_import(ptr0, len0);
                DeError(ret)
            }
        }
    }
    impl DeError {
        pub fn duplicate_field(field: &str) -> DeError {
            unsafe {
                let vec0 = field;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_de-error::duplicate-field"]
                    fn wit_import(_: i32, _: i32) -> i32;
                }
                let ret = wit_import(ptr0, len0);
                DeError(ret)
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_any(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-any"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_bool(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-bool"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_u8(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-u8"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_u16(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-u16"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_u32(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-u32"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_u64(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-u64"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_i8(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-i8"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_i16(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-i16"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_i32(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-i32"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_i64(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-i64"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_i128(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-i128"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_u128(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-u128"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_f32(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-f32"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_f64(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-f64"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_char(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-char"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_str(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-str"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_string(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-string"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_bytes(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-bytes"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_byte_buf(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-byte-buf"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_option(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-option"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_unit(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-unit"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_unit_struct(
            &self,
            name: &str,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let vec0 = name;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                let VisitorHandle { handle: handle1 } = visitor;
                let ptr2 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-unit-struct"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    ptr0,
                    len0,
                    wit_bindgen_rust::rt::as_i32(handle1),
                    ptr2,
                );
                match i32::from(*((ptr2 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr2 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr2 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_newtype_struct(
            &self,
            name: &str,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let vec0 = name;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                let VisitorHandle { handle: handle1 } = visitor;
                let ptr2 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-newtype-struct"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    ptr0,
                    len0,
                    wit_bindgen_rust::rt::as_i32(handle1),
                    ptr2,
                );
                match i32::from(*((ptr2 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr2 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr2 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_seq(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-seq"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_tuple(
            &self,
            len: Usize,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let Usize { size: size0 } = len;
                let VisitorHandle { handle: handle1 } = visitor;
                let ptr2 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-tuple"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    wit_bindgen_rust::rt::as_i32(size0),
                    wit_bindgen_rust::rt::as_i32(handle1),
                    ptr2,
                );
                match i32::from(*((ptr2 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr2 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr2 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_tuple_struct(
            &self,
            name: &str,
            len: Usize,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let vec0 = name;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                let Usize { size: size1 } = len;
                let VisitorHandle { handle: handle2 } = visitor;
                let ptr3 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-tuple-struct"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    ptr0,
                    len0,
                    wit_bindgen_rust::rt::as_i32(size1),
                    wit_bindgen_rust::rt::as_i32(handle2),
                    ptr3,
                );
                match i32::from(*((ptr3 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr3 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr3 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_map(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-map"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_struct(
            &self,
            name: &str,
            fields: &[&str],
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let vec0 = name;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                let vec2 = fields;
                let len2 = vec2.len() as i32;
                let layout2 = core::alloc::Layout::from_size_align_unchecked(vec2.len() * 8, 4);
                let result2 = std::alloc::alloc(layout2);
                if result2.is_null() {
                    std::alloc::handle_alloc_error(layout2);
                }
                for (i, e) in vec2.into_iter().enumerate() {
                    let base = result2 as i32 + (i as i32) * 8;
                    {
                        let vec1 = e;
                        let ptr1 = vec1.as_ptr() as i32;
                        let len1 = vec1.len() as i32;
                        *((base + 4) as *mut i32) = len1;
                        *((base + 0) as *mut i32) = ptr1;
                    }
                }
                let VisitorHandle { handle: handle3 } = visitor;
                let ptr4 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-struct"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    ptr0,
                    len0,
                    result2 as i32,
                    len2,
                    wit_bindgen_rust::rt::as_i32(handle3),
                    ptr4,
                );
                std::alloc::dealloc(result2, layout2);
                match i32::from(*((ptr4 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr4 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr4 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_identifier(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-identifier"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_enum(
            &self,
            name: &str,
            variants: &[&str],
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let vec0 = name;
                let ptr0 = vec0.as_ptr() as i32;
                let len0 = vec0.len() as i32;
                let vec2 = variants;
                let len2 = vec2.len() as i32;
                let layout2 = core::alloc::Layout::from_size_align_unchecked(vec2.len() * 8, 4);
                let result2 = std::alloc::alloc(layout2);
                if result2.is_null() {
                    std::alloc::handle_alloc_error(layout2);
                }
                for (i, e) in vec2.into_iter().enumerate() {
                    let base = result2 as i32 + (i as i32) * 8;
                    {
                        let vec1 = e;
                        let ptr1 = vec1.as_ptr() as i32;
                        let len1 = vec1.len() as i32;
                        *((base + 4) as *mut i32) = len1;
                        *((base + 0) as *mut i32) = ptr1;
                    }
                }
                let VisitorHandle { handle: handle3 } = visitor;
                let ptr4 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-enum"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    ptr0,
                    len0,
                    result2 as i32,
                    len2,
                    wit_bindgen_rust::rt::as_i32(handle3),
                    ptr4,
                );
                std::alloc::dealloc(result2, layout2);
                match i32::from(*((ptr4 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr4 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr4 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_deserialize_ignored_any(
            &self,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let VisitorHandle { handle: handle0 } = visitor;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-deserialize-ignored-any"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl Deserializer {
        pub fn erased_is_human_readable(&self) -> bool {
            unsafe {
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_deserializer::erased-is-human-readable"]
                    fn wit_import(_: i32) -> i32;
                }
                let ret = wit_import(self.0);
                match ret {
                    0 => false,
                    1 => true,
                    _ => ::std::rt::begin_panic("invalid bool discriminant"),
                }
            }
        }
    }
    impl SeqAccess {
        pub fn erased_next_element(
            &self,
            seed: DeserializeSeedHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let DeserializeSeedHandle { handle: handle0 } = seed;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_seq-access::erased-next-element"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl SeqAccess {
        pub fn erased_size_hint(&self) -> Option<Usize> {
            unsafe {
                let ptr0 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_seq-access::erased-size-hint"]
                    fn wit_import(_: i32, _: i32);
                }
                wit_import(self.0, ptr0);
                match i32::from(*((ptr0 + 0) as *const u8)) {
                    0 => None,
                    1 => Some(Usize {
                        size: *((ptr0 + 4) as *const i32) as u32,
                    }),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl MapAccess {
        pub fn erased_next_key(
            &self,
            seed: DeserializeSeedHandle,
        ) -> Result<Option<DeValueHandle>, DeError> {
            unsafe {
                let DeserializeSeedHandle { handle: handle0 } = seed;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_map-access::erased-next-key"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(match i32::from(*((ptr1 + 4) as *const u8)) {
                        0 => None,
                        1 => Some(DeValueHandle {
                            handle: *((ptr1 + 8) as *const i32),
                        }),
                        _ => ::std::rt::begin_panic("invalid enum discriminant"),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl MapAccess {
        pub fn erased_next_value(
            &self,
            seed: DeserializeSeedHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let DeserializeSeedHandle { handle: handle0 } = seed;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_map-access::erased-next-value"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl MapAccess {
        pub fn erased_next_entry(
            &self,
            kseed: DeserializeSeedHandle,
            vseed: DeserializeSeedHandle,
        ) -> Result<Option<(DeValueHandle, DeValueHandle)>, DeError> {
            unsafe {
                let DeserializeSeedHandle { handle: handle0 } = kseed;
                let DeserializeSeedHandle { handle: handle1 } = vseed;
                let ptr2 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_map-access::erased-next-entry"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    wit_bindgen_rust::rt::as_i32(handle0),
                    wit_bindgen_rust::rt::as_i32(handle1),
                    ptr2,
                );
                match i32::from(*((ptr2 + 0) as *const u8)) {
                    0 => Ok(match i32::from(*((ptr2 + 4) as *const u8)) {
                        0 => None,
                        1 => Some((
                            DeValueHandle {
                                handle: *((ptr2 + 8) as *const i32),
                            },
                            DeValueHandle {
                                handle: *((ptr2 + 12) as *const i32),
                            },
                        )),
                        _ => ::std::rt::begin_panic("invalid enum discriminant"),
                    }),
                    1 => Err(DeError(*((ptr2 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl MapAccess {
        pub fn erased_size_hint(&self) -> Option<Usize> {
            unsafe {
                let ptr0 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_map-access::erased-size-hint"]
                    fn wit_import(_: i32, _: i32);
                }
                wit_import(self.0, ptr0);
                match i32::from(*((ptr0 + 0) as *const u8)) {
                    0 => None,
                    1 => Some(Usize {
                        size: *((ptr0 + 4) as *const i32) as u32,
                    }),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl EnumAccess {
        pub fn erased_variant(
            &self,
            seed: DeserializeSeedHandle,
        ) -> Result<(DeValueHandle, VariantAccess), DeError> {
            unsafe {
                let DeserializeSeedHandle { handle: handle0 } = seed;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_enum-access::erased-variant"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok((
                        DeValueHandle {
                            handle: *((ptr1 + 4) as *const i32),
                        },
                        VariantAccess(*((ptr1 + 8) as *const i32)),
                    )),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl VariantAccess {
        pub fn unit_variant(&self) -> Result<(), DeError> {
            unsafe {
                let ptr0 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_variant-access::unit-variant"]
                    fn wit_import(_: i32, _: i32);
                }
                wit_import(self.0, ptr0);
                match i32::from(*((ptr0 + 0) as *const u8)) {
                    0 => Ok(()),
                    1 => Err(DeError(*((ptr0 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl VariantAccess {
        pub fn newtype_variant_seed(
            &self,
            seed: DeserializeSeedHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let DeserializeSeedHandle { handle: handle0 } = seed;
                let ptr1 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_variant-access::newtype-variant-seed"]
                    fn wit_import(_: i32, _: i32, _: i32);
                }
                wit_import(self.0, wit_bindgen_rust::rt::as_i32(handle0), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr1 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr1 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl VariantAccess {
        pub fn newtype_variant(&self) -> Result<DeValueHandle, DeError> {
            unsafe {
                let ptr0 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_variant-access::newtype-variant"]
                    fn wit_import(_: i32, _: i32);
                }
                wit_import(self.0, ptr0);
                match i32::from(*((ptr0 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr0 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr0 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl VariantAccess {
        pub fn tuple_variant(
            &self,
            len: Usize,
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let Usize { size: size0 } = len;
                let VisitorHandle { handle: handle1 } = visitor;
                let ptr2 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_variant-access::tuple-variant"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    wit_bindgen_rust::rt::as_i32(size0),
                    wit_bindgen_rust::rt::as_i32(handle1),
                    ptr2,
                );
                match i32::from(*((ptr2 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr2 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr2 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    impl VariantAccess {
        pub fn struct_variant(
            &self,
            fields: &[&str],
            visitor: VisitorHandle,
        ) -> Result<DeValueHandle, DeError> {
            unsafe {
                let vec1 = fields;
                let len1 = vec1.len() as i32;
                let layout1 = core::alloc::Layout::from_size_align_unchecked(vec1.len() * 8, 4);
                let result1 = std::alloc::alloc(layout1);
                if result1.is_null() {
                    std::alloc::handle_alloc_error(layout1);
                }
                for (i, e) in vec1.into_iter().enumerate() {
                    let base = result1 as i32 + (i as i32) * 8;
                    {
                        let vec0 = e;
                        let ptr0 = vec0.as_ptr() as i32;
                        let len0 = vec0.len() as i32;
                        *((base + 4) as *mut i32) = len0;
                        *((base + 0) as *mut i32) = ptr0;
                    }
                }
                let VisitorHandle { handle: handle2 } = visitor;
                let ptr3 = SERDE_WASM_HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "serde-wasm-host")]
                extern "C" {
                    #[link_name = "serde-wasm-host_variant-access::struct-variant"]
                    fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32);
                }
                wit_import(
                    self.0,
                    result1 as i32,
                    len1,
                    wit_bindgen_rust::rt::as_i32(handle2),
                    ptr3,
                );
                std::alloc::dealloc(result1, layout1);
                match i32::from(*((ptr3 + 0) as *const u8)) {
                    0 => Ok(DeValueHandle {
                        handle: *((ptr3 + 4) as *const i32),
                    }),
                    1 => Err(DeError(*((ptr3 + 4) as *const i32))),
                    _ => ::std::rt::begin_panic("invalid enum discriminant"),
                }
            }
        }
    }
    #[repr(align(4))]
    struct RetArea([u8; 16]);
    static mut SERDE_WASM_HOST_RET_AREA: RetArea = RetArea([0; 16]);
}
const _ : & str = "record usize {\n  size: u32,\n}\n\nvariant unexpected {\n  %bool(bool),\n  unsigned(u64),\n  signed(s64),\n  float(float64),\n  %char(char),\n  str(string),\n  bytes(list<u8>),\n  %unit,\n  %option,\n  newtype-struct,\n  seq,\n  map,\n  %enum,\n  unit-variant,\n  newtype-variant,\n  tuple-variant,\n  struct-variant,\n  other(string),\n}\n\nresource de-error {\n  static custom: func(msg: string) -> de-error\n  static invalid-type: func(unexp: unexpected, exp: string) -> de-error\n  static invalid-value: func(unexp: unexpected, exp: string) -> de-error\n  static invalid-length: func(len: usize, exp: string) -> de-error\n  static unknown-variant: func(%variant: string, %expected: list<string>) -> de-error\n  static unknown-field: func(field: string, %expected: list<string>) -> de-error\n  static missing-field: func(field: string) -> de-error\n  static duplicate-field: func(field: string) -> de-error\n}\n\nresource deserializer {\n  erased-deserialize-any: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-bool: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u8: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u16: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u32: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u64: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i8: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i16: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i32: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i64: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-i128: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-u128: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-f32: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-f64: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-char: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-str: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-string: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-bytes: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-byte-buf: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-option: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-unit: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-unit-struct: func(name: string, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-newtype-struct: func(name: string, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-seq: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-tuple: func(len: usize, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-tuple-struct: func(name: string, len: usize, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-map: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-struct: func(name: string, fields: list<string>, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-identifier: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-enum: func(name: string, variants: list<string>, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-deserialize-ignored-any: func(visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  erased-is-human-readable: func() -> bool\n}\n\nresource seq-access {\n  erased-next-element: func(seed: deserialize-seed-handle) -> expected<de-value-handle, de-error>\n  erased-size-hint: func() -> option<usize>\n}\n\nresource map-access {\n  erased-next-key: func(seed: deserialize-seed-handle) -> expected<option<de-value-handle>, de-error>\n  erased-next-value: func(seed: deserialize-seed-handle) -> expected<de-value-handle, de-error>\n  erased-next-entry: func(kseed: deserialize-seed-handle, vseed: deserialize-seed-handle) -> expected<option<tuple<de-value-handle, de-value-handle>>, de-error>\n  erased-size-hint: func() -> option<usize>\n}\n\nresource enum-access {\n  erased-variant: func(seed: deserialize-seed-handle) -> expected<tuple<de-value-handle, variant-access>, de-error>\n}\n\nresource variant-access {\n  unit-variant: func() -> expected<unit, de-error>\n  newtype-variant-seed: func(seed: deserialize-seed-handle) -> expected<de-value-handle, de-error>\n  newtype-variant: func() -> expected<de-value-handle, de-error>\n  tuple-variant: func(len: usize, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n  struct-variant: func(fields: list<string>, visitor: visitor-handle) -> expected<de-value-handle, de-error>\n}\n\nrecord visitor-handle {\n  %handle: s32\n}\n\nrecord de-value-handle {\n  %handle: s32\n}\n\nrecord deserialize-seed-handle {\n  %handle: s32\n}\n" ;
#[allow(clippy::all)]
mod serde_wasm_guest {
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
    unsafe impl wit_bindgen_rust::HandleType for super::DeValue {
        #[inline]
        fn clone(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
        #[inline]
        fn drop(_val: i32) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
    }
    unsafe impl wit_bindgen_rust::LocalHandle for super::DeValue {
        #[inline]
        fn new(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
        #[inline]
        fn get(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
    }
    const _: () = {
        #[export_name = "canonical_abi_drop_de-value"]
        extern "C" fn drop(ty: Box<super::DeValue>) {
            <super::SerdeWasmGuest as SerdeWasmGuest>::drop_de_value(*ty)
        }
    };
    unsafe impl wit_bindgen_rust::HandleType for super::DeserializeSeed {
        #[inline]
        fn clone(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
        #[inline]
        fn drop(_val: i32) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
    }
    unsafe impl wit_bindgen_rust::LocalHandle for super::DeserializeSeed {
        #[inline]
        fn new(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
        #[inline]
        fn get(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
    }
    const _: () = {
        #[export_name = "canonical_abi_drop_deserialize-seed"]
        extern "C" fn drop(ty: Box<super::DeserializeSeed>) {
            <super::SerdeWasmGuest as SerdeWasmGuest>::drop_deserialize_seed(*ty)
        }
    };
    unsafe impl wit_bindgen_rust::HandleType for super::Visitor {
        #[inline]
        fn clone(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
        #[inline]
        fn drop(_val: i32) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
    }
    unsafe impl wit_bindgen_rust::LocalHandle for super::Visitor {
        #[inline]
        fn new(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
        #[inline]
        fn get(_val: i32) -> i32 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                {
                    ::std::rt::begin_panic("handles can only be used on wasm32")
                };
            }
        }
    }
    const _: () = {
        #[export_name = "canonical_abi_drop_visitor"]
        extern "C" fn drop(ty: Box<super::Visitor>) {
            <super::SerdeWasmGuest as SerdeWasmGuest>::drop_visitor(*ty)
        }
    };
    #[export_name = "deserialize-seed::erased-deserialize"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_deserialize_seed_erased_deserialize(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::DeserializeSeed as DeserializeSeed>::erased_deserialize(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            DeserializerHandle { handle: arg1 },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-expecting"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_expecting(arg0: i32) -> i32 {
        let result = <super::Visitor as Visitor>::erased_expecting(
            &wit_bindgen_rust::Handle::from_raw(arg0),
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Some(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let vec1 = (e.into_bytes()).into_boxed_slice();
                let ptr1 = vec1.as_ptr() as i32;
                let len1 = vec1.len() as i32;
                core::mem::forget(vec1);
                *((ptr0 + 8) as *mut i32) = len1;
                *((ptr0 + 4) as *mut i32) = ptr1;
            }
            None => {
                let e = ();
                {
                    *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                    let () = e;
                }
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-bool"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_bool(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_bool(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            match arg1 {
                0 => false,
                1 => true,
                _ => ::std::rt::begin_panic("invalid bool discriminant"),
            },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-i8"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_i8(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_i8(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1 as i8,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-i16"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_i16(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_i16(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1 as i16,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-i32"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_i32(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_i32(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-i64"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_i64(
        arg0: i32,
        arg1: i64,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_i64(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-i128"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_i128(
        arg0: i32,
        arg1: i64,
        arg2: i64,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_i128(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            I128 {
                hi: arg1 as u64,
                lo: arg2 as u64,
            },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-u8"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_u8(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_u8(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1 as u8,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-u16"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_u16(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_u16(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1 as u16,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-u32"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_u32(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_u32(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1 as u32,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-u64"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_u64(
        arg0: i32,
        arg1: i64,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_u64(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1 as u64,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-u128"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_u128(
        arg0: i32,
        arg1: i64,
        arg2: i64,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_u128(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            U128 {
                hi: arg1 as u64,
                lo: arg2 as u64,
            },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-f32"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_f32(
        arg0: i32,
        arg1: f32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_f32(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-f64"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_f64(
        arg0: i32,
        arg1: f64,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_f64(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            arg1,
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-char"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_char(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_char(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            core::char::from_u32(arg1 as u32).unwrap(),
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-str"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_str(
        arg0: i32,
        arg1: i32,
        arg2: i32,
    ) -> i32 {
        let len0 = arg2 as usize;
        let result = <super::Visitor as Visitor>::erased_visit_str(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            String::from_utf8(Vec::from_raw_parts(arg1 as *mut _, len0, len0)).unwrap(),
        );
        let ptr1 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr1 + 0) as *mut u8) = (0i32) as u8;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr1 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle2 } = e;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle2);
            }
        };
        ptr1
    }
    #[export_name = "visitor::erased-visit-string"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_string(
        arg0: i32,
        arg1: i32,
        arg2: i32,
    ) -> i32 {
        let len0 = arg2 as usize;
        let result = <super::Visitor as Visitor>::erased_visit_string(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            String::from_utf8(Vec::from_raw_parts(arg1 as *mut _, len0, len0)).unwrap(),
        );
        let ptr1 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr1 + 0) as *mut u8) = (0i32) as u8;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr1 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle2 } = e;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle2);
            }
        };
        ptr1
    }
    #[export_name = "visitor::erased-visit-bytes"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_bytes(
        arg0: i32,
        arg1: i32,
        arg2: i32,
    ) -> i32 {
        let len0 = arg2 as usize;
        let result = <super::Visitor as Visitor>::erased_visit_bytes(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            Vec::from_raw_parts(arg1 as *mut _, len0, len0),
        );
        let ptr1 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr1 + 0) as *mut u8) = (0i32) as u8;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr1 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle2 } = e;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle2);
            }
        };
        ptr1
    }
    #[export_name = "visitor::erased-visit-byte-buf"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_byte_buf(
        arg0: i32,
        arg1: i32,
        arg2: i32,
    ) -> i32 {
        let len0 = arg2 as usize;
        let result = <super::Visitor as Visitor>::erased_visit_byte_buf(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            Vec::from_raw_parts(arg1 as *mut _, len0, len0),
        );
        let ptr1 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr1 + 0) as *mut u8) = (0i32) as u8;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr1 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle2 } = e;
                *((ptr1 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle2);
            }
        };
        ptr1
    }
    #[export_name = "visitor::erased-visit-none"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_none(
        arg0: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_none(
            &wit_bindgen_rust::Handle::from_raw(arg0),
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-some"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_some(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_some(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            DeserializerHandle { handle: arg1 },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-unit"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_unit(
        arg0: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_unit(
            &wit_bindgen_rust::Handle::from_raw(arg0),
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-newtype-struct"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_newtype_struct(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_newtype_struct(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            DeserializerHandle { handle: arg1 },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-seq"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_seq(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_seq(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            SeqAccessHandle { handle: arg1 },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-map"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_map(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_map(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            MapAccessHandle { handle: arg1 },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[export_name = "visitor::erased-visit-enum"]
    unsafe extern "C" fn __wit_bindgen_serde_wasm_guest_visitor_erased_visit_enum(
        arg0: i32,
        arg1: i32,
    ) -> i32 {
        let result = <super::Visitor as Visitor>::erased_visit_enum(
            &wit_bindgen_rust::Handle::from_raw(arg0),
            EnumAccessHandle { handle: arg1 },
        );
        let ptr0 = SERDE_WASM_GUEST_RET_AREA.0.as_mut_ptr() as i32;
        match result {
            Ok(e) => {
                *((ptr0 + 0) as *mut u8) = (0i32) as u8;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::Handle::into_raw(e);
            }
            Err(e) => {
                *((ptr0 + 0) as *mut u8) = (1i32) as u8;
                let DeErrorHandle { handle: handle1 } = e;
                *((ptr0 + 4) as *mut i32) = wit_bindgen_rust::rt::as_i32(handle1);
            }
        };
        ptr0
    }
    #[repr(align(4))]
    struct RetArea([u8; 12]);
    static mut SERDE_WASM_GUEST_RET_AREA: RetArea = RetArea([0; 12]);
    pub trait SerdeWasmGuest {
        /// An optional callback invoked when a handle is finalized
        /// and destroyed.
        fn drop_de_value(val: super::DeValue) {
            drop(val);
        }
        /// An optional callback invoked when a handle is finalized
        /// and destroyed.
        fn drop_deserialize_seed(val: super::DeserializeSeed) {
            drop(val);
        }
        /// An optional callback invoked when a handle is finalized
        /// and destroyed.
        fn drop_visitor(val: super::Visitor) {
            drop(val);
        }
    }
    pub trait DeserializeSeed {
        fn erased_deserialize(
            &self,
            deserializer: DeserializerHandle,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
    }
    pub trait Visitor {
        fn erased_expecting(&self) -> Option<String>;
        fn erased_visit_bool(
            &self,
            v: bool,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_i8(
            &self,
            v: i8,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_i16(
            &self,
            v: i16,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_i32(
            &self,
            v: i32,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_i64(
            &self,
            v: i64,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_i128(
            &self,
            v: I128,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_u8(
            &self,
            v: u8,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_u16(
            &self,
            v: u16,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_u32(
            &self,
            v: u32,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_u64(
            &self,
            v: u64,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_u128(
            &self,
            v: U128,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_f32(
            &self,
            v: f32,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_f64(
            &self,
            v: f64,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_char(
            &self,
            v: char,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_str(
            &self,
            v: String,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_string(
            &self,
            v: String,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_bytes(
            &self,
            v: Vec<u8>,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_byte_buf(
            &self,
            v: Vec<u8>,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_none(
            &self,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_some(
            &self,
            deserializer: DeserializerHandle,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_unit(
            &self,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_newtype_struct(
            &self,
            deserializer: DeserializerHandle,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_seq(
            &self,
            seq: SeqAccessHandle,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_map(
            &self,
            map: MapAccessHandle,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
        fn erased_visit_enum(
            &self,
            data: EnumAccessHandle,
        ) -> Result<wit_bindgen_rust::Handle<super::DeValue>, DeErrorHandle>;
    }
}
const _ : & str = "record i128 {\n  hi: u64,\n  lo: u64,\n}\n\nrecord u128 {\n  hi: u64,\n  lo: u64,\n}\n\nresource de-value {}\n\nresource deserialize-seed {\n  erased-deserialize: func(deserializer: deserializer-handle) -> expected<de-value, de-error-handle>\n}\n\nresource visitor {\n  erased-expecting: func() -> option<string>\n  \n  erased-visit-bool: func(v: bool) -> expected<de-value, de-error-handle>\n  erased-visit-i8: func(v: s8) -> expected<de-value, de-error-handle>\n  erased-visit-i16: func(v: s16) -> expected<de-value, de-error-handle>\n  erased-visit-i32: func(v: s32) -> expected<de-value, de-error-handle>\n  erased-visit-i64: func(v: s64) -> expected<de-value, de-error-handle>\n  erased-visit-i128: func(v: i128) -> expected<de-value, de-error-handle>\n  erased-visit-u8: func(v: u8) -> expected<de-value, de-error-handle>\n  erased-visit-u16: func(v: u16) -> expected<de-value, de-error-handle>\n  erased-visit-u32: func(v: u32) -> expected<de-value, de-error-handle>\n  erased-visit-u64: func(v: u64) -> expected<de-value, de-error-handle>\n  erased-visit-u128: func(v: u128) -> expected<de-value, de-error-handle>\n  erased-visit-f32: func(v: float32) -> expected<de-value, de-error-handle>\n  erased-visit-f64: func(v: float64) -> expected<de-value, de-error-handle>\n  erased-visit-char: func(v: char) -> expected<de-value, de-error-handle>\n  erased-visit-str: func(v: string) -> expected<de-value, de-error-handle>\n  erased-visit-string: func(v: string) -> expected<de-value, de-error-handle>\n  erased-visit-bytes: func(v: list<u8>) -> expected<de-value, de-error-handle>\n  erased-visit-byte-buf: func(v: list<u8>) -> expected<de-value, de-error-handle>\n  erased-visit-none: func() -> expected<de-value, de-error-handle>\n  erased-visit-some: func(deserializer: deserializer-handle) -> expected<de-value, de-error-handle>\n  erased-visit-unit: func() -> expected<de-value, de-error-handle>\n  erased-visit-newtype-struct: func(deserializer: deserializer-handle) -> expected<de-value, de-error-handle>\n  erased-visit-seq: func(seq: seq-access-handle) -> expected<de-value, de-error-handle>\n  erased-visit-map: func(map: map-access-handle) -> expected<de-value, de-error-handle>\n  erased-visit-enum: func(data: enum-access-handle) -> expected<de-value, de-error-handle>\n}\n\nrecord de-error-handle {\n  %handle: s32\n}\n\nrecord deserializer-handle {\n  %handle: s32\n}\n\nrecord seq-access-handle {\n  %handle: s32\n}\n\nrecord map-access-handle {\n  %handle: s32\n}\n\nrecord enum-access-handle {\n  %handle: s32\n}\n" ;
struct DeValue {
    inner: RefCell<Option<de::Out>>,
}
struct DeserializeSeed {
    inner: Box<RefCell<dyn de::DeserializeSeed<'static>>>,
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
struct Visitor {
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
        }
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
impl serde_wasm_guest::Visitor for Visitor {
    fn erased_expecting(&self) -> Option<String> {
        struct Expecting<'a>(&'a Visitor);
        impl<'a> fmt::Display for Expecting<'a> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                self.0.inner.borrow().erased_expecting(fmt)
            }
        }
        let mut buffer = String::new();
        match buffer.write_fmt(::core::fmt::Arguments::new_v1(
            &[""],
            &[::core::fmt::ArgumentV1::new_display(&Expecting(self))],
        )) {
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
        ::core::panicking::panic("not yet implemented")
    }
    fn erased_visit_map(
        &self,
        map: serde_wasm_guest::MapAccessHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        ::core::panicking::panic("not yet implemented")
    }
    fn erased_visit_enum(
        &self,
        data: serde_wasm_guest::EnumAccessHandle,
    ) -> Result<wit_bindgen_rust::Handle<DeValue>, serde_wasm_guest::DeErrorHandle> {
        ::core::panicking::panic("not yet implemented")
    }
}
struct SerdeWasmGuest {}
impl serde_wasm_guest::SerdeWasmGuest for SerdeWasmGuest {}
#[cfg(not(target_pointer_width = "32"))]
impl From<usize> for serde_wasm_host::Usize {
    fn from(size: usize) -> Self {
        extern "C" {
            fn usize_must_be_u32(size: usize) -> !;
        }
        unsafe { usize_must_be_u32(size) }
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
