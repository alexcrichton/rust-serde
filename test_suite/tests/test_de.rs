#![cfg_attr(feature = "unstable", feature(into_boxed_c_str))]

#[macro_use]
extern crate serde_derive;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::net;
use std::path::PathBuf;
use std::time::Duration;
use std::default::Default;
use std::ffi::{CString, OsString};

#[cfg(feature = "unstable")]
use std::ffi::CStr;

extern crate serde;
use serde::Deserialize;

extern crate fnv;
use self::fnv::FnvHasher;

extern crate serde_test;
use self::serde_test::{
    Error,
    Token,
    assert_de_tokens,
    assert_de_tokens_error,
};

#[macro_use]
mod macros;

//////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct UnitStruct;

#[derive(PartialEq, Debug, Deserialize)]
struct TupleStruct(i32, i32, i32);

#[derive(PartialEq, Debug, Deserialize)]
struct Struct {
    a: i32,
    b: i32,
    #[serde(skip_deserializing)]
    c: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknown {
    a: i32,
    #[serde(skip_deserializing)]
    b: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(default)]
struct StructDefault<T> {
    a: i32,
    b: T,
}

impl Default for StructDefault<String> {
    fn default() -> Self {
        StructDefault {
            a: 100,
            b: "default".to_string(),
        }
    }
}

#[derive(PartialEq, Debug, Deserialize)]
struct StructSkipAll {
    #[serde(skip_deserializing)]
    a: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructSkipAllDenyUnknown {
    #[serde(skip_deserializing)]
    a: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
enum Enum {
    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Skipped,
    Unit,
    Simple(i32),
    Seq(i32, i32, i32),
    Map { a: i32, b: i32, c: i32 },
}

#[derive(PartialEq, Debug, Deserialize)]
enum EnumSkipAll {
    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Skipped,
}

//////////////////////////////////////////////////////////////////////////

macro_rules! declare_test {
    ($name:ident { $($value:expr => $tokens:expr,)+ }) => {
        #[test]
        fn $name() {
            $(
                // Test ser/de roundtripping
                assert_de_tokens(&$value, $tokens);

                // Test that the tokens are ignorable
                assert_de_tokens_ignore($tokens);
            )+
        }
    }
}

macro_rules! declare_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            declare_test!($name { $($value => $tokens,)+ });
        )+
    }
}

macro_rules! declare_error_tests {
    ($($name:ident<$target:ty> { $tokens:expr, $expected:expr, })+) => {
        $(
            #[test]
            fn $name() {
                assert_de_tokens_error::<$target>($tokens, $expected);
            }
        )+
    }
}

fn assert_de_tokens_ignore(ignorable_tokens: &[Token<'static>]) {
    #[derive(PartialEq, Debug, Deserialize)]
    struct IgnoreBase {
        a: i32,
    }

    let expected = IgnoreBase{a: 1};

    // Embed the tokens to be ignored in the normal token
    // stream for an IgnoreBase type
    let concated_tokens : Vec<Token<'static>> = vec![
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("ignored")
        ]
        .into_iter()
        .chain(ignorable_tokens.to_vec().into_iter())
        .chain(vec![
            Token::MapEnd,
        ].into_iter())
        .collect();

    let mut de = serde_test::Deserializer::new(concated_tokens.into_iter());
    let v: Result<IgnoreBase, Error> = Deserialize::deserialize(&mut de);

    // We run this test on every token stream for convenience, but
    // some token streams don't make sense embedded as a map value,
    // so we ignore those. SyntaxError is the real sign of trouble.
    if let Err(Error::UnexpectedToken(_)) = v {
        return;
    }

    assert_eq!(v.as_ref(), Ok(&expected));
    assert_eq!(de.next_token(), None);
}

//////////////////////////////////////////////////////////////////////////

