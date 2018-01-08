
#![feature(test)]
#![feature(concat_idents)]
#![feature(i128_type)]
#![allow(unused)]

extern crate test;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::cmp;
use std::str::FromStr;

enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
}

thread_local! {
    static TEST_DATA: RefCell<Option<HashMap<&'static str, Rc<Vec<Value>>>>> = RefCell::new(None);
}

const METADATA: &'static str = "test_data/regex_metadata.txt";
const DEP_GRAPH: &'static str = "test_data/regex_dep_graph.txt";
const QUERY_CACHE: &'static str = "test_data/regex_query_cache.txt";

fn load_test_data(name: &'static str) -> Rc<Vec<Value>> {
    TEST_DATA.with(|test_data| {
        let mut map = test_data.borrow_mut();

        if map.is_none() {
            *map = Some(HashMap::new());
        }

        let mut map = map.as_mut().unwrap();

        if let Some(data) = map.get(name) {
            return data.clone();
        }

        let file = BufReader::new(File::open(name).unwrap());

        let mut data = Vec::new();

        for line in file.lines() {
            let line = line.unwrap();
            let sep = line.find(" ").unwrap();
            let ty = &line[..sep];
            let value = &line[sep + 1 ..];

            data.push(match ty {
                "u8" => Value::U8(u8::from_str_radix(value, 16).unwrap()),
                "u16" => Value::U16(u16::from_str_radix(value, 16).unwrap()),
                "u32" => Value::U32(u32::from_str_radix(value, 16).unwrap()),
                "u64" => Value::U64(u64::from_str_radix(value, 16).unwrap()),
                "u128" => Value::U128(u128::from_str_radix(value, 16).unwrap()),
                "usize" => Value::Usize(usize::from_str_radix(value, 16).unwrap()),
                "i8" => Value::I8(i8::from_str_radix(value, 16).unwrap()),
                "i16" => Value::I16(i16::from_str_radix(value, 16).unwrap()),
                "i32" => Value::I32(i32::from_str_radix(value, 16).unwrap()),
                "i64" => Value::I64(i64::from_str_radix(value, 16).unwrap()),
                "i128" => Value::I128(i128::from_str_radix(value, 16).unwrap()),
                "isize" => Value::Isize(isize::from_str_radix(value, 16).unwrap()),
                _ => panic!(),
            });
        }

        let data = Rc::new(data);
        map.insert(name, data.clone());
        data
    })
}



// Different ways of writing a slice to a vector -------------------------------

#[inline]
fn write_to_vec_solo(vec: &mut Vec<u8>, mut position: usize, bytes: &[u8]) {
    for &byte in bytes {
        if position == vec.len() {
            vec.push(byte);
        } else {
            vec[position] = byte;
        }
        position += 1;
    }
}

#[inline]
fn write_slice_to_vec(output: &mut Vec<u8>, start_position: usize, input: &[u8]) {
    let input_len = input.len();
    let capacity = output.len() - start_position;
    let first_half = cmp::min(capacity, input_len);

    if first_half > 0 {
        (&mut output[start_position..]).copy_from_slice(&input[.. first_half]);
    }

    if first_half < input_len {
        output.extend_from_slice(&input[first_half..]);
    }
}

#[cold]
#[inline(never)]
fn write_slice_to_vec_cold(output: &mut Vec<u8>, start_position: usize, input: &[u8]) {
    let input_len = input.len();
    let capacity = output.len() - start_position;
    let first_half = cmp::min(capacity, input_len);

    if first_half > 0 {
        (&mut output[start_position..]).copy_from_slice(&input[.. first_half]);
    }

    if first_half < input_len {
        output.extend_from_slice(&input[first_half..]);
    }
}

#[inline]
fn write_slice_to_vec_skewed(output: &mut Vec<u8>, start_position: usize, input: &[u8]) {
    if start_position == output.len() {
        output.extend_from_slice(input);
    } else {
        write_slice_to_vec_cold(output, start_position, input);
    }
}

#[inline]
fn write_to_vec(vec: &mut Vec<u8>, position: usize, byte: u8) {
    if position == vec.len() {
        vec.push(byte);
    } else {
        vec[position] = byte;
    }
}


// Uncompressed, little-endian -------------------------------------------------

macro_rules! impl_write_raw {
    ($fun:ident, $t:ident, $push:ident) => (
        #[inline]
        fn $fun(output: &mut Vec<u8>, start_position: usize, x: $t) -> usize {
            let x = x.to_le();
            let s = unsafe {
                ::std::slice::from_raw_parts(&x as *const $t as *const u8, ::std::mem::size_of::<$t>())
            };
            $push(output, start_position, s);
            ::std::mem::size_of::<$t>()
        }
    )
}

impl_write_raw!(write_raw_u8_solo, u8, write_to_vec_solo);
impl_write_raw!(write_raw_u16_solo, u16, write_to_vec_solo);
impl_write_raw!(write_raw_u32_solo, u32, write_to_vec_solo);
impl_write_raw!(write_raw_u64_solo, u64, write_to_vec_solo);
impl_write_raw!(write_raw_u128_solo, u128, write_to_vec_solo);
impl_write_raw!(write_raw_usize_solo, usize, write_to_vec_solo);
impl_write_raw!(write_raw_i8_solo, i8, write_to_vec_solo);
impl_write_raw!(write_raw_i16_solo, i16, write_to_vec_solo);
impl_write_raw!(write_raw_i32_solo, i32, write_to_vec_solo);
impl_write_raw!(write_raw_i64_solo, i64, write_to_vec_solo);
impl_write_raw!(write_raw_i128_solo, i128, write_to_vec_solo);
impl_write_raw!(write_raw_isize_solo, isize, write_to_vec_solo);

