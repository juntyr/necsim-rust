wit_bindgen_wasmtime::export!("src/serde-wasm-host.wit");
wit_bindgen_wasmtime::import!("src/serde-wasm-guest.wit");

struct DeValue {
    inner: serde_wasm_guest::DeValue,
}

struct DeserializeSeed {
    inner: serde_wasm_guest::DeserializeSeed,
}

struct Visitor {
    inner: serde_wasm_guest::Visitor,
}

#[derive(Debug)]
struct DeError {
    // TODO
}

#[derive(Debug)]
struct SeqAccess {
    // TODO
}

#[derive(Debug)]
struct MapAccess {
    // TODO
}

#[derive(Debug)]
struct EnumAccess {
    // TODO
}

#[derive(Debug)]
struct VariantAccess {
    // TODO
}

#[derive(Debug)]
struct Deserializer {
    // TODO
}

struct SerdeWasmHost {}

impl serde_wasm_host::SerdeWasmHost for SerdeWasmHost {
    type DeError = DeError;
    type Deserializer = Deserializer;
    type EnumAccess = EnumAccess;
    type MapAccess = MapAccess;
    type SeqAccess = SeqAccess;
    type VariantAccess = VariantAccess;

    fn de_error_custom(&mut self, msg: &str) -> Self::DeError {
        todo!()
    }

    fn de_error_invalid_type(
        &mut self,
        unexp: serde_wasm_host::Unexpected<'_>,
        exp: &str,
    ) -> Self::DeError {
        todo!()
    }

    fn de_error_invalid_value(
        &mut self,
        unexp: serde_wasm_host::Unexpected<'_>,
        exp: &str,
    ) -> Self::DeError {
        todo!()
    }

    fn de_error_invalid_length(&mut self, len: serde_wasm_host::Usize, exp: &str) -> Self::DeError {
        todo!()
    }

    fn de_error_unknown_variant(&mut self, variant: &str, expected: Vec<&str>) -> Self::DeError {
        todo!()
    }

    fn de_error_unknown_field(&mut self, field: &str, expected: Vec<&str>) -> Self::DeError {
        todo!()
    }

    fn de_error_missing_field(&mut self, field: &str) -> Self::DeError {
        todo!()
    }

    fn de_error_duplicate_field(&mut self, field: &str) -> Self::DeError {
        todo!()
    }

    fn deserializer_erased_deserialize_any(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_bool(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_u8(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_u16(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_u32(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_u64(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_i8(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_i16(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_i32(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_i64(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_i128(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_u128(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_f32(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_f64(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_char(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_str(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_string(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_bytes(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_byte_buf(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_option(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_unit(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_unit_struct(
        &mut self,
        self_: &Self::Deserializer,
        name: &str,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_newtype_struct(
        &mut self,
        self_: &Self::Deserializer,
        name: &str,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_seq(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_tuple(
        &mut self,
        self_: &Self::Deserializer,
        len: serde_wasm_host::Usize,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_tuple_struct(
        &mut self,
        self_: &Self::Deserializer,
        name: &str,
        len: serde_wasm_host::Usize,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_map(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_struct(
        &mut self,
        self_: &Self::Deserializer,
        name: &str,
        fields: Vec<&str>,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_identifier(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_enum(
        &mut self,
        self_: &Self::Deserializer,
        name: &str,
        variants: Vec<&str>,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_deserialize_ignored_any(
        &mut self,
        self_: &Self::Deserializer,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn deserializer_erased_is_human_readable(&mut self, self_: &Self::Deserializer) -> bool {
        todo!()
    }

    fn seq_access_erased_next_element(
        &mut self,
        self_: &Self::SeqAccess,
        seed: serde_wasm_host::DeserializeSeedHandle,
    ) -> Result<Option<serde_wasm_host::DeValueHandle>, Self::DeError> {
        todo!()
    }

    fn seq_access_erased_size_hint(
        &mut self,
        self_: &Self::SeqAccess,
    ) -> Option<serde_wasm_host::Usize> {
        todo!()
    }

    fn map_access_erased_next_key(
        &mut self,
        self_: &Self::MapAccess,
        seed: serde_wasm_host::DeserializeSeedHandle,
    ) -> Result<Option<serde_wasm_host::DeValueHandle>, Self::DeError> {
        todo!()
    }

    fn map_access_erased_next_value(
        &mut self,
        self_: &Self::MapAccess,
        seed: serde_wasm_host::DeserializeSeedHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn map_access_erased_next_entry(
        &mut self,
        self_: &Self::MapAccess,
        kseed: serde_wasm_host::DeserializeSeedHandle,
        vseed: serde_wasm_host::DeserializeSeedHandle,
    ) -> Result<
        Option<(
            serde_wasm_host::DeValueHandle,
            serde_wasm_host::DeValueHandle,
        )>,
        Self::DeError,
    > {
        todo!()
    }

    fn map_access_erased_size_hint(
        &mut self,
        self_: &Self::MapAccess,
    ) -> Option<serde_wasm_host::Usize> {
        todo!()
    }

    fn enum_access_erased_variant_seed(
        &mut self,
        self_: &Self::EnumAccess,
        seed: serde_wasm_host::DeserializeSeedHandle,
    ) -> Result<(serde_wasm_host::DeValueHandle, Self::VariantAccess), Self::DeError> {
        todo!()
    }

    fn variant_access_unit_variant(
        &mut self,
        self_: &Self::VariantAccess,
    ) -> Result<(), Self::DeError> {
        todo!()
    }

    fn variant_access_newtype_variant_seed(
        &mut self,
        self_: &Self::VariantAccess,
        seed: serde_wasm_host::DeserializeSeedHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn variant_access_tuple_variant(
        &mut self,
        self_: &Self::VariantAccess,
        len: serde_wasm_host::Usize,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }

    fn variant_access_struct_variant(
        &mut self,
        self_: &Self::VariantAccess,
        fields: Vec<&str>,
        visitor: serde_wasm_host::VisitorHandle,
    ) -> Result<serde_wasm_host::DeValueHandle, Self::DeError> {
        todo!()
    }
}