declare_tests! {
    test_bool {
        true => &[Token::Bool(true)],
        false => &[Token::Bool(false)],
    }
    test_isize {
        0isize => &[Token::I8(0)],
        0isize => &[Token::I16(0)],
        0isize => &[Token::I32(0)],
        0isize => &[Token::I64(0)],
        0isize => &[Token::U8(0)],
        0isize => &[Token::U16(0)],
        0isize => &[Token::U32(0)],
        0isize => &[Token::U64(0)],
        0isize => &[Token::F32(0.)],
        0isize => &[Token::F64(0.)],
    }
    test_ints {
        0i8 => &[Token::I8(0)],
        0i16 => &[Token::I16(0)],
        0i32 => &[Token::I32(0)],
        0i64 => &[Token::I64(0)],
    }
    test_uints {
        0u8 => &[Token::U8(0)],
        0u16 => &[Token::U16(0)],
        0u32 => &[Token::U32(0)],
        0u64 => &[Token::U64(0)],
    }
    test_floats {
        0f32 => &[Token::F32(0.)],
        0f64 => &[Token::F64(0.)],
    }
    test_char {
        'a' => &[Token::Char('a')],
        'a' => &[Token::Str("a")],
        'a' => &[Token::String("a".to_owned())],
    }
    test_string {
        "abc".to_owned() => &[Token::Str("abc")],
        "abc".to_owned() => &[Token::String("abc".to_owned())],
        "a".to_owned() => &[Token::Char('a')],
    }
    test_option {
        None::<i32> => &[Token::Unit],
        None::<i32> => &[Token::Option(false)],
        Some(1) => &[Token::I32(1)],
        Some(1) => &[
            Token::Option(true),
            Token::I32(1),
        ],
    }
    test_result {
        Ok::<i32, i32>(0) => &[
            Token::EnumStart("Result"),
            Token::Str("Ok"),
            Token::I32(0),
        ],
        Err::<i32, i32>(1) => &[
            Token::EnumStart("Result"),
            Token::Str("Err"),
            Token::I32(1),
        ],
    }
    test_unit {
        () => &[Token::Unit],
        () => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        () => &[
            Token::SeqStart(None),
            Token::SeqEnd,
        ],
        () => &[
            Token::TupleStructStart("Anything", 0),
            Token::TupleStructEnd,
        ],
    }
    test_unit_struct {
        UnitStruct => &[Token::Unit],
        UnitStruct => &[
            Token::UnitStruct("UnitStruct"),
        ],
        UnitStruct => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        UnitStruct => &[
            Token::SeqStart(None),
            Token::SeqEnd,
        ],
    }
    test_unit_string {
        String::new() => &[Token::Unit],
    }
    test_tuple_struct {
        TupleStruct(1, 2, 3) => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::SeqStart(None),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::TupleStructStart("TupleStruct", 3),
                Token::TupleStructSep,
                Token::I32(1),

                Token::TupleStructSep,
                Token::I32(2),

                Token::TupleStructSep,
                Token::I32(3),
            Token::TupleStructEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::TupleStructStart("TupleStruct", 3),
                Token::TupleStructSep,
                Token::I32(1),

                Token::TupleStructSep,
                Token::I32(2),

                Token::TupleStructSep,
                Token::I32(3),
            Token::TupleStructEnd,
        ],
    }
    test_btreeset {
        BTreeSet::<isize>::new() => &[
            Token::Unit,
        ],
        BTreeSet::<isize>::new() => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        btreeset![btreeset![], btreeset![1], btreeset![2, 3]] => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::SeqStart(Some(0)),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(2)),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        BTreeSet::<isize>::new() => &[
            Token::UnitStruct("Anything"),
        ],
        BTreeSet::<isize>::new() => &[
            Token::TupleStructStart("Anything", 0),
            Token::TupleStructEnd,
        ],
    }
    test_hashset {
        HashSet::<isize>::new() => &[
            Token::Unit,
        ],
        HashSet::<isize>::new() => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        hashset![1, 2, 3] => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        HashSet::<isize>::new() => &[
            Token::UnitStruct("Anything"),
        ],
        HashSet::<isize>::new() => &[
            Token::TupleStructStart("Anything", 0),
            Token::TupleStructEnd,
        ],
        hashset![FnvHasher @ 1, 2, 3] => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => &[
            Token::Unit,
        ],
        Vec::<isize>::new() => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::SeqStart(Some(0)),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(2)),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        Vec::<isize>::new() => &[
            Token::UnitStruct("Anything"),
        ],
        Vec::<isize>::new() => &[
            Token::TupleStructStart("Anything", 0),
            Token::TupleStructEnd,
        ],
    }
    test_array {
        [0; 0] => &[
            Token::Unit,
        ],
        [0; 0] => &[
            Token::SeqStart(Some(0)),
            Token::SeqEnd,
        ],
        [0; 0] => &[
            Token::SeqArrayStart(0),
            Token::SeqEnd,
        ],
        ([0; 0], [1], [2, 3]) => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::SeqStart(Some(0)),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(1)),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqStart(Some(2)),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        ([0; 0], [1], [2, 3]) => &[
            Token::SeqArrayStart(3),
                Token::SeqSep,
                Token::SeqArrayStart(0),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqArrayStart(1),
                    Token::SeqSep,
                    Token::I32(1),
                Token::SeqEnd,

                Token::SeqSep,
                Token::SeqArrayStart(2),
                    Token::SeqSep,
                    Token::I32(2),

                    Token::SeqSep,
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        [0; 0] => &[
            Token::UnitStruct("Anything"),
        ],
        [0; 0] => &[
            Token::TupleStructStart("Anything", 0),
            Token::TupleStructEnd,
        ],
    }
    test_tuple {
        (1,) => &[
            Token::SeqStart(Some(1)),
                Token::SeqSep,
                Token::I32(1),
            Token::SeqEnd,
        ],
        (1, 2, 3) => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),

                Token::SeqSep,
                Token::I32(3),
            Token::SeqEnd,
        ],
        (1,) => &[
            Token::TupleStart(1),
                Token::TupleSep,
                Token::I32(1),
            Token::TupleEnd,
        ],
        (1, 2, 3) => &[
            Token::TupleStart(3),
                Token::TupleSep,
                Token::I32(1),

                Token::TupleSep,
                Token::I32(2),

                Token::TupleSep,
                Token::I32(3),
            Token::TupleEnd,
        ],
    }
    test_btreemap {
        BTreeMap::<isize, isize>::new() => &[
            Token::Unit,
        ],
        BTreeMap::<isize, isize>::new() => &[
            Token::MapStart(Some(0)),
            Token::MapEnd,
        ],
        btreemap![1 => 2] => &[
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::MapStart(Some(0)),
                Token::MapEnd,

                Token::MapSep,
                Token::I32(2),
                Token::MapStart(Some(2)),
                    Token::MapSep,
                    Token::I32(3),
                    Token::I32(4),

                    Token::MapSep,
                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        BTreeMap::<isize, isize>::new() => &[
            Token::UnitStruct("Anything"),
        ],
        BTreeMap::<isize, isize>::new() => &[
            Token::StructStart("Anything", 0),
            Token::StructEnd,
        ],
    }
    test_hashmap {
        HashMap::<isize, isize>::new() => &[
            Token::Unit,
        ],
        HashMap::<isize, isize>::new() => &[
            Token::MapStart(Some(0)),
            Token::MapEnd,
        ],
        hashmap![1 => 2] => &[
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        hashmap![1 => 2, 3 => 4] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        hashmap![1 => hashmap![], 2 => hashmap![3 => 4, 5 => 6]] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::MapStart(Some(0)),
                Token::MapEnd,

                Token::MapSep,
                Token::I32(2),
                Token::MapStart(Some(2)),
                    Token::MapSep,
                    Token::I32(3),
                    Token::I32(4),

                    Token::MapSep,
                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        HashMap::<isize, isize>::new() => &[
            Token::UnitStruct("Anything"),
        ],
        HashMap::<isize, isize>::new() => &[
            Token::StructStart("Anything", 0),
            Token::StructEnd,
        ],
        hashmap![FnvHasher @ 1 => 2, 3 => 4] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::I32(1),
                Token::I32(2),

                Token::MapSep,
                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::MapStart(Some(3)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::StructStart("Struct", 3),
                Token::StructSep,
                Token::Str("a"),
                Token::I32(1),

                Token::StructSep,
                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::SeqStart(Some(3)),
                Token::SeqSep,
                Token::I32(1),

                Token::SeqSep,
                Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_struct_with_skip {
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::MapStart(Some(3)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("b"),
                Token::I32(2),

                Token::MapSep,
                Token::Str("c"),
                Token::I32(3),

                Token::MapSep,
                Token::Str("d"),
                Token::I32(4),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::StructStart("Struct", 3),
                Token::StructSep,
                Token::Str("a"),
                Token::I32(1),

                Token::StructSep,
                Token::Str("b"),
                Token::I32(2),

                Token::StructSep,
                Token::Str("c"),
                Token::I32(3),

                Token::StructSep,
                Token::Str("d"),
                Token::I32(4),
            Token::StructEnd,
        ],
    }
    test_struct_skip_all {
        StructSkipAll { a: 0 } => &[
            Token::StructStart("StructSkipAll", 0),
            Token::StructEnd,
        ],
        StructSkipAll { a: 0 } => &[
            Token::StructStart("StructSkipAll", 1),
                Token::StructSep,
                Token::Str("a"),
                Token::I32(1),

                Token::StructSep,
                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
    }
    test_struct_skip_all_deny_unknown {
        StructSkipAllDenyUnknown { a: 0 } => &[
            Token::StructStart("StructSkipAllDenyUnknown", 0),
            Token::StructEnd,
        ],
    }
    test_struct_default {
        StructDefault { a: 50, b: "overwritten".to_string() } => &[
            Token::StructStart("StructDefault", 1),
                Token::StructSep,
                Token::Str("a"),
                Token::I32(50),

                Token::StructSep,
                Token::Str("b"),
                Token::String("overwritten".to_string()),
            Token::StructEnd,
        ],
        StructDefault { a: 100, b: "default".to_string() } => &[
            Token::StructStart("StructDefault", 0),
            Token::StructEnd,
        ],
    }
    test_enum_unit {
        Enum::Unit => &[
            Token::EnumUnit("Enum", "Unit"),
        ],
    }
    test_enum_simple {
        Enum::Simple(1) => &[
            Token::EnumNewType("Enum", "Simple"),
            Token::I32(1),
        ],
    }
    test_enum_seq {
        Enum::Seq(1, 2, 3) => &[
            Token::EnumSeqStart("Enum", "Seq", 3),
                Token::EnumSeqSep,
                Token::I32(1),

                Token::EnumSeqSep,
                Token::I32(2),

                Token::EnumSeqSep,
                Token::I32(3),
            Token::EnumSeqEnd,
        ],
    }
    test_enum_map {
        Enum::Map { a: 1, b: 2, c: 3 } => &[
            Token::EnumMapStart("Enum", "Map", 3),
                Token::EnumMapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::EnumMapSep,
                Token::Str("b"),
                Token::I32(2),

                Token::EnumMapSep,
                Token::Str("c"),
                Token::I32(3),
            Token::EnumMapEnd,
        ],
    }
    test_enum_unit_usize {
        Enum::Unit => &[
            Token::EnumStart("Enum"),
            Token::U32(0),
            Token::Unit,
        ],
    }
    test_enum_unit_bytes {
        Enum::Unit => &[
            Token::EnumStart("Enum"),
            Token::Bytes(b"Unit"),
            Token::Unit,
        ],
    }
    test_box {
        Box::new(0i32) => &[Token::I32(0)],
    }
    test_boxed_slice {
        Box::new([0, 1, 2]) => &[
            Token::SeqStart(Some(3)),
            Token::SeqSep,
            Token::I32(0),
            Token::SeqSep,
            Token::I32(1),
            Token::SeqSep,
            Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_duration {
        Duration::new(1, 2) => &[
            Token::StructStart("Duration", 2),
                Token::StructSep,
                Token::Str("secs"),
                Token::U64(1),

                Token::StructSep,
                Token::Str("nanos"),
                Token::U32(2),
            Token::StructEnd,
        ],
        Duration::new(1, 2) => &[
            Token::SeqStart(Some(2)),
                Token::SeqSep,
                Token::I64(1),

                Token::SeqSep,
                Token::I64(2),
            Token::SeqEnd,
        ],
    }
    test_range {
        1u32..2u32 => &[
            Token::StructStart("Range", 2),
                Token::StructSep,
                Token::Str("start"),
                Token::U32(1),

                Token::StructSep,
                Token::Str("end"),
                Token::U32(2),
            Token::StructEnd,
        ],
        1u32..2u32 => &[
            Token::SeqStart(Some(2)),
                Token::SeqSep,
                Token::U64(1),

                Token::SeqSep,
                Token::U64(2),
            Token::SeqEnd,
        ],
    }
    test_net_ipv4addr {
        "1.2.3.4".parse::<net::Ipv4Addr>().unwrap() => &[Token::Str("1.2.3.4")],
    }
    test_net_ipv6addr {
        "::1".parse::<net::Ipv6Addr>().unwrap() => &[Token::Str("::1")],
    }
    test_net_socketaddr {
        "1.2.3.4:1234".parse::<net::SocketAddr>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "1.2.3.4:1234".parse::<net::SocketAddrV4>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "[::1]:1234".parse::<net::SocketAddrV6>().unwrap() => &[Token::Str("[::1]:1234")],
    }
    test_path_buf {
        PathBuf::from("/usr/local/lib") => &[
            Token::String("/usr/local/lib".to_owned()),
        ],
    }
    test_cstring {
        CString::new("abc").unwrap() => &[
            Token::Bytes(b"abc"),
        ],
    }
    test_osstring {
        OsString::from("abc") => &[
            Token::String("abc".to_owned()),
        ],
        {
            #[cfg(unix)]
            fn funky() -> Option<OsString> {
                use std::os::unix::ffi::OsStringExt;
                Some(OsString::from_vec(vec![1, 2, 3]))
            }
            #[cfg(windows)]
            fn funky() -> Option<OsString> {
                None
            }
            match funky() {
                Some(s) => s,
                None => return,
            }
        } => &[
            Token::Bytes(b"\x01\x02\x03"),
        ],
        {
            #[cfg(windows)]
            fn funky() -> Option<OsString> {
                use std::os::widnows::ffi::OsStringExt;
                Some(OsString::from_wide(&[1, 2, 3]))
            }
            #[cfg(unix)]
            fn funky() -> Option<OsString> {
                None
            }
            match funky() {
                Some(s) => s,
                None => return,
            }
        } => &[
            Token::SeqStart(Some(2)),
                Token::SeqSep,
                Token::U16(1),

                Token::SeqSep,
                Token::U16(2),

                Token::SeqSep,
                Token::U16(3),
            Token::SeqEnd,
        ],
    }
}

#[cfg(feature = "unstable")]
#[test]
fn test_cstr() {
    assert_de_tokens::<Box<CStr>>(&CString::new("abc").unwrap().into_boxed_c_str(),
                                  &[Token::Bytes(b"abc")]);
}

#[cfg(feature = "unstable")]
#[test]
fn test_net_ipaddr() {
    assert_de_tokens(
        &"1.2.3.4".parse::<net::IpAddr>().unwrap(),
        &[Token::Str("1.2.3.4")],
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_cstr_internal_null() {
    assert_de_tokens_error::<Box<CStr>>(
        &[
            Token::Bytes(b"a\0c"),
        ],
        Error::Message("nul byte found in provided data at position: 1".into())
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_cstr_internal_null_end() {
    assert_de_tokens_error::<Box<CStr>>(
        &[
            Token::Bytes(b"ac\0"),
        ],
        Error::Message("nul byte found in provided data at position: 2".into())
    );
}

declare_error_tests! {
    test_unknown_field<StructDenyUnknown> {
        &[
            Token::StructStart("StructDenyUnknown", 2),
                Token::StructSep,
                Token::Str("a"),
                Token::I32(0),

                Token::StructSep,
                Token::Str("d"),
        ],
        Error::Message("unknown field `d`, expected `a`".to_owned()),
    }
    test_skipped_field_is_unknown<StructDenyUnknown> {
        &[
            Token::StructStart("StructDenyUnknown", 2),
                Token::StructSep,
                Token::Str("b"),
        ],
        Error::Message("unknown field `b`, expected `a`".to_owned()),
    }
    test_skip_all_deny_unknown<StructSkipAllDenyUnknown> {
        &[
            Token::StructStart("StructSkipAllDenyUnknown", 1),
                Token::StructSep,
                Token::Str("a"),
        ],
        Error::Message("unknown field `a`, there are no fields".to_owned()),
    }
    test_unknown_variant<Enum> {
        &[
            Token::EnumUnit("Enum", "Foo"),
        ],
        Error::Message("unknown variant `Foo`, expected one of `Unit`, `Simple`, `Seq`, `Map`".to_owned()),
    }
    test_enum_skipped_variant<Enum> {
        &[
            Token::EnumUnit("Enum", "Skipped"),
        ],
        Error::Message("unknown variant `Skipped`, expected one of `Unit`, `Simple`, `Seq`, `Map`".to_owned()),
    }
    test_enum_skip_all<EnumSkipAll> {
        &[
            Token::EnumUnit("EnumSkipAll", "Skipped"),
        ],
        Error::Message("unknown variant `Skipped`, there are no variants".to_owned()),
    }
    test_struct_seq_too_long<Struct> {
        &[
            Token::SeqStart(Some(4)),
                Token::SeqSep, Token::I32(1),
                Token::SeqSep, Token::I32(2),
                Token::SeqSep, Token::I32(3),
        ],
        Error::UnexpectedToken(Token::SeqSep),
    }
    test_duplicate_field_struct<Struct> {
        &[
            Token::MapStart(Some(3)),
                Token::MapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::MapSep,
                Token::Str("a"),
        ],
        Error::Message("duplicate field `a`".to_owned()),
    }
    test_duplicate_field_enum<Enum> {
        &[
            Token::EnumMapStart("Enum", "Map", 3),
                Token::EnumMapSep,
                Token::Str("a"),
                Token::I32(1),

                Token::EnumMapSep,
                Token::Str("a"),
        ],
        Error::Message("duplicate field `a`".to_owned()),
    }
    test_enum_out_of_range<Enum> {
        &[
            Token::EnumStart("Enum"),
            Token::U32(4),
            Token::Unit,
        ],
        Error::Message("invalid value: integer `4`, expected variant index 0 <= i < 4".into()),
    }
    test_short_tuple<(u8, u8, u8)> {
        &[
            Token::TupleStart(1),
            Token::TupleSep,
            Token::U8(1),
            Token::TupleEnd,
        ],
        Error::Message("invalid length 1, expected a tuple of size 3".into()),
    }
    test_short_array<[u8; 3]> {
        &[
            Token::SeqStart(Some(1)),
            Token::SeqSep,
            Token::U8(1),
            Token::SeqEnd,
        ],
        Error::Message("invalid length 1, expected an array of length 3".into()),
    }
    test_cstring_internal_null<CString> {
        &[
            Token::Bytes(b"a\0c"),
        ],
        Error::Message("nul byte found in provided data at position: 1".into()),
    }
    test_cstring_internal_null_end<CString> {
        &[
            Token::Bytes(b"ac\0"),
        ],
        Error::Message("nul byte found in provided data at position: 2".into()),
    }
}