impl_write_raw!(write_raw_u8_slice, u8, write_slice_to_vec);
impl_write_raw!(write_raw_u16_slice, u16, write_slice_to_vec);
impl_write_raw!(write_raw_u32_slice, u32, write_slice_to_vec);
impl_write_raw!(write_raw_u64_slice, u64, write_slice_to_vec);
impl_write_raw!(write_raw_u128_slice, u128, write_slice_to_vec);
impl_write_raw!(write_raw_usize_slice, usize, write_slice_to_vec);
impl_write_raw!(write_raw_i8_slice, i8, write_slice_to_vec);
impl_write_raw!(write_raw_i16_slice, i16, write_slice_to_vec);
impl_write_raw!(write_raw_i32_slice, i32, write_slice_to_vec);
impl_write_raw!(write_raw_i64_slice, i64, write_slice_to_vec);
impl_write_raw!(write_raw_i128_slice, i128, write_slice_to_vec);
impl_write_raw!(write_raw_isize_slice, isize, write_slice_to_vec);

impl_write_raw!(write_raw_u8_skewed, u8, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_u16_skewed, u16, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_u32_skewed, u32, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_u64_skewed, u64, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_u128_skewed, u128, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_usize_skewed, usize, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_i8_skewed, i8, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_i16_skewed, i16, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_i32_skewed, i32, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_i64_skewed, i64, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_i128_skewed, i128, write_slice_to_vec_skewed);
impl_write_raw!(write_raw_isize_skewed, isize, write_slice_to_vec_skewed);



macro_rules! impl_write_shift {
    ($fun:ident, $t:ident) => (
        #[inline]
        fn $fun(out: &mut Vec<u8>, start_position: usize, x: $t) -> usize {
            for i in 0 .. ::std::mem::size_of::<$t>() {

                write_to_vec(out, start_position + i, (x >> i * 8) as u8);
            }

            ::std::mem::size_of::<$t>()
        }
    )
}

impl_write_shift!(write_shift_u8, u8);
impl_write_shift!(write_shift_u16, u16);
impl_write_shift!(write_shift_u32, u32);
impl_write_shift!(write_shift_u64, u64);
impl_write_shift!(write_shift_u128, u128);
impl_write_shift!(write_shift_usize, usize);


// Reference implementation of leb128 ------------------------------------------

macro_rules! impl_write_unsigned_leb128a {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(out: &mut Vec<u8>, start_position: usize, mut value: $int_ty) -> usize {

            let mut position = start_position;

            loop {
                let mut byte = (value as u8) & 0b0111_1111;
                value = value >> 7;

                if value == 0 {
                    if position == out.len() {
                        out.push(byte);
                    } else {
                        out[position] = byte;
                    }

                    return (1 + position) - start_position
                } else {
                    byte |= 0b1000_0000;
                    if position == out.len() {
                        out.push(byte);
                    } else {
                        out[position] = byte;
                    }
                }

                position += 1;
            }
        }
    )
}

impl_write_unsigned_leb128a!(write_leb128a_u16, u16);
impl_write_unsigned_leb128a!(write_leb128a_u32, u32);
impl_write_unsigned_leb128a!(write_leb128a_u64, u64);
impl_write_unsigned_leb128a!(write_leb128a_u128, u128);
impl_write_unsigned_leb128a!(write_leb128a_usize, usize);



// leb128 with fixed iteration counts ------------------------------------------

#[cfg(target_pointer_width = "32")]
const USIZE_LEB128_SIZE: usize = 5;
#[cfg(target_pointer_width = "64")]
const USIZE_LEB128_SIZE: usize = 10;

macro_rules! leb128_size {
    (u16) => (3);
    (u32) => (5);
    (u64) => (10);
    (u128) => (19);
    (usize) => (USIZE_LEB128_SIZE);
}

macro_rules! impl_write_unsigned_leb128b {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(out: &mut Vec<u8>, start_position: usize, mut value: $int_ty) -> usize {
            let mut position = 0;
            for i in 0 .. leb128_size!($int_ty) {
                let mut byte = (value & 0x7F) as u8;
                value >>= 7;
                if value != 0 {
                    byte |= 0x80;
                }

                write_to_vec(out, position, byte);
                position += 1;

                if value == 0 {
                    break;
                }
            }

            position
        }
    )
}

impl_write_unsigned_leb128b!(write_leb128b_u16_solo, u16);
impl_write_unsigned_leb128b!(write_leb128b_u32_solo, u32);
impl_write_unsigned_leb128b!(write_leb128b_u64_solo, u64);
impl_write_unsigned_leb128b!(write_leb128b_u128_solo, u128);
impl_write_unsigned_leb128b!(write_leb128b_usize_solo, usize);



// Current leb128 implementation from Rust compiler ----------------------------

#[inline]
pub fn write_unsigned_leb128_to<W>(mut value: u128, mut write: W) -> usize
    where W: FnMut(usize, u8)
{
    let mut position = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }

        write(position, byte);
        position += 1;

        if value == 0 {
            break;
        }
    }

    position
}

