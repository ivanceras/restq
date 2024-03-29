use restq::{
    ast::{ddl::*, dml::*, *},
    *,
};

#[test]
fn more_complex_table_def() {
    let input = r#"adempiere.ad_element{*ad_element_id:f64,ad_client_id:f64,ad_org_id:f64,isactive:text('Y'),created:local,createdby:f64,updated:local(now()),updatedby:f64,columnname:text,entitytype(adempiere.ad_entitytype):text('D'),name:text,printname:text,description:text?,help:text?,po_name:text?,po_printname:text?,po_description:text?,po_help:text?,ad_reference_id(adempiere.ad_reference):f64?,fieldlength:f64?,ad_reference_value_id(adempiere.ad_reference):f64?,uuid:text?(NULL)}"#;
    let input_chars = to_chars(input);
    let ret = table_def().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
}

#[test]
fn complex_table_def() {
    let input = r#"public.film{*film_id:s32,title:text,description:text?,release_year:s16?,language_id(public.language::language_id):s16,original_language_id(public.language):s16?,rental_duration:s16(3),rental_rate:f64(4.99),length:s16?,replacement_cost:f64(19.99),rating:text?("'G'::mpaa_rating"),last_update:local,special_features:text?,fulltext:text}"#;
    let input_chars = to_chars(input);
    let ret = table_def().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
    assert_eq!(
        ret,
        TableDef {
            table: TableName {
                name: "public.film".to_string(),
            },
            columns: vec![
                ColumnDef {
                    column: ColumnName {
                        name: "film_id".to_string()
                    },
                    attributes: Some(vec![ColumnAttribute::Primary,],),
                    data_type_def: DataTypeDef {
                        data_type: DataType::S32,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "title".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "description".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: true,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "release_year".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::S16,
                        is_optional: true,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "language_id".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::S16,
                        is_optional: false,
                        default: None,
                    },
                    foreign: Some(Foreign {
                        table: TableName {
                            name: "public.language".to_string(),
                        },
                        column: Some(ColumnName {
                            name: "language_id".to_string()
                        })
                    },),
                },
                ColumnDef {
                    column: ColumnName {
                        name: "original_language_id".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::S16,
                        is_optional: true,
                        default: None,
                    },
                    foreign: Some(Foreign {
                        table: TableName {
                            name: "public.language".to_string(),
                        },
                        column: None
                    },),
                },
                ColumnDef {
                    column: ColumnName {
                        name: "rental_duration".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::S16,
                        is_optional: false,
                        default: Some(DefaultValue::DataValue(DataValue::S16(
                            3,
                        ))),
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "rental_rate".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::F64,
                        is_optional: false,
                        default: Some(DefaultValue::DataValue(DataValue::F64(
                            4.99,
                        ))),
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "length".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::S16,
                        is_optional: true,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "replacement_cost".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::F64,
                        is_optional: false,
                        default: Some(DataValue::F64(19.99,).into()),
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "rating".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: true,
                        default: Some(
                            DataValue::Text("\'G\'::mpaa_rating".to_string(),)
                                .into()
                        ),
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "last_update".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Local,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "special_features".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: true,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "fulltext".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
            ],
        }
    )
}

#[test]
fn table_def_with_default_function() {
    let input = r#"public.film{*film_id:uuid(uuid_generate_v4()),title:text,release_date:utc?(now())}"#;
    let input_chars = to_chars(input);
    let ret = table_def().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
    assert_eq!(
        ret,
        TableDef {
            table: TableName {
                name: "public.film".to_string(),
            },
            columns: vec![
                ColumnDef {
                    column: ColumnName {
                        name: "film_id".to_string()
                    },
                    attributes: Some(vec![ColumnAttribute::Primary,],),
                    data_type_def: DataTypeDef {
                        data_type: DataType::Uuid,
                        is_optional: false,
                        default: Some(
                            Function {
                                name: "uuid_generate_v4".into(),
                                params: vec![],
                            }
                            .into()
                        ),
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "title".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "release_date".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Utc,
                        is_optional: true,
                        default: Some(
                            Function {
                                name: "now".to_string(),
                                params: vec![]
                            }
                            .into()
                        ),
                    },
                    foreign: None,
                },
            ],
        }
    )
}

#[test]
fn table_def_has_column_with_space() {
    let input = r#"public.film{*film_id:uuid(uuid_generate_v4()),"zip code":text,title:text,release_date:utc?(now())}"#;
    let input_chars = to_chars(input);
    let ret = table_def().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
    assert_eq!(
        ret,
        TableDef {
            table: TableName {
                name: "public.film".to_string(),
            },
            columns: vec![
                ColumnDef {
                    column: ColumnName {
                        name: "film_id".to_string()
                    },
                    attributes: Some(vec![ColumnAttribute::Primary,],),
                    data_type_def: DataTypeDef {
                        data_type: DataType::Uuid,
                        is_optional: false,
                        default: Some(
                            Function {
                                name: "uuid_generate_v4".into(),
                                params: vec![],
                            }
                            .into()
                        ),
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "zip code".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "title".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: ColumnName {
                        name: "release_date".to_string(),
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Utc,
                        is_optional: true,
                        default: Some(
                            Function {
                                name: "now".to_string(),
                                params: vec![]
                            }
                            .into()
                        ),
                    },
                    foreign: None,
                },
            ],
        }
    )
}
