use serde::{
    ser::{
        Error, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize, Serializer,
};

pub struct DelayedSerializer;

pub struct StaticString(Box<str>);

impl StaticString {
    pub fn get(&self) -> &'static str {
        unsafe { &*(&*self.0 as *const str) }
    }
}

impl From<&'static str> for StaticString {
    fn from(string: &'static str) -> Self {
        Self(String::from(string).into_boxed_str())
    }
}

pub enum DelayedSerialization {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Char(char),
    Str(String),
    Bytes(Box<[u8]>),
    None,
    Some(Box<DelayedSerialization>),
    Unit,
    UnitStruct {
        name: StaticString,
    },
    UnitVariant {
        name: StaticString,
        variant_index: u32,
        variant: StaticString,
    },
    NewtypeStruct {
        name: StaticString,
        value: Box<DelayedSerialization>,
    },
    NewtypeVariant {
        name: StaticString,
        variant_index: u32,
        variant: StaticString,
        value: Box<DelayedSerialization>,
    },
    Seq {
        len: Option<usize>,
        elements: Box<[DelayedSerialization]>,
    },
    Tuple {
        len: usize,
        fields: Box<[DelayedSerialization]>,
    },
    TupleStruct {
        name: StaticString,
        len: usize,
        fields: Box<[DelayedSerialization]>,
    },
    TupleVariant {
        name: StaticString,
        variant_index: u32,
        variant: StaticString,
        len: usize,
        fields: Box<[DelayedSerialization]>,
    },
    Map {
        len: Option<usize>,
        entries: Box<[(Option<DelayedSerialization>, Option<DelayedSerialization>)]>,
    },
    Struct {
        name: StaticString,
        len: usize,
        fields: Box<[(StaticString, Option<DelayedSerialization>)]>,
    },
    StructVariant {
        name: StaticString,
        variant_index: u32,
        variant: StaticString,
        len: usize,
        fields: Box<[(StaticString, Option<DelayedSerialization>)]>,
    },
    /* DelayedElement {
     * key: String,
     * }, */
}

#[derive(Debug)]
pub struct CustomError(String);

impl std::fmt::Display for CustomError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&self.0)
    }
}

impl std::error::Error for CustomError {}

impl Error for CustomError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        CustomError(msg.to_string())
    }
}

pub struct DelayedSequenceSerializer {
    len: Option<usize>,
    elements: Vec<DelayedSerialization>,
}

impl SerializeSeq for DelayedSequenceSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.elements.push(value.serialize(DelayedSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Seq {
            len: self.len,
            elements: self.elements.into_boxed_slice(),
        })
    }
}

pub struct DelayedTupleSerializer {
    len: usize,
    fields: Vec<DelayedSerialization>,
}

impl SerializeTuple for DelayedTupleSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.fields.push(value.serialize(DelayedSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Tuple {
            len: self.len,
            fields: self.fields.into_boxed_slice(),
        })
    }
}

pub struct DelayedTupleStructSerializer {
    name: StaticString,
    len: usize,
    fields: Vec<DelayedSerialization>,
}

impl SerializeTupleStruct for DelayedTupleStructSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.fields.push(value.serialize(DelayedSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::TupleStruct {
            name: self.name,
            len: self.len,
            fields: self.fields.into_boxed_slice(),
        })
    }
}

pub struct DelayedTupleVariantSerializer {
    name: StaticString,
    variant_index: u32,
    variant: StaticString,
    len: usize,
    fields: Vec<DelayedSerialization>,
}

impl SerializeTupleVariant for DelayedTupleVariantSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.fields.push(value.serialize(DelayedSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::TupleVariant {
            name: self.name,
            variant_index: self.variant_index,
            variant: self.variant,
            len: self.len,
            fields: self.fields.into_boxed_slice(),
        })
    }
}

pub struct DelayedMapSerializer {
    len: Option<usize>,
    entries: Vec<(Option<DelayedSerialization>, Option<DelayedSerialization>)>,
}

impl SerializeMap for DelayedMapSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.entries
            .push((Some(key.serialize(DelayedSerializer)?), None));
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.entries
            .push((None, Some(value.serialize(DelayedSerializer)?)));
        Ok(())
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        self.entries.push((
            Some(key.serialize(DelayedSerializer)?),
            Some(value.serialize(DelayedSerializer)?),
        ));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Map {
            len: self.len,
            entries: self.entries.into_boxed_slice(),
        })
    }
}

pub struct DelayedStructSerializer {
    name: StaticString,
    len: usize,
    fields: Vec<(StaticString, Option<DelayedSerialization>)>,
}

impl SerializeStruct for DelayedStructSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.fields
            .push((key.into(), Some(value.serialize(DelayedSerializer)?)));
        Ok(())
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.fields.push((key.into(), None));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Struct {
            name: self.name,
            len: self.len,
            fields: self.fields.into_boxed_slice(),
        })
    }
}

pub struct DelayedStructVariantSerializer {
    name: StaticString,
    variant_index: u32,
    variant: StaticString,
    len: usize,
    fields: Vec<(StaticString, Option<DelayedSerialization>)>,
}

impl SerializeStructVariant for DelayedStructVariantSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.fields
            .push((key.into(), Some(value.serialize(DelayedSerializer)?)));
        Ok(())
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.fields.push((key.into(), None));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::StructVariant {
            name: self.name,
            variant_index: self.variant_index,
            variant: self.variant,
            len: self.len,
            fields: self.fields.into_boxed_slice(),
        })
    }
}