macro_rules! impl_write_unsigned_leb128c {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(out: &mut Vec<u8>, start_position: usize, value: $int_ty) -> usize {
            write_unsigned_leb128_to(value as u128, |i, v| write_to_vec(out, start_position+i, v))
        }
    )
}

impl_write_unsigned_leb128c!(write_leb128c_u16, u16);
impl_write_unsigned_leb128c!(write_leb128c_u32, u32);
impl_write_unsigned_leb128c!(write_leb128c_u64, u64);
impl_write_unsigned_leb128c!(write_leb128c_u128, u128);
impl_write_unsigned_leb128c!(write_leb128c_usize, usize);


// Unsafe leb128 implementation without bounds checks --------------------------

macro_rules! impl_write_unsigned_leb128d {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(out: &mut Vec<u8>, start_position: usize, mut value: $int_ty) -> usize {
            out.reserve(leb128_size!($int_ty));

            let mut position = start_position;
            for i in 0 .. leb128_size!($int_ty) {
                let mut byte = (value & 0x7F) as u8;
                value >>= 7;

                if value != 0 {
                    byte |= 0x80;
                }

                unsafe {
                    *out.get_unchecked_mut(position) = byte;
                }

                position += 1;

                if value == 0 {
                    break;
                }
            }

            let bytes_written = position - start_position;
            let initial_len = out.len();

            if start_position == initial_len {
                unsafe {
                    out.set_len(initial_len + bytes_written);
                }
            } else {
                let bytes_overwritten = initial_len - start_position;
                let additional_bytes = bytes_written.saturating_sub(bytes_overwritten);

                if additional_bytes > 0 {
                    unsafe {
                        out.set_len(initial_len + additional_bytes);
                    }
                }
            }

            bytes_written
        }
    )
}

impl_write_unsigned_leb128d!(write_leb128d_u16, u16);
impl_write_unsigned_leb128d!(write_leb128d_u32, u32);
impl_write_unsigned_leb128d!(write_leb128d_u64, u64);
impl_write_unsigned_leb128d!(write_leb128d_u128, u128);
impl_write_unsigned_leb128d!(write_leb128d_usize, usize);



#[cfg(target_pointer_width = "32")]
const USIZE_PREFIX_SIZE: usize = 5;
#[cfg(target_pointer_width = "64")]
const USIZE_PREFIX_SIZE: usize = 9;

macro_rules! prefix_size {
    (u16) => (3);
    (u32) => (5);
    (u64) => (9);
    (u128) => (17);
    (usize) => (USIZE_PREFIX_SIZE);
}


macro_rules! impl_write_unsigned_prefix {
    ($fn_name:ident, $int_ty:ident, $write:ident) => (
        #[inline]
        pub fn $fn_name(_out: &mut Vec<u8>, _start_position: usize, _value: $int_ty) -> usize {
            return 0;


            // let bits = prefix_size!($int_ty) * 8 - (value | 1).leading_zeros() as usize;
            // let prefix_bits = (bits + 7) / 8;
            // let total_bits = bits + prefix_bits;

            // if total_bits <= 64 {
            //     let mut value = ((value as u64) << prefix_bits) | (1 << (prefix_bits - 1));

            //     let total_bytes = (bits + prefix_bits + 7) / 8;

            //     let value = value.to_le();
            //     let value = unsafe {
            //         ::std::slice::from_raw_parts(&value as *const _ as *const u8, total_bytes)
            //     };

            //     write_slice_to_vec_skewed(out, start_position, value);
            //     total_bytes
            // } else {
            //     write_to_vec(out, start_position, 0);
            // }
        }
    )
}

impl_write_unsigned_prefix!(impl_write_usize_prefix, usize, write_slice_to_vec_skewed);
impl_write_unsigned_prefix!(impl_write_u32_prefix, u32, write_slice_to_vec_skewed);



macro_rules! impl_write_unsigned_lesqlite {
    ($fn_name:ident, $int_ty:ident, $write:ident) => (
        // #[inline]
        pub fn $fn_name(out: &mut Vec<u8>, start_position: usize, mut value: $int_ty) -> usize {
            const CUT1: $int_ty = 185;
            const CUT2: $int_ty = 249;

            if value < CUT1 {
                write_to_vec(out, start_position, value as u8);
                1
            } else if value <= (CUT1 + 255 + 256 * (CUT2 - 1 - CUT1)) {
                value -= CUT1;
                write_to_vec(out, start_position, CUT1 as u8 + ((value >> 8) as u8));
                write_to_vec(out, start_position + 1, value as u8);
                // write_to_vec_solo(out, start_position, &[CUT1 as u8 + ((value >> 8) as u8), value as u8]);
                return 2
            } else {
                let bits = ::std::mem::size_of::<$int_ty>() * 8 - value.leading_zeros() as usize;
                let bytes = (bits + 7) / 8;

                write_to_vec(out, start_position, CUT2 as u8 + (bytes - 2) as u8);

                let value = value.to_le();
                let value = unsafe {
                    ::std::slice::from_raw_parts(&value as *const _ as *const u8, bytes)
                };
                write_slice_to_vec_skewed(out, start_position + 1, value);
                bytes + 1
            }
        }
    )
}

impl_write_unsigned_lesqlite!(impl_write_usize_lesqlite, usize, write_slice_to_vec_skewed);
impl_write_unsigned_lesqlite!(impl_write_u32_lesqlite, u32, write_slice_to_vec_skewed);



