use std::{
    collections::HashMap,
    ffi::{CStr, CString},
};

use regex::bytes::Regex;

#[derive(Debug)]
pub struct PtxJIT {
    ptx_slices: Box<[PtxElement]>,
    last_arguments: Option<Box<[Box<[u8]>]>>,
    last_ptx: CString,
}

#[derive(Debug)]
enum PtxLoadWidth {
    B2,
    B4,
    B8,
}

#[derive(Debug)]
enum PtxElement {
    Source {
        ptx: Box<[u8]>,
    },
    ConstLoad {
        ptx: Box<[u8]>,
        parameter_index: usize,
        byte_offset: usize,
        load_width: PtxLoadWidth,
        register: Box<[u8]>,
    },
}

impl PtxJIT {
    pub fn new(ptx: &CStr) -> Self {
        let const_marker_regex =
            Regex::new(r"(?-u)// <rust-cuda-const-marker-(?P<tmpreg>%r\d+)-(?P<param>\d+)> //")
                .unwrap();
        let const_base_register_regex = Regex::new(
            r"(?-u)ld\.global\.u32\s*(?P<tmpreg>%r\d+)\s*,\s*\[(?P<basereg>%r[ds]?\d+)]\s*;",
        )
        .unwrap();
        let const_load_regex = Regex::new(
            r"(?x-u)(?P<instruction>ld\.global\.[suf](?P<loadwidth>16|32|64)\s*(?P<constreg>
            %[rf][sd]?\d+),\s*\[(?P<basereg>%r[ds]?\d+)(?:\+(?P<loadoffset>\d+))?\]\s*;)",
        )
        .unwrap();

        let ptx = ptx.to_bytes();

        let mut const_markers: HashMap<&[u8], usize> = HashMap::new();

        // Find injected rust-cuda-const-markers which identify dummy register rxx
        for const_marker in const_marker_regex.captures_iter(ptx) {
            if let Some(tmpreg) = const_marker.name("tmpreg").map(|s| s.as_bytes()) {
                if let Some(param) = const_marker
                    .name("param")
                    .map(|s| s.as_bytes())
                    .and_then(|b| std::str::from_utf8(b).ok())
                    .and_then(|s| s.parse().ok())
                {
                    const_markers.insert(tmpreg, param);
                }
            }
        }
        // const_markers now contains a mapping rxx => param index

        let mut const_base_registers: HashMap<&[u8], usize> = HashMap::new();

        // Find base register ryy which was used in `ld.global.u32 rxx, [ryy];`
        for const_base_register in const_base_register_regex.captures_iter(ptx) {
            if let Some(tmpreg) = const_base_register.name("tmpreg").map(|s| s.as_bytes()) {
                if let Some(param) = const_markers.get(tmpreg) {
                    if let Some(basereg) = const_base_register.name("basereg").map(|s| s.as_bytes())
                    {
                        const_base_registers.insert(basereg, *param);
                    }
                }
            }
        }
        // const_base_registers now contains a mapping ryy => param index

        let mut from_index = 0_usize;
        let mut last_slice = Vec::new();

        let mut ptx_slices: Vec<PtxElement> = Vec::new();

        for const_load_instruction in const_load_regex.captures_iter(ptx) {
            if let Some(basereg) = const_load_instruction.name("basereg").map(|s| s.as_bytes()) {
                if let Some(param) = const_base_registers.get(basereg) {
                    if let Some(loadwidth) = match const_load_instruction
                        .name("loadwidth")
                        .map(|s| s.as_bytes())
                    {
                        Some(&[0x31, 0x36]) => Some(PtxLoadWidth::B2),
                        Some(&[0x33, 0x32]) => Some(PtxLoadWidth::B4),
                        Some(&[0x36, 0x34]) => Some(PtxLoadWidth::B8),
                        _ => None,
                    } {
                        if let Some(constreg) = const_load_instruction
                            .name("constreg")
                            .map(|s| s.as_bytes())
                        {
                            if let Some(loadoffset) = std::str::from_utf8(
                                const_load_instruction
                                    .name("loadoffset")
                                    .map(|s| s.as_bytes())
                                    .unwrap_or(&[0x30]),
                            )
                            .ok()
                            .and_then(|s| s.parse().ok())
                            {
                                if let Some((range, instruction)) = const_load_instruction
                                    .name("instruction")
                                    .map(|s| (s.range(), s.as_bytes()))
                                {
                                    last_slice.extend_from_slice(&ptx[from_index..range.start]);

                                    ptx_slices.push(PtxElement::Source {
                                        ptx: std::mem::replace(&mut last_slice, Vec::new())
                                            .into_boxed_slice(),
                                    });

                                    from_index = range.end;

                                    ptx_slices.push(PtxElement::ConstLoad {
                                        ptx: instruction.to_owned().into_boxed_slice(),
                                        parameter_index: *param,
                                        byte_offset: loadoffset,
                                        load_width: loadwidth,
                                        register: constreg.to_owned().into_boxed_slice(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        last_slice.extend_from_slice(&ptx[from_index..ptx.len()]);

        if !last_slice.is_empty() {
            ptx_slices.push(PtxElement::Source {
                ptx: last_slice.into_boxed_slice(),
            });
        }

        Self {
            ptx_slices: ptx_slices.into_boxed_slice(),
            last_arguments: None,
            last_ptx: unsafe { CString::from_vec_unchecked(ptx.to_owned()) },
        }
    }

    pub fn with_arguments(&mut self, arguments: Option<Box<[Box<[u8]>]>>) -> &CStr {
        if self.last_arguments != arguments {
            self.last_arguments = arguments;

            let mut output_ptx = Vec::new();

            if let Some(args) = &self.last_arguments {
                for element in self.ptx_slices.iter() {
                    match element {
                        PtxElement::Source { ptx } => output_ptx.extend_from_slice(&ptx),
                        PtxElement::ConstLoad {
                            ptx,
                            parameter_index,
                            byte_offset,
                            load_width,
                            register,
                        } => {
                            if let Some(arg) = args.get(*parameter_index) {
                                if let Some(bytes) = arg.get(
                                    *byte_offset
                                        ..byte_offset
                                            + match load_width {
                                                PtxLoadWidth::B2 => 2,
                                                PtxLoadWidth::B4 => 4,
                                                PtxLoadWidth::B8 => 8,
                                            },
                                ) {
                                    output_ptx.extend_from_slice("mov.".as_bytes());
                                    output_ptx.extend_from_slice(if register.contains(&0x72) {
                                        "u".as_bytes()
                                    } else {
                                        "f".as_bytes()
                                    });
                                    output_ptx.extend_from_slice(if register.contains(&0x73) {
                                        "16".as_bytes()
                                    } else if register.contains(&0x64) {
                                        "64".as_bytes()
                                    } else {
                                        "32".as_bytes()
                                    });

                                    output_ptx.extend_from_slice(" \t".as_bytes());

                                    output_ptx.extend_from_slice(&register);

                                    output_ptx.extend_from_slice(", 0x".as_bytes());

                                    for byte in bytes.iter().rev() {
                                        output_ptx
                                            .extend_from_slice(format!("{:X}", byte).as_bytes());
                                    }

                                    output_ptx.extend_from_slice(";".as_bytes());

                                    continue;
                                }
                            }

                            // else
                            output_ptx.extend_from_slice(&ptx);
                        },
                    }
                }
            } else {
                for element in self.ptx_slices.iter() {
                    match element {
                        PtxElement::Source { ptx } | PtxElement::ConstLoad { ptx, .. } => {
                            output_ptx.extend_from_slice(&ptx)
                        },
                    }
                }
            }

            self.last_ptx = unsafe { CString::from_vec_unchecked(output_ptx) };
        }

        &self.last_ptx
    }
}
