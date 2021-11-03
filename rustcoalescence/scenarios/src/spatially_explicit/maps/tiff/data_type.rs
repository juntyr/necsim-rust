use tiff::{decoder::DecodingBuffer, tags::SampleFormat};

#[allow(clippy::module_name_repetitions)]
pub trait TiffDataType:
    Clone
    + std::fmt::Debug
    + Default
    + std::str::FromStr<Err: std::error::Error + Send + Sync + 'static>
    + std::cmp::PartialEq
{
    const BIT_WIDTH: u8;
    const SAMPLE_FORMAT: SampleFormat;

    fn decoding_buffer_from_data(data: &mut [Self]) -> DecodingBuffer;
}

impl TiffDataType for u8 {
    const BIT_WIDTH: u8 = 8_u8;
    const SAMPLE_FORMAT: SampleFormat = SampleFormat::Uint;

    fn decoding_buffer_from_data(data: &mut [Self]) -> DecodingBuffer {
        DecodingBuffer::U8(data)
    }
}

impl TiffDataType for u16 {
    const BIT_WIDTH: u8 = 16_u8;
    const SAMPLE_FORMAT: SampleFormat = SampleFormat::Uint;

    fn decoding_buffer_from_data(data: &mut [Self]) -> DecodingBuffer {
        DecodingBuffer::U16(data)
    }
}

impl TiffDataType for u32 {
    const BIT_WIDTH: u8 = 32_u8;
    const SAMPLE_FORMAT: SampleFormat = SampleFormat::Uint;

    fn decoding_buffer_from_data(data: &mut [Self]) -> DecodingBuffer {
        DecodingBuffer::U32(data)
    }
}

impl TiffDataType for u64 {
    const BIT_WIDTH: u8 = 64_u8;
    const SAMPLE_FORMAT: SampleFormat = SampleFormat::Uint;

    fn decoding_buffer_from_data(data: &mut [Self]) -> DecodingBuffer {
        DecodingBuffer::U64(data)
    }
}

impl TiffDataType for f32 {
    const BIT_WIDTH: u8 = 32_u8;
    const SAMPLE_FORMAT: SampleFormat = SampleFormat::IEEEFP;

    fn decoding_buffer_from_data(data: &mut [Self]) -> DecodingBuffer {
        DecodingBuffer::F32(data)
    }
}

impl TiffDataType for f64 {
    const BIT_WIDTH: u8 = 64_u8;
    const SAMPLE_FORMAT: SampleFormat = SampleFormat::IEEEFP;

    fn decoding_buffer_from_data(data: &mut [Self]) -> DecodingBuffer {
        DecodingBuffer::F64(data)
    }
}