// Benchmarks ------------------------------------------------------------------

macro_rules! impl_bench {
    ($bench_name:ident, $variant:ident, $fun:ident, $data:ident) => (
        #[bench]
        fn $bench_name(b: &mut test::Bencher) {

            let test_data = load_test_data($data);
            let test_data: Vec<_> = test_data.iter().filter_map(|entry| {
                match *entry {
                    Value::$variant(val) => Some(val),
                    _ => None,
                }
            }).collect();

            if let Some(&x) = test_data.get(0) {
                b.bytes =  (test_data.len() * ::std::mem::size_of_val(&x)) as u64;
            }

            let mut size = 0;

            b.iter(|| {
                let mut output = Vec::new();
                let mut position = 0;

                for &val in test_data.iter() {
                    position += $fun(&mut output, position, val);
                }

                size = position;
            });

            if b.bytes > 0 {
                print!("size: {}%, ", (100 * size) / (b.bytes as usize));
            }
        }
    )
}

impl_bench!(write_raw_u8_solo_query_cache, U8, write_raw_u8_solo, QUERY_CACHE);
impl_bench!(write_raw_u8_slice_query_cache, U8, write_raw_u8_slice, QUERY_CACHE);
impl_bench!(write_raw_u8_skewed_query_cache, U8, write_raw_u8_skewed, QUERY_CACHE);
impl_bench!(write_raw_u16_solo_query_cache, U16, write_raw_u16_solo, QUERY_CACHE);
impl_bench!(write_raw_u16_slice_query_cache, U16, write_raw_u16_slice, QUERY_CACHE);
impl_bench!(write_raw_u16_skewed_query_cache, U16, write_raw_u16_skewed, QUERY_CACHE);


impl_bench!(write_raw_u32_solo_metadata, U32, write_raw_u32_solo, METADATA);
impl_bench!(write_raw_u32_solo_dep_graph, U32, write_raw_u32_solo, DEP_GRAPH);
impl_bench!(write_raw_u32_solo_query_cache, U32, write_raw_u32_solo, QUERY_CACHE);

impl_bench!(write_raw_u32_slice_metadata, U32, write_raw_u32_slice, METADATA);
impl_bench!(write_raw_u32_slice_dep_graph, U32, write_raw_u32_slice, DEP_GRAPH);
impl_bench!(write_raw_u32_slice_query_cache, U32, write_raw_u32_slice, QUERY_CACHE);

impl_bench!(write_raw_u32_skewed_metadata, U32, write_raw_u32_skewed, METADATA);
impl_bench!(write_raw_u32_skewed_dep_graph, U32, write_raw_u32_skewed, DEP_GRAPH);
impl_bench!(write_raw_u32_skewed_query_cache, U32, write_raw_u32_skewed, QUERY_CACHE);

impl_bench!(write_raw_u64_solo_metadata, U64, write_raw_u64_solo, METADATA);
impl_bench!(write_raw_u64_solo_dep_graph, U64, write_raw_u64_solo, DEP_GRAPH);
impl_bench!(write_raw_u64_solo_query_cache, U64, write_raw_u64_solo, QUERY_CACHE);

impl_bench!(write_raw_u64_slice_metadata, U64, write_raw_u64_slice, METADATA);
impl_bench!(write_raw_u64_slice_dep_graph, U64, write_raw_u64_slice, DEP_GRAPH);
impl_bench!(write_raw_u64_slice_query_cache, U64, write_raw_u64_slice, QUERY_CACHE);

impl_bench!(write_raw_u64_skewed_metadata, U64, write_raw_u64_skewed, METADATA);
impl_bench!(write_raw_u64_skewed_dep_graph, U64, write_raw_u64_skewed, DEP_GRAPH);
impl_bench!(write_raw_u64_skewed_query_cache, U64, write_raw_u64_skewed, QUERY_CACHE);

impl_bench!(write_raw_usize_solo_metadata, Usize, write_raw_usize_solo, METADATA);
impl_bench!(write_raw_usize_solo_dep_graph, Usize, write_raw_usize_solo, DEP_GRAPH);
impl_bench!(write_raw_usize_solo_query_cache, Usize, write_raw_usize_solo, QUERY_CACHE);

impl_bench!(write_raw_usize_slice_metadata, Usize, write_raw_usize_slice, METADATA);
impl_bench!(write_raw_usize_slice_dep_graph, Usize, write_raw_usize_slice, DEP_GRAPH);
impl_bench!(write_raw_usize_slice_query_cache, Usize, write_raw_usize_slice, QUERY_CACHE);

impl_bench!(write_raw_usize_skewed_metadata, Usize, write_raw_usize_skewed, METADATA);
impl_bench!(write_raw_usize_skewed_dep_graph, Usize, write_raw_usize_skewed, DEP_GRAPH);
impl_bench!(write_raw_usize_skewed_query_cache, Usize, write_raw_usize_skewed, QUERY_CACHE);



impl_bench!(write_shift_u8_query_cache, U8, write_shift_u8, QUERY_CACHE);
impl_bench!(write_shift_u16_query_cache, U16, write_shift_u16, QUERY_CACHE);

