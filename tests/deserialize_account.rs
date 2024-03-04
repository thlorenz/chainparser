use std::collections::{HashMap, HashSet};

use borsh::BorshSerialize;
use serde::{Deserialize, Serialize};
use solana_idl::{
    EnumFields, IdlEnumVariant, IdlType, IdlTypeDefinition, IdlTypeDefinitionTy,
};
use solana_sdk::pubkey::Pubkey;

mod utils;
pub use chainparser::{
    de::{
        i128_from_string, i64_from_string, opt_pubkey_from_base58,
        pubkey_from_base58, u128_from_string, u64_from_string,
        vec_pubkey_from_base58,
    },
    json::JsonSerializationOpts,
};

use crate::utils::{
    process_test_case_json, process_test_case_json_compare_str, to_if,
};

#[test]
fn deserialize_struct_with_floats() {
    let ty_name = "Floats";

    #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize)]
    pub struct Floats {
        pub float_32: f32,
        pub float_64: f64,
    }

    fn approx_equal(a: f64, b: f64, dp: u8) -> bool {
        let p = 10f64.powi(-(dp as i32));
        (a - b).abs() < p
    }

    impl PartialEq for Floats {
        fn eq(&self, other: &Self) -> bool {
            approx_equal(self.float_32 as f64, other.float_32 as f64, 1)
                && approx_equal(self.float_64, other.float_64, 1)
        }
    }
    impl Eq for Floats {}

    let idl_type_def = IdlTypeDefinition {
        name: ty_name.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("float_32", IdlType::F32),
                to_if("float_64", IdlType::F64),
            ],
        },
    };

    let t = "Positive Floats Case";
    {
        let instance = Floats {
            // Basically encoding the rounding error here.
            // We had 1.1 but then the JSON serializer turned it into 1.100000023841858 and
            // our test which compares strings failed
            #[allow(clippy::excessive_precision)]
            float_32: 1.100000023841858,
            float_64: 3.40282348e+38,
        };

        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance.clone(),
            ty_name,
            &mut writer,
            None,
            None,
        )
    }

    let t = "Negative Floats Case";
    {
        let instance = Floats {
            // Basically encoding the rounding error here.
            // We had 1.1 but then the JSON serializer turned it into 1.100000023841858 and
            // our test which compares strings failed
            #[allow(clippy::excessive_precision)]
            float_32: -1.100000023841858,
            float_64: -3.40282348e+38,
        };

        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance.clone(),
            ty_name,
            &mut writer,
            None,
            None,
        )
    }

    let t = "NAN Floats Case";
    {
        let expected = r#"{ "float_32": NaN,"float_64": NaN }"#
            .to_string()
            .split_whitespace()
            .collect::<String>();

        let mut writer = String::new();
        process_test_case_json_compare_str(
            t,
            &[&idl_type_def],
            ty_name,
            &mut writer,
            None,
            //  f32:NAN           f64:NAN
            vec![0, 0, 0x01, 0xFF, 0, 0, 0, 0, 0, 0, 0xF8, 0xFF],
            &expected,
        )
    }
}

