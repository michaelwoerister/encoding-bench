
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

#[inline(always)]
fn write_slice_to_vec_skewed(output: &mut Vec<u8>, start_position: usize, input: &[u8]) {
    if start_position == output.len() {
        output.extend_from_slice(input);
    } else {
        write_slice_to_vec_cold(output, start_position, input);
    }
}



// Uncompressed, little-endian -------------------------------------------------

macro_rules! impl_write_raw {
    ($fun:ident, $t:ident, $push:ident) => (
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
    ($fn_name:ident, $int_ty:ident, $write:ident) => (
        #[inline]
        pub fn $fn_name(out: &mut Vec<u8>, start_position: usize, mut value: $int_ty) -> usize {
            let mut encoded = [0u8; leb128_size!($int_ty)];

            for i in 0 .. leb128_size!($int_ty) {
                encoded[i] = (value as u8) & 0b0111_1111;
                value = value >> 7;

                if value == 0 {
                    let bytes_written = i + 1;
                    $write(out, start_position, &encoded[0 .. bytes_written]);
                    return bytes_written
                } else {
                    encoded[i] |= 0b1000_0000;
                }
            }

            unreachable!()
        }
    )
}

impl_write_unsigned_leb128b!(write_leb128b_u16_solo, u16, write_to_vec_solo);
impl_write_unsigned_leb128b!(write_leb128b_u32_solo, u32, write_to_vec_solo);
impl_write_unsigned_leb128b!(write_leb128b_u64_solo, u64, write_to_vec_solo);
impl_write_unsigned_leb128b!(write_leb128b_u128_solo, u128, write_to_vec_solo);
impl_write_unsigned_leb128b!(write_leb128b_usize_solo, usize, write_to_vec_solo);

impl_write_unsigned_leb128b!(write_leb128b_u16_slice, u16, write_slice_to_vec);
impl_write_unsigned_leb128b!(write_leb128b_u32_slice, u32, write_slice_to_vec);
impl_write_unsigned_leb128b!(write_leb128b_u64_slice, u64, write_slice_to_vec);
impl_write_unsigned_leb128b!(write_leb128b_u128_slice, u128, write_slice_to_vec);
impl_write_unsigned_leb128b!(write_leb128b_usize_slice, usize, write_slice_to_vec);

impl_write_unsigned_leb128b!(write_leb128b_u16_skewed, u16, write_slice_to_vec_skewed);
impl_write_unsigned_leb128b!(write_leb128b_u32_skewed, u32, write_slice_to_vec_skewed);
impl_write_unsigned_leb128b!(write_leb128b_u64_skewed, u64, write_slice_to_vec_skewed);
impl_write_unsigned_leb128b!(write_leb128b_u128_skewed, u128, write_slice_to_vec_skewed);
impl_write_unsigned_leb128b!(write_leb128b_usize_skewed, usize, write_slice_to_vec_skewed);



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

            b.iter(|| {
                let mut output = Vec::new();
                let mut position = 0;

                for &val in test_data.iter() {
                    position += $fun(&mut output, position, val);
                }
            });
        }
    )
}

impl_bench!(write_raw_u8_solo_metadata, U8, write_raw_u8_solo, METADATA);
impl_bench!(write_raw_u8_solo_dep_graph, U8, write_raw_u8_solo, DEP_GRAPH);
impl_bench!(write_raw_u8_solo_query_cache, U8, write_raw_u8_solo, QUERY_CACHE);

impl_bench!(write_raw_u8_slice_metadata, U8, write_raw_u8_slice, METADATA);
impl_bench!(write_raw_u8_slice_dep_graph, U8, write_raw_u8_slice, DEP_GRAPH);
impl_bench!(write_raw_u8_slice_query_cache, U8, write_raw_u8_slice, QUERY_CACHE);

impl_bench!(write_raw_u8_skewed_metadata, U8, write_raw_u8_skewed, METADATA);
impl_bench!(write_raw_u8_skewed_dep_graph, U8, write_raw_u8_skewed, DEP_GRAPH);
impl_bench!(write_raw_u8_skewed_query_cache, U8, write_raw_u8_skewed, QUERY_CACHE);

impl_bench!(write_raw_u16_solo_metadata, U16, write_raw_u16_solo, METADATA);
impl_bench!(write_raw_u16_solo_dep_graph, U16, write_raw_u16_solo, DEP_GRAPH);
impl_bench!(write_raw_u16_solo_query_cache, U16, write_raw_u16_solo, QUERY_CACHE);

impl_bench!(write_raw_u16_slice_metadata, U16, write_raw_u16_slice, METADATA);
impl_bench!(write_raw_u16_slice_dep_graph, U16, write_raw_u16_slice, DEP_GRAPH);
impl_bench!(write_raw_u16_slice_query_cache, U16, write_raw_u16_slice, QUERY_CACHE);

impl_bench!(write_raw_u16_skewed_metadata, U16, write_raw_u16_skewed, METADATA);
impl_bench!(write_raw_u16_skewed_dep_graph, U16, write_raw_u16_skewed, DEP_GRAPH);
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