impl_bench!(write_shift_u32_metadata, U32, write_shift_u32, METADATA);
impl_bench!(write_shift_u32_dep_graph, U32, write_shift_u32, DEP_GRAPH);
impl_bench!(write_shift_u32_query_cache, U32, write_shift_u32, QUERY_CACHE);

impl_bench!(write_shift_u64_metadata, U64, write_shift_u64, METADATA);
impl_bench!(write_shift_u64_dep_graph, U64, write_shift_u64, DEP_GRAPH);
impl_bench!(write_shift_u64_query_cache, U64, write_shift_u64, QUERY_CACHE);

impl_bench!(write_shift_usize_metadata, Usize, write_shift_usize, METADATA);
impl_bench!(write_shift_usize_dep_graph, Usize, write_shift_usize, DEP_GRAPH);
impl_bench!(write_shift_usize_query_cache, Usize, write_shift_usize, QUERY_CACHE);




impl_bench!(write_leb128b_u16_solo_query_cache, U16, write_leb128b_u16_solo, QUERY_CACHE);

impl_bench!(write_leb128b_u32_solo_metadata, U32, write_leb128b_u32_solo, METADATA);
impl_bench!(write_leb128b_u32_solo_dep_graph, U32, write_leb128b_u32_solo, DEP_GRAPH);
impl_bench!(write_leb128b_u32_solo_query_cache, U32, write_leb128b_u32_solo, QUERY_CACHE);

impl_bench!(write_leb128b_u64_solo_metadata, U64, write_leb128b_u64_solo, METADATA);
impl_bench!(write_leb128b_u64_solo_dep_graph, U64, write_leb128b_u64_solo, DEP_GRAPH);
impl_bench!(write_leb128b_u64_solo_query_cache, U64, write_leb128b_u64_solo, QUERY_CACHE);

impl_bench!(write_leb128b_usize_solo_metadata, Usize, write_leb128b_usize_solo, METADATA);
impl_bench!(write_leb128b_usize_solo_dep_graph, Usize, write_leb128b_usize_solo, DEP_GRAPH);
impl_bench!(write_leb128b_usize_solo_query_cache, Usize, write_leb128b_usize_solo, QUERY_CACHE);



impl_bench!(write_leb128a_u16_query_cache, U16, write_leb128a_u16, QUERY_CACHE);

impl_bench!(write_leb128a_u32_metadata, U32, write_leb128a_u32, METADATA);
impl_bench!(write_leb128a_u32_dep_graph, U32, write_leb128a_u32, DEP_GRAPH);
impl_bench!(write_leb128a_u32_query_cache, U32, write_leb128a_u32, QUERY_CACHE);

impl_bench!(write_leb128a_u64_metadata, U64, write_leb128a_u64, METADATA);
impl_bench!(write_leb128a_u64_dep_graph, U64, write_leb128a_u64, DEP_GRAPH);
impl_bench!(write_leb128a_u64_query_cache, U64, write_leb128a_u64, QUERY_CACHE);

impl_bench!(write_leb128a_usize_metadata, Usize, write_leb128a_usize, METADATA);
impl_bench!(write_leb128a_usize_dep_graph, Usize, write_leb128a_usize, DEP_GRAPH);
impl_bench!(write_leb128a_usize_query_cache, Usize, write_leb128a_usize, QUERY_CACHE);


impl_bench!(write_leb128c_u16_query_cache, U16, write_leb128c_u16, QUERY_CACHE);

impl_bench!(write_leb128c_u32_metadata, U32, write_leb128c_u32, METADATA);
impl_bench!(write_leb128c_u32_dep_graph, U32, write_leb128c_u32, DEP_GRAPH);
impl_bench!(write_leb128c_u32_query_cache, U32, write_leb128c_u32, QUERY_CACHE);

impl_bench!(write_leb128c_u64_metadata, U64, write_leb128c_u64, METADATA);
impl_bench!(write_leb128c_u64_dep_graph, U64, write_leb128c_u64, DEP_GRAPH);
impl_bench!(write_leb128c_u64_query_cache, U64, write_leb128c_u64, QUERY_CACHE);

impl_bench!(write_leb128c_usize_metadata, Usize, write_leb128c_usize, METADATA);
impl_bench!(write_leb128c_usize_dep_graph, Usize, write_leb128c_usize, DEP_GRAPH);
impl_bench!(write_leb128c_usize_query_cache, Usize, write_leb128c_usize, QUERY_CACHE);


impl_bench!(write_leb128d_u16_query_cache, U16, write_leb128d_u16, QUERY_CACHE);

impl_bench!(write_leb128d_u32_metadata, U32, write_leb128d_u32, METADATA);
impl_bench!(write_leb128d_u32_dep_graph, U32, write_leb128d_u32, DEP_GRAPH);
impl_bench!(write_leb128d_u32_query_cache, U32, write_leb128d_u32, QUERY_CACHE);

impl_bench!(write_leb128d_u64_metadata, U64, write_leb128d_u64, METADATA);
impl_bench!(write_leb128d_u64_dep_graph, U64, write_leb128d_u64, DEP_GRAPH);
impl_bench!(write_leb128d_u64_query_cache, U64, write_leb128d_u64, QUERY_CACHE);

impl_bench!(write_leb128d_usize_metadata, Usize, write_leb128d_usize, METADATA);
impl_bench!(write_leb128d_usize_dep_graph, Usize, write_leb128d_usize, DEP_GRAPH);
impl_bench!(write_leb128d_usize_query_cache, Usize, write_leb128d_usize, QUERY_CACHE);