impl Serializer for DelayedSerializer {
    type Error = CustomError;
    type Ok = DelayedSerialization;
    type SerializeMap = DelayedMapSerializer;
    type SerializeSeq = DelayedSequenceSerializer;
    type SerializeStruct = DelayedStructSerializer;
    type SerializeStructVariant = DelayedStructVariantSerializer;
    type SerializeTuple = DelayedTupleSerializer;
    type SerializeTupleStruct = DelayedTupleStructSerializer;
    type SerializeTupleVariant = DelayedTupleVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::I8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::I64(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::I128(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::U16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::U64(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::U128(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Char(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Str(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Bytes(v.to_owned().into_boxed_slice()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::None)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Some(Box::new(value.serialize(self)?)))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::UnitStruct { name: name.into() })
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::UnitVariant {
            name: name.into(),
            variant_index,
            variant: variant.into(),
        })
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::NewtypeStruct {
            name: name.into(),
            value: Box::new(value.serialize(self)?),
        })
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(DelayedSerialization::NewtypeVariant {
            name: name.into(),
            variant_index,
            variant: variant.into(),
            value: Box::new(value.serialize(self)?),
        })
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(DelayedSequenceSerializer {
            len,
            elements: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(DelayedTupleSerializer {
            len,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(DelayedTupleStructSerializer {
            name: name.into(),
            len,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(DelayedTupleVariantSerializer {
            name: name.into(),
            variant_index,
            variant: variant.into(),
            len,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(DelayedMapSerializer {
            len,
            entries: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(DelayedStructSerializer {
            name: name.into(),
            len,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(DelayedStructVariantSerializer {
            name: name.into(),
            variant_index,
            variant: variant.into(),
            len,
            fields: Vec::with_capacity(len),
        })
    }
}

impl Serialize for DelayedSerialization {
    #[allow(clippy::too_many_lines)]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::I8(v) => serializer.serialize_i8(*v),
            Self::I16(v) => serializer.serialize_i16(*v),
            Self::I32(v) => serializer.serialize_i32(*v),
            Self::I64(v) => serializer.serialize_i64(*v),
            Self::I128(v) => serializer.serialize_i128(*v),
            Self::U8(v) => serializer.serialize_u8(*v),
            Self::U16(v) => serializer.serialize_u16(*v),
            Self::U32(v) => serializer.serialize_u32(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::U128(v) => serializer.serialize_u128(*v),
            Self::F32(v) => serializer.serialize_f32(*v),
            Self::F64(v) => serializer.serialize_f64(*v),
            Self::Char(v) => serializer.serialize_char(*v),
            Self::Str(v) => serializer.serialize_str(v),
            Self::Bytes(v) => serializer.serialize_bytes(v),
            Self::None => serializer.serialize_none(),
            Self::Some(value) => serializer.serialize_some(value),
            Self::Unit => serializer.serialize_unit(),
            Self::UnitStruct { name } => serializer.serialize_unit_struct(name.get()),
            Self::UnitVariant {
                name,
                variant_index,
                variant,
            } => serializer.serialize_unit_variant(name.get(), *variant_index, variant.get()),
            Self::NewtypeStruct { name, value } => {
                serializer.serialize_newtype_struct(name.get(), value)
            },
            Self::NewtypeVariant {
                name,
                variant_index,
                variant,
                value,
            } => serializer.serialize_newtype_variant(
                name.get(),
                *variant_index,
                variant.get(),
                value,
            ),
            Self::Seq { len, elements } => {
                let mut seq = serializer.serialize_seq(*len)?;

                for element in elements.iter() {
                    seq.serialize_element(element)?;
                }

                seq.end()
            },
            Self::Tuple { len, fields } => {
                let mut tuple = serializer.serialize_tuple(*len)?;

                for field in fields.iter() {
                    tuple.serialize_element(field)?;
                }

                tuple.end()
            },
            Self::TupleStruct { name, len, fields } => {
                let mut tuple_struct = serializer.serialize_tuple_struct(name.get(), *len)?;

                for field in fields.iter() {
                    tuple_struct.serialize_field(field)?;
                }

                tuple_struct.end()
            },
            Self::TupleVariant {
                name,
                variant_index,
                variant,
                len,
                fields,
            } => {
                let mut tuple_variant = serializer.serialize_tuple_variant(
                    name.get(),
                    *variant_index,
                    variant.get(),
                    *len,
                )?;

                for field in fields.iter() {
                    tuple_variant.serialize_field(field)?;
                }

                tuple_variant.end()
            },
            Self::Map { len, entries } => {
                let mut map = serializer.serialize_map(*len)?;

                for (key, value) in entries.iter() {
                    match (key, value) {
                        (None, None) => (),
                        (Some(key), None) => map.serialize_key(key)?,
                        (None, Some(value)) => map.serialize_value(value)?,
                        (Some(key), Some(value)) => map.serialize_entry(key, value)?,
                    };
                }

                map.end()
            },
            Self::Struct { name, len, fields } => {
                let mut r#struct = serializer.serialize_struct(name.get(), *len)?;

                for (key, value) in fields.iter() {
                    if let Some(value) = value {
                        r#struct.serialize_field(key.get(), value)?;
                    } else {
                        r#struct.skip_field(key.get())?;
                    }
                }

                r#struct.end()
            },
            Self::StructVariant {
                name,
                variant_index,
                variant,
                len,
                fields,
            } => {
                let mut struct_variant = serializer.serialize_struct_variant(
                    name.get(),
                    *variant_index,
                    variant.get(),
                    *len,
                )?;

                for (key, value) in fields.iter() {
                    if let Some(value) = value {
                        struct_variant.serialize_field(key.get(), value)?;
                    } else {
                        struct_variant.skip_field(key.get())?;
                    }
                }

                struct_variant.end()
            },
            // Self::DelayedElement { key: _key } => unimplemented!("TODO"),
        }
    }
}