impl_bench!(write_leb128b_u16_solo_metadata, U16, write_leb128b_u16_solo, METADATA);
impl_bench!(write_leb128b_u16_solo_dep_graph, U16, write_leb128b_u16_solo, DEP_GRAPH);
impl_bench!(write_leb128b_u16_solo_query_cache, U16, write_leb128b_u16_solo, QUERY_CACHE);

impl_bench!(write_leb128b_u16_slice_metadata, U16, write_leb128b_u16_slice, METADATA);
impl_bench!(write_leb128b_u16_slice_dep_graph, U16, write_leb128b_u16_slice, DEP_GRAPH);
impl_bench!(write_leb128b_u16_slice_query_cache, U16, write_leb128b_u16_slice, QUERY_CACHE);

impl_bench!(write_leb128b_u16_skewed_metadata, U16, write_leb128b_u16_skewed, METADATA);
impl_bench!(write_leb128b_u16_skewed_dep_graph, U16, write_leb128b_u16_skewed, DEP_GRAPH);
impl_bench!(write_leb128b_u16_skewed_query_cache, U16, write_leb128b_u16_skewed, QUERY_CACHE);

impl_bench!(write_leb128b_u32_solo_metadata, U32, write_leb128b_u32_solo, METADATA);
impl_bench!(write_leb128b_u32_solo_dep_graph, U32, write_leb128b_u32_solo, DEP_GRAPH);
impl_bench!(write_leb128b_u32_solo_query_cache, U32, write_leb128b_u32_solo, QUERY_CACHE);

impl_bench!(write_leb128b_u32_slice_metadata, U32, write_leb128b_u32_slice, METADATA);
impl_bench!(write_leb128b_u32_slice_dep_graph, U32, write_leb128b_u32_slice, DEP_GRAPH);
impl_bench!(write_leb128b_u32_slice_query_cache, U32, write_leb128b_u32_slice, QUERY_CACHE);

impl_bench!(write_leb128b_u32_skewed_metadata, U32, write_leb128b_u32_skewed, METADATA);
impl_bench!(write_leb128b_u32_skewed_dep_graph, U32, write_leb128b_u32_skewed, DEP_GRAPH);
impl_bench!(write_leb128b_u32_skewed_query_cache, U32, write_leb128b_u32_skewed, QUERY_CACHE);

impl_bench!(write_leb128b_u64_solo_metadata, U64, write_leb128b_u64_solo, METADATA);
impl_bench!(write_leb128b_u64_solo_dep_graph, U64, write_leb128b_u64_solo, DEP_GRAPH);
impl_bench!(write_leb128b_u64_solo_query_cache, U64, write_leb128b_u64_solo, QUERY_CACHE);

impl_bench!(write_leb128b_u64_slice_metadata, U64, write_leb128b_u64_slice, METADATA);
impl_bench!(write_leb128b_u64_slice_dep_graph, U64, write_leb128b_u64_slice, DEP_GRAPH);
impl_bench!(write_leb128b_u64_slice_query_cache, U64, write_leb128b_u64_slice, QUERY_CACHE);

impl_bench!(write_leb128b_u64_skewed_metadata, U64, write_leb128b_u64_skewed, METADATA);
impl_bench!(write_leb128b_u64_skewed_dep_graph, U64, write_leb128b_u64_skewed, DEP_GRAPH);
impl_bench!(write_leb128b_u64_skewed_query_cache, U64, write_leb128b_u64_skewed, QUERY_CACHE);

impl_bench!(write_leb128b_usize_solo_metadata, Usize, write_leb128b_usize_solo, METADATA);
impl_bench!(write_leb128b_usize_solo_dep_graph, Usize, write_leb128b_usize_solo, DEP_GRAPH);
impl_bench!(write_leb128b_usize_solo_query_cache, Usize, write_leb128b_usize_solo, QUERY_CACHE);

impl_bench!(write_leb128b_usize_slice_metadata, Usize, write_leb128b_usize_slice, METADATA);
impl_bench!(write_leb128b_usize_slice_dep_graph, Usize, write_leb128b_usize_slice, DEP_GRAPH);
impl_bench!(write_leb128b_usize_slice_query_cache, Usize, write_leb128b_usize_slice, QUERY_CACHE);

impl_bench!(write_leb128b_usize_skewed_metadata, Usize, write_leb128b_usize_skewed, METADATA);
impl_bench!(write_leb128b_usize_skewed_dep_graph, Usize, write_leb128b_usize_skewed, DEP_GRAPH);
impl_bench!(write_leb128b_usize_skewed_query_cache, Usize, write_leb128b_usize_skewed, QUERY_CACHE);


impl_bench!(write_leb128a_u16_metadata, U16, write_leb128a_u16, METADATA);
impl_bench!(write_leb128a_u16_dep_graph, U16, write_leb128a_u16, DEP_GRAPH);
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