impl_bench!(write_lesqlite_usize_metadata, Usize, impl_write_usize_lesqlite, METADATA);
impl_bench!(write_lesqlite_usize_dep_graph, Usize, impl_write_usize_lesqlite, DEP_GRAPH);
impl_bench!(write_lesqlite_usize_query_cache, Usize, impl_write_usize_lesqlite, QUERY_CACHE);

impl_bench!(write_lesqlite_u32_metadata, U32, impl_write_u32_lesqlite, METADATA);
impl_bench!(write_lesqlite_u32_dep_graph, U32, impl_write_u32_lesqlite, DEP_GRAPH);
impl_bench!(write_lesqlite_u32_query_cache, U32, impl_write_u32_lesqlite, QUERY_CACHE);
















#[inline]
fn read_unsigned_leb128_ref(data: &[u8], start_position: usize) -> (u128, usize) {
    let mut result = 0;
    let mut shift = 0;
    let mut position = start_position;
    loop {
        let byte = data[position];
        position += 1;
        result |= ((byte & 0x7F) as u128) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
    }

    (result, position - start_position)
}

macro_rules! impl_read_unsigned_leb128_ref {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(data: &[u8], start_position: usize) -> ($int_ty, usize) {
            let (val, read) = read_unsigned_leb128_ref(data, start_position);
            (val as $int_ty, read)
        }
    )
}

impl_read_unsigned_leb128_ref!(read_leb128_ref_u16, u16);
impl_read_unsigned_leb128_ref!(read_leb128_ref_u32, u32);
impl_read_unsigned_leb128_ref!(read_leb128_ref_u64, u64);
impl_read_unsigned_leb128_ref!(read_leb128_ref_u128, u128);
impl_read_unsigned_leb128_ref!(read_leb128_ref_usize, usize);


macro_rules! impl_read_unsigned_leb128_fixed {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(data: &[u8], start_position: usize) -> ($int_ty, usize) {
            let mut result = 0;
            let mut shift = 0;
            let mut position = start_position;

            for _ in 0 .. leb128_size!($int_ty) {
                let byte = data[position];
                position += 1;
                result |= ((byte & 0x7F) as u128) << shift;
                if (byte & 0x80) == 0 {
                    break;
                }
                shift += 7;
            }

            (result as $int_ty, position - start_position)
        }
    )
}

impl_read_unsigned_leb128_fixed!(read_leb128_fixed_u16, u16);
impl_read_unsigned_leb128_fixed!(read_leb128_fixed_u32, u32);
impl_read_unsigned_leb128_fixed!(read_leb128_fixed_u64, u64);
impl_read_unsigned_leb128_fixed!(read_leb128_fixed_u128, u128);
impl_read_unsigned_leb128_fixed!(read_leb128_fixed_usize, usize);


macro_rules! impl_read_unsigned_leb128_fixed2 {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(data: &[u8], start_position: usize) -> ($int_ty, usize) {
            let mut result: $int_ty = 0;
            let mut shift = 0;
            let mut position = start_position;

            for _ in 0 .. leb128_size!($int_ty) {
                let byte = data[position];
                position += 1;
                result |= ((byte & 0x7F) as $int_ty) << shift;
                if (byte & 0x80) == 0 {
                    break;
                }
                shift += 7;
            }

            (result, position - start_position)
        }
    )
}

impl_read_unsigned_leb128_fixed2!(read_leb128_fixed2_u16, u16);
impl_read_unsigned_leb128_fixed2!(read_leb128_fixed2_u32, u32);
impl_read_unsigned_leb128_fixed2!(read_leb128_fixed2_u64, u64);
impl_read_unsigned_leb128_fixed2!(read_leb128_fixed2_u128, u128);
impl_read_unsigned_leb128_fixed2!(read_leb128_fixed2_usize, usize);


macro_rules! impl_read_unsigned_leb128_unsafe {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(data: &[u8], start_position: usize) -> ($int_ty, usize) {
            unsafe {
                let mut result: $int_ty = 0;
                let mut shift = 0;
                let mut position = start_position;

                for _ in 0 .. leb128_size!($int_ty) {
                    let byte = *data.get_unchecked(position);
                    position += 1;
                    result |= ((byte & 0x7F) as $int_ty) << shift;
                    if (byte & 0x80) == 0 {
                        break;
                    }
                    shift += 7;
                }

                assert!(position <= data.len());

                (result, position - start_position)
            }
        }
    )
}

impl_read_unsigned_leb128_unsafe!(read_leb128_unsafe_u16, u16);
impl_read_unsigned_leb128_unsafe!(read_leb128_unsafe_u32, u32);
impl_read_unsigned_leb128_unsafe!(read_leb128_unsafe_u64, u64);
impl_read_unsigned_leb128_unsafe!(read_leb128_unsafe_u128, u128);
impl_read_unsigned_leb128_unsafe!(read_leb128_unsafe_usize, usize);




macro_rules! impl_read_unsigned_leb128_weird {
    ($fn_name:ident, $int_ty:ident) => (
        #[inline]
        pub fn $fn_name(data: &[u8], start_position: usize) -> ($int_ty, usize) {
            unsafe {
                let mut result: $int_ty = 0;
                let mut shift = 0;
                let mut position = start_position;

                for _ in 0 .. leb128_size!($int_ty) {
                    let byte = *data.get_unchecked(position);
                    let mult = (byte >> 7) as usize;
                    position += mult;
                    result |= ((byte & 0x7F) as $int_ty) << shift;
                    shift += 7 * mult;
                }

                position += 1;

                assert!(position <= data.len());

                (result, position - start_position)
            }
        }
    )
}

