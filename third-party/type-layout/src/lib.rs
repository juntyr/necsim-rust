/*!
[![GitHub CI Status](https://github.com/LPGhatguy/type-layout/workflows/CI/badge.svg)](https://github.com/LPGhatguy/type-layout/actions)
[![type-layout on crates.io](https://img.shields.io/crates/v/type-layout.svg)](https://crates.io/crates/type-layout)
[![type-layout docs](https://img.shields.io/badge/docs-docs.rs-orange.svg)](https://docs.rs/type-layout)

type-layout is a type layout debugging aid, providing a `#[derive]`able trait
that reports:
- The type's name, size, and minimum alignment
- Each field's name, type, offset, and size
- Padding due to alignment requirements

**type-layout currently only functions on structs with named fields.** This is a
temporary limitation.

## Examples

The layout of types is only defined if they're `#[repr(C)]`. This crate works on
non-`#[repr(C)]` types, but their layout is unpredictable.

```rust
use type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C)]
struct Foo {
    a: u8,
    b: u32,
}

println!("{}", Foo::type_layout());
// prints:
// Foo (size 8, alignment 4)
// | Offset | Name      | Size |
// | ------ | --------- | ---- |
// | 0      | a         | 1    |
// | 1      | [padding] | 3    |
// | 4      | b         | 4    |
```

Over-aligned types have trailing padding, which can be a source of bugs in some
FFI scenarios:

```rust
use type_layout::TypeLayout;

#[derive(TypeLayout)]
#[repr(C, align(128))]
struct OverAligned {
    value: u8,
}

println!("{}", OverAligned::type_layout());
// prints:
// OverAligned (size 128, alignment 128)
// | Offset | Name      | Size |
// | ------ | --------- | ---- |
// | 0      | value     | 1    |
// | 1      | [padding] | 127  |
```

## Minimum Supported Rust Version (MSRV)

type-layout supports Rust 1.34.1 and newer. Until type-layout reaches 1.0,
changes to the MSRV will require major version bumps. After 1.0, MSRV changes
will only require minor version bumps, but will need significant justification.
*/

#![deny(clippy::pedantic)]
#![no_std]

#[doc(hidden)]
pub extern crate alloc;

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt::{self, Display};
use core::str;

pub use type_layout_derive::TypeLayout;

#[doc(hidden)]
pub use memoffset;

#[doc(hidden)]
pub use tynm;

pub trait TypeLayout {
    fn type_layout() -> TypeLayoutInfo;
}

#[derive(Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeLayoutInfo {
    pub name: Cow<'static, str>,
    pub size: usize,
    pub alignment: usize,
    pub structure: TypeStructure,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeStructure {
    StructLike {
        fields: Vec<Field>,
    },
    Enum {
        variants: Vec<(Cow<'static, str>, Vec<Field>)>
    },
}

#[derive(Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum Field {
    Field {
        name: Cow<'static, str>,
        ty: Cow<'static, str>,
        offset: usize,
        size: usize,
        alignment: usize,
    },
    Padding {
        offset: usize,
        size: usize,
    },
}

impl fmt::Display for TypeLayoutInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "{} (size {}, alignment {})",
            self.name, self.size, self.alignment
        )?;

        match &self.structure {
            TypeStructure::StructLike { fields } => format_fields(fields, formatter)?,
            TypeStructure::Enum { variants } => {
                if variants.is_empty() {
                    return writeln!(formatter, "  never");
                }
                
                for (variant, fields) in variants {
                    writeln!(formatter, "- {}:", variant)?;

                    format_fields(fields, formatter)?;
                }
            }
        }

        Ok(())
    }
}

fn format_fields(fields: &[Field], formatter: &mut fmt::Formatter) -> fmt::Result {
        if fields.is_empty() {
            return writeln!(formatter, "  unit");            
        }

        let longest_name = "Name".len().max(fields
            .iter()
            .map(|field| match field {
                Field::Field { name, .. } => name.len(),
                Field::Padding { .. } => "[padding]".len(),
            })
            .max()
            .unwrap_or(0));

        let longest_type = "Type".len().max(fields
            .iter()
            .map(|field| match field {
                Field::Field { ty, .. } => ty.len(),
                Field::Padding { .. } => "[padding]".len(),
            })
            .max()
            .unwrap_or(0));

        let widths = RowWidths {
            offset: "Offset".len(),
            name: longest_name,
            ty: longest_type,
            size: "Size".len(),
            alignment: "Alignment".len(),
        };

        write_row(
            formatter,
            widths,
            &Row {
                offset: "Offset",
                name: "Name",
                ty: "Type",
                size: "Size",
                alignment: "Alignment",
            },
        )?;

        write_row(
            formatter,
            widths,
            &Row {
                offset: "------",
                name: str::repeat("-", longest_name),
                ty: str::repeat("-", longest_type),
                size: "----",
                alignment: "---------",
            },
        )?;

        for field in fields {
            match field {
                Field::Field { name, ty, offset, size, alignment } => {
                    write_row(formatter, widths, &Row { offset, name, ty, size, alignment })?;
                }
                Field::Padding { offset, size } => {
                    write_row(
                        formatter,
                        widths,
                        &Row {
                            offset,
                            name: "[padding]",
                            ty: "[padding]",
                            size,
                            alignment: 0,
                        },
                    )?;
                }
            }
        }

        Ok(())
}

#[derive(Clone, Copy)]
struct RowWidths {
    offset: usize,
    name: usize,
    ty: usize,
    size: usize,
    alignment: usize,
}

struct Row<O, N, T, S, A> {
    offset: O,
    name: N,
    ty: T,
    size: S,
    alignment: A,
}

fn write_row<O: Display, N: Display, T: Display, S: Display, A: Display>(
    formatter: &mut fmt::Formatter,
    widths: RowWidths,
    row: &Row<O, N, T, S, A>,
) -> fmt::Result {
    writeln!(
        formatter,
        "| {:<offset_width$} | {:<name_width$} | {:<type_width$} | {:<size_width$} | {:<alignment_width$} |",
        row.offset,
        row.name,
        row.ty,
        row.size,
        row.alignment,
        offset_width = widths.offset,
        name_width = widths.name,
        type_width = widths.ty,
        size_width = widths.size,
        alignment_width = widths.alignment,
    )
}
