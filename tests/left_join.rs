use restq::{
    ast::{ddl::*, dml::*, *},
    *,
};

#[test]
fn multiple_left_joins() {
    let input = "bazaar.review<-bazaar.product_review<-bazaar.product{bazaar.review.review_id}?bazaar.product.product_id=eq.'3c03c6f0-7d91-4570-a882-0ef44c427b90'&page=1&page_size=40";
    println!("{}", input);
    let input_chars = to_chars(input);
    let ret = select().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
    let mut table_lookup = TableLookup::new();

    let product = TableDef {
        table: TableName {
            name: "bazaar.product".into(),
        },
        columns: vec![
            ColumnDef {
                column: ColumnName {
                    name: "organization_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "client_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "created".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Utc,
                    is_optional: false,
                    default: Some(DefaultValue::Function(Function {
                        name: "now".into(),
                        params: vec![],
                    })),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "created_by".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "updated".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Utc,
                    is_optional: false,
                    default: Some(DefaultValue::Function(Function {
                        name: "now".into(),
                        params: vec![],
                    })),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "updated_by".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "priority".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::F64,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "name".into(),
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
                    name: "description".into(),
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
                    name: "help".into(),
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
                    name: "active".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Bool,
                    is_optional: false,
                    default: Some(DefaultValue::DataValue(DataValue::Bool(
                        true,
                    ))),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "product_id".into(),
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: false,
                    default: Some(DefaultValue::Function(Function {
                        name: "uuid_generate_v4".into(),
                        params: vec![],
                    })),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "parent_product_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "is_service".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Bool,
                    is_optional: true,
                    default: Some(DefaultValue::DataValue(DataValue::Bool(
                        false,
                    ))),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "price".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::F64,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "use_parent_price".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Bool,
                    is_optional: true,
                    default: Some(DefaultValue::DataValue(DataValue::Bool(
                        false,
                    ))),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "unit".into(),
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
                    name: "tags".into(),
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
                    name: "info".into(),
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
                    name: "seq_no".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::I32,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "upfront_fee".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::F64,
                    is_optional: true,
                    default: Some(DefaultValue::DataValue(DataValue::F64(0.0))),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "barcode".into(),
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
                    name: "owner_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "bazaar.users".into(),
                    },
                    column: Some(ColumnName {
                        name: "user_id".into(),
                    }),
                }),
            },
            ColumnDef {
                column: ColumnName {
                    name: "currency_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "payment.currency".into(),
                    },
                    column: Some(ColumnName {
                        name: "currency_id".into(),
                    }),
                }),
            },
        ],
    };

    let product_review = TableDef {
        table: TableName {
            name: "bazaar.product_review".into(),
        },
        columns: vec![
            ColumnDef {
                column: ColumnName {
                    name: "organization_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "client_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "created".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Utc,
                    is_optional: false,
                    default: Some(DefaultValue::Function(Function {
                        name: "now".into(),
                        params: vec![],
                    })),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "created_by".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "updated".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Utc,
                    is_optional: false,
                    default: Some(DefaultValue::Function(Function {
                        name: "now".into(),
                        params: vec![],
                    })),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "updated_by".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "priority".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::F64,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "product_id".into(),
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: false,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "bazaar.product".into(),
                    },
                    column: Some(ColumnName {
                        name: "product_id".into(),
                    }),
                }),
            },
            ColumnDef {
                column: ColumnName {
                    name: "review_id".into(),
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: false,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "bazaar.review".into(),
                    },
                    column: Some(ColumnName {
                        name: "review_id".into(),
                    }),
                }),
            },
        ],
    };

    let review = TableDef {
        table: TableName {
            name: "bazaar.review".into(),
        },
        columns: vec![
            ColumnDef {
                column: ColumnName {
                    name: "organization_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "client_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "created".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Utc,
                    is_optional: false,
                    default: Some(DefaultValue::Function(Function {
                        name: "now".into(),
                        params: vec![],
                    })),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "created_by".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "updated".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Utc,
                    is_optional: false,
                    default: Some(DefaultValue::Function(Function {
                        name: "now".into(),
                        params: vec![],
                    })),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "updated_by".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "priority".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::F64,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "name".into(),
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
                    name: "description".into(),
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
                    name: "help".into(),
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
                    name: "active".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Bool,
                    is_optional: false,
                    default: Some(DefaultValue::DataValue(DataValue::Bool(
                        true,
                    ))),
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "rating".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::I32,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "comment".into(),
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
                    name: "review_id".into(),
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: false,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "user_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "bazaar.users".into(),
                    },
                    column: Some(ColumnName {
                        name: "user_id".into(),
                    }),
                }),
            },
            ColumnDef {
                column: ColumnName {
                    name: "approved".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Bool,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "approvedby".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::Uuid,
                    is_optional: true,
                    default: None,
                },
                foreign: None,
            },
        ],
    };
    table_lookup.add_table(product);
    table_lookup.add_table(product_review);
    table_lookup.add_table(review);

    let sql = ret
        .into_sql_statement(Some(&table_lookup))
        .expect("must not error");
    println!("sql: {}", sql.to_string());
    assert_eq!(input, ret.to_string());
    assert_eq!("SELECT bazaar.review.review_id FROM bazaar.review LEFT JOIN bazaar.product_review ON bazaar.product_review.review_id = bazaar.review.review_id LEFT JOIN bazaar.product ON bazaar.product_review.product_id = bazaar.product.product_id WHERE bazaar.product.product_id = '3c03c6f0-7d91-4570-a882-0ef44c427b90' LIMIT 40 OFFSET 0", sql.to_string());
}