impl_read_unsigned_leb128_weird!(read_leb128_weird_u16, u16);
impl_read_unsigned_leb128_weird!(read_leb128_weird_u32, u32);
impl_read_unsigned_leb128_weird!(read_leb128_weird_u64, u64);
impl_read_unsigned_leb128_weird!(read_leb128_weird_u128, u128);
impl_read_unsigned_leb128_weird!(read_leb128_weird_usize, usize);


macro_rules! impl_read_bench {
    ($bench_name:ident, $variant:ident, $fun:ident, $data:ident) => (
        #[bench]
        fn $bench_name(b: &mut test::Bencher) {

            let test_data = load_test_data($data);
            let test_data: Vec<_> = test_data.iter().filter_map(|entry| {
                match *entry {
                    Value::$variant(val) => Some(val),
                    _ => None,
                }
            }).collect();


            if let Some(&x) = test_data.get(0) {
                b.bytes =  (test_data.len() * ::std::mem::size_of_val(&x)) as u64;
            }

            let mut encoded = Vec::new();

            for &val in test_data.iter() {
                let pos = encoded.len();
                write_leb128c_u128(&mut encoded, pos, val as u128);
            }

            b.iter(|| {
                let mut position = 0;
                for _ in 0 .. test_data.len() {
                    let (val, count) = $fun(&mut encoded, position);
                    test::black_box(val);
                    position += count;
                }
            });
        }
    )
}

impl_read_bench!(read_leb128_ref_u16_dep_graph, Usize, read_leb128_ref_u16, DEP_GRAPH);
impl_read_bench!(read_leb128_ref_u32_dep_graph, Usize, read_leb128_ref_u32, DEP_GRAPH);
impl_read_bench!(read_leb128_ref_u64_dep_graph, Usize, read_leb128_ref_u64, DEP_GRAPH);
impl_read_bench!(read_leb128_ref_u128_dep_graph, Usize, read_leb128_ref_u128, DEP_GRAPH);
impl_read_bench!(read_leb128_ref_usize_dep_graph, Usize, read_leb128_ref_usize, DEP_GRAPH);

impl_read_bench!(read_leb128_ref_u16_metadata, U16, read_leb128_ref_u16, METADATA);
impl_read_bench!(read_leb128_ref_u32_metadata, U32, read_leb128_ref_u32, METADATA);
impl_read_bench!(read_leb128_ref_u64_metadata, U64, read_leb128_ref_u64, METADATA);
impl_read_bench!(read_leb128_ref_u128_metadata, U128, read_leb128_ref_u128, METADATA);
impl_read_bench!(read_leb128_ref_usize_metadata, Usize, read_leb128_ref_usize, METADATA);

impl_read_bench!(read_leb128_ref_u16_query_cache, U16, read_leb128_ref_u16, QUERY_CACHE);
impl_read_bench!(read_leb128_ref_u32_query_cache, U32, read_leb128_ref_u32, QUERY_CACHE);
impl_read_bench!(read_leb128_ref_u64_query_cache, U64, read_leb128_ref_u64, QUERY_CACHE);
impl_read_bench!(read_leb128_ref_u128_query_cache, U128, read_leb128_ref_u128, QUERY_CACHE);
impl_read_bench!(read_leb128_ref_usize_query_cache, Usize, read_leb128_ref_usize, QUERY_CACHE);




impl_read_bench!(read_leb128_fixed_u16_dep_graph, Usize, read_leb128_fixed_u16, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed_u32_dep_graph, Usize, read_leb128_fixed_u32, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed_u64_dep_graph, Usize, read_leb128_fixed_u64, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed_u128_dep_graph, Usize, read_leb128_fixed_u128, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed_usize_dep_graph, Usize, read_leb128_fixed_usize, DEP_GRAPH);

impl_read_bench!(read_leb128_fixed_u16_metadata, U16, read_leb128_fixed_u16, METADATA);
impl_read_bench!(read_leb128_fixed_u32_metadata, U32, read_leb128_fixed_u32, METADATA);
impl_read_bench!(read_leb128_fixed_u64_metadata, U64, read_leb128_fixed_u64, METADATA);
impl_read_bench!(read_leb128_fixed_u128_metadata, U128, read_leb128_fixed_u128, METADATA);
impl_read_bench!(read_leb128_fixed_usize_metadata, Usize, read_leb128_fixed_usize, METADATA);

impl_read_bench!(read_leb128_fixed_u16_query_cache, U16, read_leb128_fixed_u16, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed_u32_query_cache, U32, read_leb128_fixed_u32, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed_u64_query_cache, U64, read_leb128_fixed_u64, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed_u128_query_cache, U128, read_leb128_fixed_u128, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed_usize_query_cache, Usize, read_leb128_fixed_usize, QUERY_CACHE);





impl_read_bench!(read_leb128_fixed2_u16_dep_graph, Usize, read_leb128_fixed2_u16, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed2_u32_dep_graph, Usize, read_leb128_fixed2_u32, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed2_u64_dep_graph, Usize, read_leb128_fixed2_u64, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed2_u128_dep_graph, Usize, read_leb128_fixed2_u128, DEP_GRAPH);
impl_read_bench!(read_leb128_fixed2_usize_dep_graph, Usize, read_leb128_fixed2_usize, DEP_GRAPH);