#[test]
fn deserialize_struct_with_composites() {
    let ty_name = "Person";

    #[derive(
        Clone, Debug, Serialize, Deserialize, BorshSerialize, Eq, PartialEq,
    )]
    pub struct Person {
        pub name: String,
        pub age: u64,
        pub ns: Vec<i16>,
        pub map: HashMap<u8, String>,
        pub set: HashSet<String>,
        pub opt: Option<u64>,
        pub bytes: Vec<u8>,
        pub tuple: (u64, String, Option<u8>),
        pub vec_tuple: Vec<(u64, String)>,
        #[serde(deserialize_with = "pubkey_from_base58")]
        pub pubkey: Pubkey,
    }

    let idl_type_def = IdlTypeDefinition {
        name: ty_name.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("name", IdlType::String),
                to_if("age", IdlType::U64),
                to_if("ns", IdlType::Vec(Box::new(IdlType::I16))),
                to_if(
                    "map",
                    IdlType::HashMap(
                        Box::new(IdlType::U8),
                        Box::new(IdlType::String),
                    ),
                ),
                to_if("set", IdlType::HashSet(Box::new(IdlType::String))),
                to_if("opt", IdlType::Option(Box::new(IdlType::U64))),
                to_if("bytes", IdlType::Bytes),
                to_if(
                    "tuple",
                    IdlType::Tuple(vec![
                        IdlType::U64,
                        IdlType::String,
                        IdlType::Option(Box::new(IdlType::U8)),
                    ]),
                ),
                to_if(
                    "vec_tuple",
                    IdlType::Vec(Box::new(IdlType::Tuple(vec![
                        IdlType::U64,
                        IdlType::String,
                    ]))),
                ),
                to_if("pubkey", IdlType::PublicKey),
            ],
        },
    };

    let t = "Typical Case with all Composites Populated";
    {
        let instance = Person {
            name: "John".to_string(),
            age: 30,
            ns: vec![1, 2, -3],
            map: vec![(1, "foo".to_string()), (3, "bar".to_string())]
                .into_iter()
                .collect(),
            set: vec!["uno".to_string(), "dos".to_string()]
                .into_iter()
                .collect(),
            opt: Some(42),
            bytes: vec![1, 2, 3, 4, 5, 6, 7, 8],
            tuple: (42, "foo".to_string(), Some(3)),
            vec_tuple: vec![(1, "foo".to_string()), (3, "bar".to_string())],
            pubkey: Pubkey::new_unique(),
        };

        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance.clone(),
            ty_name,
            &mut writer,
            None,
            None,
        )
    }

    let t = "Empty Composites";
    {
        let instance = Person {
            name: "John".to_string(),
            age: 30,
            ns: vec![],
            map: HashMap::new(),
            set: HashSet::new(),
            opt: None,
            bytes: Vec::new(),
            tuple: (0, "".to_string(), None),
            vec_tuple: vec![],
            pubkey: Pubkey::default(),
        };
        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance.clone(),
            ty_name,
            &mut writer,
            None,
            None,
        )
    }
}

#[test]
fn deserialize_large_nums() {
    let ty_name = "Primitives";
    let idl_type_def = IdlTypeDefinition {
        name: ty_name.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("large_unsigned", IdlType::U64),
                to_if("large_signed", IdlType::I64),
                to_if("very_large_unsigned", IdlType::U128),
                to_if("very_large_signed", IdlType::I128),
            ],
        },
    };

    let t = "Default Opts";
    {
        #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
        pub struct Primitives {
            large_unsigned: u64,
            large_signed: i64,
            very_large_unsigned: u128,
            very_large_signed: i128,
        }
        let instance = Primitives {
            large_unsigned: u64::MAX,
            large_signed: i64::MIN,
            very_large_unsigned: u128::MAX,
            very_large_signed: i128::MIN,
        };
        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance.clone(),
            ty_name,
            &mut writer,
            None,
            None,
        );
    }

    // The below two only make a difference for JSON
    let t = "Opts to not stringify u64/i64";
    #[derive(Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
    pub struct Primitives {
        #[serde(deserialize_with = "u64_from_string")]
        large_unsigned: u64,
        #[serde(deserialize_with = "i64_from_string")]
        large_signed: i64,

        very_large_unsigned: u128,
        very_large_signed: i128,
    }
    let instance = Primitives {
        large_unsigned: u64::MAX,
        large_signed: i64::MIN,
        very_large_unsigned: u128::MAX,
        very_large_signed: i128::MIN,
    };
    {
        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance,
            ty_name,
            &mut writer,
            Some(JsonSerializationOpts {
                n64_as_string: true,
                ..Default::default()
            }),
            None,
        );
    }
}

#[test]
fn deserialize_pubkeys() {
    let ty_name = "Pubkeys";
    let idl_type_def = IdlTypeDefinition {
        name: ty_name.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("pubkey", IdlType::PublicKey),
                to_if("pubkey_vec", IdlType::Vec(Box::new(IdlType::PublicKey))),
                to_if(
                    "pubkey_opt",
                    IdlType::Option(Box::new(IdlType::PublicKey)),
                ),
            ],
        },
    };

    let t = "Opts to not stringify Pubkey";
    {
        #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
        pub struct Pubkeys {
            pubkey: Pubkey,
            pubkey_vec: Vec<Pubkey>,
            pubkey_opt: Option<Pubkey>,
        }
        let instance = Pubkeys {
            pubkey: Pubkey::new_unique(),
            pubkey_vec: vec![Pubkey::new_unique(), Pubkey::new_unique()],
            pubkey_opt: Some(Pubkey::new_unique()),
        };
        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance.clone(),
            ty_name,
            &mut writer,
            Some(JsonSerializationOpts {
                pubkey_as_base58: false,
                ..Default::default()
            }),
            None,
        );
    }

    let t = "Default opts";
    {
        #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
        pub struct Pubkeys {
            #[serde(deserialize_with = "pubkey_from_base58")]
            pubkey: Pubkey,
            #[serde(deserialize_with = "vec_pubkey_from_base58")]
            pubkey_vec: Vec<Pubkey>,
            #[serde(deserialize_with = "opt_pubkey_from_base58")]
            pubkey_opt: Option<Pubkey>,
        }
        let instance = Pubkeys {
            pubkey: Pubkey::new_unique(),
            pubkey_vec: vec![Pubkey::new_unique(), Pubkey::new_unique()],
            pubkey_opt: Some(Pubkey::new_unique()),
        };
        let mut writer = String::new();
        process_test_case_json(
            t,
            &[&idl_type_def],
            instance.clone(),
            ty_name,
            &mut writer,
            None,
            None,
        );
    }
}