impl_read_bench!(read_leb128_fixed2_u16_metadata, U16, read_leb128_fixed2_u16, METADATA);
impl_read_bench!(read_leb128_fixed2_u32_metadata, U32, read_leb128_fixed2_u32, METADATA);
impl_read_bench!(read_leb128_fixed2_u64_metadata, U64, read_leb128_fixed2_u64, METADATA);
impl_read_bench!(read_leb128_fixed2_u128_metadata, U128, read_leb128_fixed2_u128, METADATA);
impl_read_bench!(read_leb128_fixed2_usize_metadata, Usize, read_leb128_fixed2_usize, METADATA);

impl_read_bench!(read_leb128_fixed2_u16_query_cache, U16, read_leb128_fixed2_u16, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed2_u32_query_cache, U32, read_leb128_fixed2_u32, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed2_u64_query_cache, U64, read_leb128_fixed2_u64, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed2_u128_query_cache, U128, read_leb128_fixed2_u128, QUERY_CACHE);
impl_read_bench!(read_leb128_fixed2_usize_query_cache, Usize, read_leb128_fixed2_usize, QUERY_CACHE);



impl_read_bench!(read_leb128_unsafe_u16_dep_graph, Usize, read_leb128_unsafe_u16, DEP_GRAPH);
impl_read_bench!(read_leb128_unsafe_u32_dep_graph, Usize, read_leb128_unsafe_u32, DEP_GRAPH);
impl_read_bench!(read_leb128_unsafe_u64_dep_graph, Usize, read_leb128_unsafe_u64, DEP_GRAPH);
impl_read_bench!(read_leb128_unsafe_u128_dep_graph, Usize, read_leb128_unsafe_u128, DEP_GRAPH);
impl_read_bench!(read_leb128_unsafe_usize_dep_graph, Usize, read_leb128_unsafe_usize, DEP_GRAPH);

impl_read_bench!(read_leb128_unsafe_u16_metadata, U16, read_leb128_unsafe_u16, METADATA);
impl_read_bench!(read_leb128_unsafe_u32_metadata, U32, read_leb128_unsafe_u32, METADATA);
impl_read_bench!(read_leb128_unsafe_u64_metadata, U64, read_leb128_unsafe_u64, METADATA);
impl_read_bench!(read_leb128_unsafe_u128_metadata, U128, read_leb128_unsafe_u128, METADATA);
impl_read_bench!(read_leb128_unsafe_usize_metadata, Usize, read_leb128_unsafe_usize, METADATA);

impl_read_bench!(read_leb128_unsafe_u16_query_cache, U16, read_leb128_unsafe_u16, QUERY_CACHE);
impl_read_bench!(read_leb128_unsafe_u32_query_cache, U32, read_leb128_unsafe_u32, QUERY_CACHE);
impl_read_bench!(read_leb128_unsafe_u64_query_cache, U64, read_leb128_unsafe_u64, QUERY_CACHE);
impl_read_bench!(read_leb128_unsafe_u128_query_cache, U128, read_leb128_unsafe_u128, QUERY_CACHE);
impl_read_bench!(read_leb128_unsafe_usize_query_cache, Usize, read_leb128_unsafe_usize, QUERY_CACHE);




impl_read_bench!(read_leb128_weird_u16_dep_graph, Usize, read_leb128_weird_u16, DEP_GRAPH);
impl_read_bench!(read_leb128_weird_u32_dep_graph, Usize, read_leb128_weird_u32, DEP_GRAPH);
impl_read_bench!(read_leb128_weird_u64_dep_graph, Usize, read_leb128_weird_u64, DEP_GRAPH);
impl_read_bench!(read_leb128_weird_u128_dep_graph, Usize, read_leb128_weird_u128, DEP_GRAPH);
impl_read_bench!(read_leb128_weird_usize_dep_graph, Usize, read_leb128_weird_usize, DEP_GRAPH);

impl_read_bench!(read_leb128_weird_u16_metadata, U16, read_leb128_weird_u16, METADATA);
impl_read_bench!(read_leb128_weird_u32_metadata, U32, read_leb128_weird_u32, METADATA);
impl_read_bench!(read_leb128_weird_u64_metadata, U64, read_leb128_weird_u64, METADATA);
impl_read_bench!(read_leb128_weird_u128_metadata, U128, read_leb128_weird_u128, METADATA);
impl_read_bench!(read_leb128_weird_usize_metadata, Usize, read_leb128_weird_usize, METADATA);

impl_read_bench!(read_leb128_weird_u16_query_cache, U16, read_leb128_weird_u16, QUERY_CACHE);
impl_read_bench!(read_leb128_weird_u32_query_cache, U32, read_leb128_weird_u32, QUERY_CACHE);
impl_read_bench!(read_leb128_weird_u64_query_cache, U64, read_leb128_weird_u64, QUERY_CACHE);
impl_read_bench!(read_leb128_weird_u128_query_cache, U128, read_leb128_weird_u128, QUERY_CACHE);
impl_read_bench!(read_leb128_weird_usize_query_cache, Usize, read_leb128_weird_usize, QUERY_CACHE);