#[test]
fn deserialize_nested_types() {
    // -----------------
    // Types and Definitions
    // -----------------

    // TypeUno
    let ty_uno = "TypeUno";
    #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
    pub struct TypeUno {
        key: String,
        value: u64,
    }
    let itd_uno = IdlTypeDefinition {
        name: ty_uno.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("key", IdlType::String),
                to_if("value", IdlType::U64),
            ],
        },
    };

    // TypeDos
    let ty_dos = "TypeDos";
    #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
    pub struct TypeDos {
        key: u8,
        value: String,
    }
    let itd_dos = IdlTypeDefinition {
        name: ty_dos.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("key", IdlType::U8),
                to_if("value", IdlType::String),
            ],
        },
    };

    // NestOneLevelSimple
    let ty_nest_one_level_simple = "NestOneLevelSimple";
    #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
    pub struct NestOneLevelSimple {
        key: String,
        uno: TypeUno,
        dos: TypeDos,
    }
    let itd_nest_one_level_simple = IdlTypeDefinition {
        name: ty_nest_one_level_simple.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("key", IdlType::String),
                to_if("uno", IdlType::Defined(ty_uno.to_string())),
                to_if("dos", IdlType::Defined(ty_dos.to_string())),
            ],
        },
    };

    // NestOneLevelComposite
    let ty_nest_one_level_composite = "NestOneLevelComposite";
    #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
    pub struct NestOneLevelComposite {
        key: String,
        uno: HashMap<u8, TypeUno>,
        dos: Vec<TypeDos>,
    }
    let itd_nest_one_level_composite = IdlTypeDefinition {
        name: ty_nest_one_level_composite.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("key", IdlType::String),
                to_if(
                    "uno",
                    IdlType::HashMap(
                        Box::new(IdlType::U8),
                        Box::new(IdlType::Defined(ty_uno.to_string())),
                    ),
                ),
                to_if(
                    "dos",
                    IdlType::Vec(Box::new(IdlType::Defined(
                        ty_dos.to_string(),
                    ))),
                ),
            ],
        },
    };

    // Nest Two Levels
    let ty_nest_two_levels = "NestTwoLevels";
    #[derive(Clone, Debug, Deserialize, BorshSerialize, Eq, PartialEq)]
    pub struct NestTwoLevels {
        key: String,
        simple: NestOneLevelSimple,
        composite: NestOneLevelComposite,
    }
    let itd_nest_two_levels = IdlTypeDefinition {
        name: ty_nest_two_levels.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("key", IdlType::String),
                to_if(
                    "simple",
                    IdlType::Defined(ty_nest_one_level_simple.to_string()),
                ),
                to_if(
                    "composite",
                    IdlType::Defined(ty_nest_one_level_composite.to_string()),
                ),
            ],
        },
    };

    let idl_type_defs = [
        &itd_uno,
        &itd_dos,
        &itd_nest_one_level_simple,
        &itd_nest_one_level_composite,
        &itd_nest_two_levels,
    ];

    // -----------------
    // Tests
    // -----------------
    let t = "NestOneLevelSimple";
    {
        let instance = NestOneLevelSimple {
            key: "simple".to_string(),
            uno: TypeUno {
                key: "uno".to_string(),
                value: 1,
            },
            dos: TypeDos {
                key: 2,
                value: "dos".to_string(),
            },
        };

        let mut writer = String::new();
        process_test_case_json(
            t,
            &idl_type_defs,
            instance.clone(),
            ty_nest_one_level_simple,
            &mut writer,
            None,
            None,
        );
    }
    let t = "NestOneLevelComposite";
    {
        let instance = NestOneLevelComposite {
            key: "composite:key".to_string(),
            uno: vec![
                (
                    1,
                    TypeUno {
                        key: "uno:1".to_string(),
                        value: 1,
                    },
                ),
                (
                    2,
                    TypeUno {
                        key: "uno:2".to_string(),
                        value: 2,
                    },
                ),
            ]
            .into_iter()
            .collect(),
            dos: vec![
                TypeDos {
                    key: 1,
                    value: "dos:1".to_string(),
                },
                TypeDos {
                    key: 2,
                    value: "dos:2".to_string(),
                },
            ],
        };

        let mut writer = String::new();
        process_test_case_json(
            t,
            &idl_type_defs,
            instance.clone(),
            ty_nest_one_level_composite,
            &mut writer,
            None,
            None,
        );
    }

    let t = "NestTwoLevels";
    {
        let instance = NestTwoLevels {
            key: "nest:two:levels".to_string(),
            simple: NestOneLevelSimple {
                key: "simple".to_string(),
                uno: TypeUno {
                    key: "uno".to_string(),
                    value: 1,
                },
                dos: TypeDos {
                    key: 2,
                    value: "dos".to_string(),
                },
            },
            composite: NestOneLevelComposite {
                key: "composite:key".to_string(),
                uno: vec![
                    (
                        1,
                        TypeUno {
                            key: "uno:1".to_string(),
                            value: 1,
                        },
                    ),
                    (
                        2,
                        TypeUno {
                            key: "uno:2".to_string(),
                            value: 2,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                dos: vec![
                    TypeDos {
                        key: 1,
                        value: "dos:1".to_string(),
                    },
                    TypeDos {
                        key: 2,
                        value: "dos:2".to_string(),
                    },
                ],
            },
        };

        let mut writer = String::new();
        process_test_case_json(
            t,
            &idl_type_defs,
            instance.clone(),
            ty_nest_two_levels,
            &mut writer,
            None,
            None,
        );
    }
}

#[test]
fn deserialize_mixed_enum() {
    let ty_mixed_enum = "MixedEnum";
    #[derive(
        Clone, Debug, Serialize, Deserialize, BorshSerialize, Eq, PartialEq,
    )]
    enum MixedEnum {
        Scalar,
        NamedFields { uno: u8, dos: u8 },
        UnnamedFields(u8, HashMap<u8, String>),
    }

    let itd_mixed_enum = IdlTypeDefinition {
        name: ty_mixed_enum.to_string(),
        ty: IdlTypeDefinitionTy::Enum {
            variants: vec![
                IdlEnumVariant {
                    name: "Scalar".to_string(),
                    fields: None,
                },
                IdlEnumVariant {
                    name: "NamedFields".to_string(),
                    fields: Some(EnumFields::Named(vec![
                        to_if("uno", IdlType::U8),
                        to_if("dos", IdlType::U8),
                    ])),
                },
                IdlEnumVariant {
                    name: "UnnamedFields".to_string(),
                    fields: Some(EnumFields::Tuple(vec![
                        IdlType::U8,
                        IdlType::HashMap(
                            Box::new(IdlType::U8),
                            Box::new(IdlType::String),
                        ),
                    ])),
                },
            ],
        },
    };

    let ty_has_mixed_enums = "HasMixedEnums";
    #[derive(
        Clone, Debug, Serialize, Deserialize, BorshSerialize, Eq, PartialEq,
    )]
    struct HasMixedEnums {
        key: String,
        scalar: MixedEnum,
        named_fields: MixedEnum,
        unnamed_fields: MixedEnum,
    }

    let itd_has_mixed_enums = IdlTypeDefinition {
        name: ty_has_mixed_enums.to_string(),
        ty: IdlTypeDefinitionTy::Struct {
            fields: vec![
                to_if("key", IdlType::String),
                to_if("scalar", IdlType::Defined(ty_mixed_enum.to_string())),
                to_if(
                    "named_fields",
                    IdlType::Defined(ty_mixed_enum.to_string()),
                ),
                to_if(
                    "unnamed_fields",
                    IdlType::Defined(ty_mixed_enum.to_string()),
                ),
            ],
        },
    };

    let instance = HasMixedEnums {
        key: "has:mixed:enums".to_string(),
        scalar: MixedEnum::Scalar,
        named_fields: MixedEnum::NamedFields { uno: 1, dos: 2 },
        unnamed_fields: MixedEnum::UnnamedFields(
            3,
            vec![(4, "four".to_string())].into_iter().collect(),
        ),
    };
    let idl_type_defs = [&itd_mixed_enum, &itd_has_mixed_enums];

    let t = "ScalarEnum";
    let mut writer = String::new();
    process_test_case_json(
        t,
        &idl_type_defs,
        instance.clone(),
        ty_has_mixed_enums,
        &mut writer,
        None,
        None,
    );
}
