use super::*;
use crate::{
    ast::{
        ddl::{
            ColumnAttribute,
            ColumnDef,
            DataTypeDef,
            Foreign,
            TableDef,
        },
        ColumnName,
        TableName,
    },
    DataType,
};

#[test]
fn test_column_name_with_table_dot() {
    let input = to_chars("product.name");
    let ret = column().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ColumnName {
            name: "product.name".to_string()
        }
    );
}

#[test]
fn test_schema_table_column_name() {
    let input = to_chars("public.product.name");
    let ret = column().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ColumnName {
            name: "public.product.name".to_string()
        }
    );
}

#[test]
fn test_column_name_with_sapce() {
    let input = to_chars("\"zip code\"");
    let ret = column().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ColumnName {
            name: "zip code".to_string()
        }
    );
}

#[test]
#[should_panic]
fn test_expect_table() {
    let input = to_chars("123123");
    let ret = select().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
}

#[test]
fn test_select_with_no_selection() {
    let input = to_chars("product");
    let ret = select().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Select {
            from_table: FromTable {
                from: TableName {
                    name: "product".into()
                },
                join: None,
            },
            ..Default::default()
        }
    );
}

#[test]
fn test_right_join_table() {
    let input = to_chars("product->product_old");
    let ret = select().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Select {
            from_table: FromTable {
                from: TableName {
                    name: "product".into()
                },
                join: Some((
                    JoinType::RightJoin,
                    Box::new(FromTable {
                        from: TableName {
                            name: "product_old".into()
                        },
                        join: None,
                    })
                )),
            },
            ..Default::default()
        }
    );
}

#[test]
#[should_panic]
fn test_expect_valid_table_after_join() {
    let input = to_chars("product->123213");
    let ret = select().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
}

#[test]
fn test_description() {
    let input = to_chars("description");
    let ret = column().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ColumnName {
            name: "description".into()
        },
    )
}

#[test]
fn test_ascension() {
    let input = to_chars("ascension");
    let ret = column().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ColumnName {
            name: "ascension".into()
        },
    )
}

#[test]
fn test_expr_rename() {
    let input = to_chars("column1=>new_column");
    let ret = expr_rename().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ExprRename {
            expr: Expr::Column(ColumnName {
                name: "column1".into()
            }),
            rename: Some("new_column".to_string())
        }
    )
}
#[test]
fn test_expr_with_renames() {
    let input = to_chars("column1=>new_column,column2,column3");
    let ret = exprs_with_renames().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        vec![
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column1".into()
                }),
                rename: Some("new_column".to_string())
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column2".into()
                }),
                rename: None,
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column3".into()
                }),
                rename: None,
            },
        ]
    )
}
#[test]
fn test_expr_with_renames2() {
    let input = to_chars("column1,column2,column3");
    let ret = exprs_with_renames().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        vec![
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column1".into()
                }),
                rename: None,
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column2".into()
                }),
                rename: None,
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column3".into()
                }),
                rename: None,
            },
        ]
    )
}

#[test]
fn test_expr_list() {
    let input = to_chars("{column1,column2,column3}");
    let ret = expr_projection().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        vec![
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column1".into()
                }),
                rename: None,
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column2".into()
                }),
                rename: None,
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "column3".into()
                }),
                rename: None,
            },
        ]
    )
}
#[test]
fn test_expr_no_rename() {
    let input = to_chars("column1");
    let ret = expr_rename().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ExprRename {
            expr: Expr::Column(ColumnName {
                name: "column1".into()
            }),
            rename: None,
        }
    )
}

#[test]
#[should_panic]
fn test_fail_strict_ident() {
    let input = to_chars("order_by");
    let _ret = strict_ident().parse(&input).expect("must be parsed");
}

#[test]
#[should_panic]
fn test_fail_strict_ident2() {
    let input = to_chars("group_by");
    let _ret = strict_ident().parse(&input).expect("must be parsed");
}

#[test]
fn test_strict_ident() {
    let input = to_chars("column1");
    let ret = strict_ident().parse(&input).expect("must be parsed");
    assert_eq!(ret, "column1");
}
#[test]
fn test_strict_column() {
    let input = to_chars("column1");
    let ret = column().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        ColumnName {
            name: "column1".into()
        }
    );
}
#[test]
#[should_panic]
fn test_strict_column_fail() {
    let input = to_chars("group_by");
    let _ret = column().parse(&input).expect("must be parsed");
}

#[test]
fn test_simple_select_to_sql() {
    let input = to_chars("person{name,age,class}");
    let ret = select().parse(&input).expect("must be parsed");
    let select = ret.into_sql_select(None).expect("must not fail");
    println!("{}", select);
    assert_eq!(select.to_string(), "SELECT name, age, class FROM person");
}

#[test]
fn test_table_query_with_range_only() {
    let input = to_chars("person&page=1&page_size=20");
    let ret = select().parse(&input).expect("must be parsed");
    let select = ret.into_sql_query(None).expect("must not fail");
    println!("{}", select);
    assert_eq!(
        select.to_string(),
        "SELECT * FROM person \
         LIMIT 20 OFFSET 0"
    );
}

#[test]
fn test_complex_query() {
    let input = to_chars("person{name,age,class}?(age=gt.42&student=eq.true)|(gender=eq.'M'&is_active=true)&group_by=sum(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10");
    let ret = select().parse(&input).expect("must be parsed");
    let select = ret.into_sql_query(None).expect("must not fail");
    println!("{}", select);
    assert_eq!(
        select.to_string(),
        "SELECT name, age, class FROM person \
         WHERE \
         (age > 42 AND student = true) OR (gender = 'M' AND is_active = true) \
         GROUP BY sum(age), grade, gender \
         HAVING min(age) >= 42 ORDER BY age DESC, height ASC \
         LIMIT 10 OFFSET 10"
    );
}
#[test]
fn test_complex_query_with_join() {
    let input = to_chars("person->users{name,age,class}?(age=gt.42&student=eq.true)|(gender=eq.'M'&is_active=true)&group_by=sum(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10");
    let ret = select().parse(&input).expect("must be parsed");
    let person_table = TableDef {
        table: TableName {
            name: "person".into(),
        },
        columns: vec![ColumnDef {
            column: ColumnName { name: "id".into() },
            attributes: Some(vec![ColumnAttribute::Primary]),
            data_type_def: DataTypeDef {
                data_type: DataType::S64,
                is_optional: false,
                default: None,
            },
            foreign: None,
        }],
    };
    let users_table = TableDef {
        table: TableName {
            name: "users".into(),
        },
        columns: vec![ColumnDef {
            column: ColumnName {
                name: "person_id".into(),
            },
            attributes: None,
            data_type_def: DataTypeDef {
                data_type: DataType::U64,
                is_optional: false,
                default: None,
            },
            foreign: Some(Foreign {
                table: TableName {
                    name: "person".into(),
                },
                column: None,
            }),
        }],
    };
    let mut table_lookup = TableLookup::new(); //no table lookup
    table_lookup.add_table(person_table);
    table_lookup.add_table(users_table);

    let select = ret
        .into_sql_query(Some(&table_lookup))
        .expect("must not fail");

    println!("ret: {}", select);

    assert_eq!(
        select.to_string(),
        "SELECT name, age, class FROM person \
         RIGHT JOIN users ON users.person_id = person.id \
         WHERE \
         (age > 42 AND student = true) OR (gender = 'M' AND is_active = true) \
         GROUP BY sum(age), grade, gender \
         HAVING min(age) >= 42 ORDER BY age DESC, height ASC \
         LIMIT 10 OFFSET 10"
    );
}

#[test]
fn test_complex_query_with_multiple_join() {
    let input = to_chars("person->users<-student{name,age,class}?(age=gt.42&student=eq.true)|(gender=eq.'M'&is_active=true)&group_by=sum(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10");
    let ret = select().parse(&input).expect("must be parsed");
    let person_table = TableDef {
        table: TableName {
            name: "person".into(),
        },
        columns: vec![ColumnDef {
            column: ColumnName { name: "id".into() },
            attributes: Some(vec![ColumnAttribute::Primary]),
            data_type_def: DataTypeDef {
                data_type: DataType::S64,
                is_optional: false,
                default: None,
            },
            foreign: None,
        }],
    };
    let users_table = TableDef {
        table: TableName {
            name: "users".into(),
        },
        columns: vec![
            ColumnDef {
                column: ColumnName {
                    name: "user_id".into(),
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::S64,
                    is_optional: false,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "person_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::U64,
                    is_optional: false,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "person".into(),
                    },
                    column: None,
                }),
            },
        ],
    };
    let student_table = TableDef {
        table: TableName {
            name: "student".into(),
        },
        columns: vec![ColumnDef {
            column: ColumnName {
                name: "user_id".into(),
            },
            attributes: None,
            data_type_def: DataTypeDef {
                data_type: DataType::U64,
                is_optional: false,
                default: None,
            },
            foreign: Some(Foreign {
                table: TableName {
                    name: "users".into(),
                },
                column: None,
            }),
        }],
    };
    let mut table_lookup = TableLookup::new(); //no table lookup
    table_lookup.add_table(person_table);
    table_lookup.add_table(users_table);
    table_lookup.add_table(student_table);

    let select = ret
        .into_sql_query(Some(&table_lookup))
        .expect("must not fail");

    println!("ret: {}", select);

    assert_eq!(
        select.to_string(),
        "SELECT name, age, class FROM person \
         RIGHT JOIN users ON users.person_id = person.id \
         LEFT JOIN student ON student.user_id = users.user_id \
         WHERE \
         (age > 42 AND student = true) OR (gender = 'M' AND is_active = true) \
         GROUP BY sum(age), grade, gender \
         HAVING min(age) >= 42 ORDER BY age DESC, height ASC \
         LIMIT 10 OFFSET 10"
    );
}

#[test]
fn test_complex_query_with_multiple_join_full_join() {
    let input = to_chars("person->users<-->student{name,age,class}?(age=gt.42&student=eq.true)|(gender=eq.'M'&is_active=true)&group_by=sum(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10");
    let ret = select().parse(&input).expect("must be parsed");
    let person_table = TableDef {
        table: TableName {
            name: "person".into(),
        },
        columns: vec![ColumnDef {
            column: ColumnName { name: "id".into() },
            attributes: Some(vec![ColumnAttribute::Primary]),
            data_type_def: DataTypeDef {
                data_type: DataType::S64,
                is_optional: false,
                default: None,
            },
            foreign: None,
        }],
    };
    let users_table = TableDef {
        table: TableName {
            name: "users".into(),
        },
        columns: vec![
            ColumnDef {
                column: ColumnName {
                    name: "user_id".into(),
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::S64,
                    is_optional: false,
                    default: None,
                },
                foreign: None,
            },
            ColumnDef {
                column: ColumnName {
                    name: "person_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::U64,
                    is_optional: false,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "person".into(),
                    },
                    column: None,
                }),
            },
        ],
    };
    let student_table = TableDef {
        table: TableName {
            name: "student".into(),
        },
        columns: vec![ColumnDef {
            column: ColumnName {
                name: "user_id".into(),
            },
            attributes: None,
            data_type_def: DataTypeDef {
                data_type: DataType::U64,
                is_optional: false,
                default: None,
            },
            foreign: Some(Foreign {
                table: TableName {
                    name: "users".into(),
                },
                column: None,
            }),
        }],
    };
    let mut table_lookup = TableLookup::new(); //no table lookup
    table_lookup.add_table(person_table);
    table_lookup.add_table(users_table);
    table_lookup.add_table(student_table);

    let select = ret
        .into_sql_query(Some(&table_lookup))
        .expect("must not fail");

    println!("ret: {}", select);

    assert_eq!(
        select.to_string(),
        "SELECT name, age, class FROM person \
         RIGHT JOIN users ON users.person_id = person.id \
         FULL JOIN student ON student.user_id = users.user_id \
         WHERE \
         (age > 42 AND student = true) OR (gender = 'M' AND is_active = true) \
         GROUP BY sum(age), grade, gender \
         HAVING min(age) >= 42 ORDER BY age DESC, height ASC \
         LIMIT 10 OFFSET 10"
    );
}

#[test]
fn test_simple_select() {
    let input = to_chars("person->users{name,age,class}?(age=gt.42&student=eq.true)|(gender=eq.M&is_active=true)&group_by=sum(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10");
    let ret = select().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Select {
            from_table: FromTable {
                from: TableName {
                    name: "person".into()
                },
                join: Some((
                    JoinType::RightJoin,
                    Box::new(FromTable {
                        from: TableName {
                            name: "users".into()
                        },
                        join: None,
                    }),
                ),),
            },
            filter: Some(Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::BinaryOperation(Box::new(
                            BinaryOperation {
                                left: Expr::Column(ColumnName {
                                    name: "age".into()
                                },),
                                operator: Operator::Gt,
                                right: Expr::Value(Value::Number(42.0,),),
                            }
                        ),),
                        operator: Operator::And,
                        right: Expr::BinaryOperation(Box::new(
                            BinaryOperation {
                                left: Expr::Column(ColumnName {
                                    name: "student".into()
                                },),
                                operator: Operator::Eq,
                                right: Expr::Value(Value::Bool(true,),),
                            }
                        ),),
                    }
                ),))),
                operator: Operator::Or,
                right: Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::BinaryOperation(Box::new(
                            BinaryOperation {
                                left: Expr::Column(ColumnName {
                                    name: "gender".into()
                                },),
                                operator: Operator::Eq,
                                right: Expr::Column(ColumnName {
                                    name: "M".into()
                                },),
                            }
                        ),),
                        operator: Operator::And,
                        right: Expr::BinaryOperation(Box::new(
                            BinaryOperation {
                                left: Expr::Column(ColumnName {
                                    name: "is_active".into()
                                },),
                                operator: Operator::Eq,
                                right: Expr::Value(Value::Bool(true,),),
                            }
                        ),),
                    }
                ),))),
            }),),),
            group_by: Some(vec![
                Expr::Function(Function {
                    name: "sum".into(),
                    params: vec![Expr::Column(ColumnName {
                        name: "age".into()
                    },),],
                },),
                Expr::Column(ColumnName {
                    name: "grade".into()
                },),
                Expr::Column(ColumnName {
                    name: "gender".into()
                },),
            ],),
            having: Some(Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Function(Function {
                    name: "min".into(),
                    params: vec![Expr::Column(ColumnName {
                        name: "age".into()
                    },),],
                },),
                operator: Operator::Gte,
                right: Expr::Value(Value::Number(42.0,),),
            }),),),
            projection: Some(vec![
                ExprRename {
                    expr: Expr::Column(ColumnName {
                        name: "name".into()
                    },),
                    rename: None,
                },
                ExprRename {
                    expr: Expr::Column(ColumnName { name: "age".into() },),
                    rename: None,
                },
                ExprRename {
                    expr: Expr::Column(ColumnName {
                        name: "class".into()
                    },),
                    rename: None,
                },
            ],),
            order_by: Some(vec![
                Order {
                    expr: Expr::Column(ColumnName { name: "age".into() },),
                    direction: Some(Direction::Desc,),
                },
                Order {
                    expr: Expr::Column(ColumnName {
                        name: "height".into()
                    },),
                    direction: Some(Direction::Asc,),
                },
            ],),
            range: Some(Range::Page(Page {
                page: 2,
                page_size: 10,
            },),),
        }
    );
}

#[test]
fn test_expr_selection() {
    let input =
        to_chars("{name,description,age,max(height)=>new_height,class}");
    let ret = expr_projection().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        vec![
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "name".into()
                }),
                rename: None,
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "description".into()
                }),
                rename: None,
            },
            ExprRename {
                expr: Expr::Column(ColumnName { name: "age".into() }),
                rename: None
            },
            ExprRename {
                expr: Expr::Function(Function {
                    name: "max".into(),
                    params: vec![Expr::Column(ColumnName {
                        name: "height".into()
                    })]
                }),
                rename: Some("new_height".to_string()),
            },
            ExprRename {
                expr: Expr::Column(ColumnName {
                    name: "class".into()
                }),
                rename: None,
            }
        ]
    )
}

#[test]
fn test_order() {
    let input = to_chars("score.desc");
    let ret = order().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Order {
            expr: Expr::Column(ColumnName {
                name: "score".to_string()
            }),
            direction: Some(Direction::Desc)
        }
    );
}
#[test]
fn test_column_with_order() {
    let input = to_chars("score.desc");
    let ret = column().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ColumnName {
            name: "score".into()
        }
    )
}

#[test]
fn test_column_with_not_order() {
    let input = to_chars("score.something");
    let ret = column().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        ColumnName {
            name: "score.something".into()
        }
    )
}

#[test]
fn test_range_using_page() {
    let input = to_chars("page=2&page_size=10");
    let ret = range().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Range::Page(Page {
            page: 2,
            page_size: 10,
        })
    )
}

#[test]
fn test_range_using_limit() {
    let input = to_chars("limit=10&offset=20");
    let ret = range().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Range::Limit(Limit {
            limit: 10,
            offset: Some(20),
        })
    )
}

#[test]
fn test_page_with_size() {
    let input = to_chars("page=2&page_size=10");
    let ret = page().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Page {
            page: 2,
            page_size: 10,
        }
    )
}
#[test]
#[should_panic]
fn test_page_invalid() {
    let input = to_chars("page=xx&page_size=10");
    let ret = page().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Page {
            page: 2,
            page_size: 10,
        }
    )
}
#[test]
#[should_panic]
fn test_page_with_size_invalid() {
    let input = to_chars("page=2&page_size=zy");
    let ret = page().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Page {
            page: 2,
            page_size: 10,
        }
    )
}

#[test]
fn test_limit() {
    let input = to_chars("limit=10");
    let ret = limit().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Limit {
            limit: 10,
            offset: None,
        }
    )
}

#[test]
fn test_limit_with_offset() {
    let input = to_chars("limit=10&offset=20");
    let ret = limit().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Limit {
            limit: 10,
            offset: Some(20),
        }
    )
}

#[test]
fn test_from_right_join() {
    let input = to_chars("product->users");
    let ret = from_table().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        FromTable {
            from: TableName {
                name: "product".into()
            },
            join: Some((
                JoinType::RightJoin,
                Box::new(FromTable {
                    from: TableName {
                        name: "users".into()
                    },
                    join: None,
                }),
            ),),
        }
    );
}

#[test]
fn test_from_left_join() {
    let input = to_chars("product<-users");
    let ret = from_table().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        FromTable {
            from: TableName {
                name: "product".into()
            },
            join: Some((
                JoinType::LeftJoin,
                Box::new(FromTable {
                    from: TableName {
                        name: "users".into()
                    },
                    join: None,
                }),
            ),),
        }
    );
}

#[test]
fn test_from_inner_join() {
    let input = to_chars("product-><-users");
    let ret = from_table().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        FromTable {
            from: TableName {
                name: "product".into()
            },
            join: Some((
                JoinType::InnerJoin,
                Box::new(FromTable {
                    from: TableName {
                        name: "users".into()
                    },
                    join: None,
                }),
            ),),
        }
    );
}

#[test]
fn test_from_full_join() {
    let input = to_chars("product<-->users");
    let ret = from_table().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        FromTable {
            from: TableName {
                name: "product".into()
            },
            join: Some((
                JoinType::FullJoin,
                Box::new(FromTable {
                    from: TableName {
                        name: "users".into()
                    },
                    join: None,
                }),
            ),),
        }
    );
}

#[test]
fn test_from_table() {
    let input = to_chars("product->users<-customer");
    let ret = from_table().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        FromTable {
            from: TableName {
                name: "product".into()
            },
            join: Some((
                JoinType::RightJoin,
                Box::new(FromTable {
                    from: TableName {
                        name: "users".into()
                    },
                    join: Some((
                        JoinType::LeftJoin,
                        Box::new(FromTable {
                            from: TableName {
                                name: "customer".into()
                            },
                            join: None,
                        }),
                    ),),
                }),
            ),),
        }
    );
}

#[test]
fn test_more_complex_filter2() {
    let input =
        to_chars("(age=gt.42&is_active=true)|(gender=eq.'M'&class='Human')");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                BinaryOperation {
                    left: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName { name: "age".into() }),
                        operator: Operator::Gt,
                        right: Expr::Value(Value::Number(42.0))
                    })),
                    operator: Operator::And,
                    right: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName {
                            name: "is_active".into()
                        }),
                        operator: Operator::Eq,
                        right: Expr::Value(Value::Bool(true))
                    }))
                }
            )))),
            operator: Operator::Or,
            right: Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                BinaryOperation {
                    left: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName {
                            name: "gender".into()
                        }),
                        operator: Operator::Eq,
                        right: Expr::Value(Value::String("M".into()))
                    },)),
                    operator: Operator::And,
                    right: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName {
                            name: "class".into()
                        }),
                        operator: Operator::Eq,
                        right: Expr::Value(Value::String("Human".into()))
                    },)),
                }
            ))))
        }))
    );
}

#[test]
fn test_complex_filter_with_many_parens() {
    let input = to_chars("(((age=gt.42&is_active=true)))|((gender=eq.'M'))");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Nested(Box::new(Expr::Nested(Box::new(Expr::Nested(
                Box::new(Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName { name: "age".into() }),
                        operator: Operator::Gt,
                        right: Expr::Value(Value::Number(42.0))
                    })),
                    operator: Operator::And,
                    right: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName {
                            name: "is_active".into()
                        }),
                        operator: Operator::Eq,
                        right: Expr::Value(Value::Bool(true))
                    }))
                })))
            ))))),
            operator: Operator::Or,
            right: Expr::Nested(Box::new(Expr::Nested(Box::new(
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName {
                        name: "gender".into()
                    }),
                    operator: Operator::Eq,
                    right: Expr::Value(Value::String("M".into()))
                },))
            ))))
        }))
    );
}

#[test]
fn test_complex_filter1() {
    let input = to_chars("(age=gt.42&is_active=true)|gender=eq.'M'");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                BinaryOperation {
                    left: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName { name: "age".into() }),
                        operator: Operator::Gt,
                        right: Expr::Value(Value::Number(42.0))
                    })),
                    operator: Operator::And,
                    right: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName {
                            name: "is_active".into()
                        }),
                        operator: Operator::Eq,
                        right: Expr::Value(Value::Bool(true))
                    }))
                }
            )))),
            operator: Operator::Or,
            right: Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Column(ColumnName {
                    name: "gender".into()
                }),
                operator: Operator::Eq,
                right: Expr::Value(Value::String("M".into()))
            },))
        }))
    );
}

#[test]
fn test_grouped_filter() {
    let input = to_chars("(gender=eq.'M'&class='Human')");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    println!("ret: {:#?}", ret);
    assert_eq!(
        ret,
        Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
            BinaryOperation {
                left: Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName {
                        name: "gender".into()
                    }),
                    operator: Operator::Eq,
                    right: Expr::Value(Value::String("M".into()))
                })),
                operator: Operator::And,
                right: Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName {
                        name: "class".into()
                    }),
                    operator: Operator::Eq,
                    right: Expr::Value(Value::String("Human".into()))
                }))
            }
        ))))
    );
}

#[test]
fn test_filter_simple_filter_with_group_by() {
    let input = to_chars("(age=gt.42&is_active=true)&group_by=age");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    println!("{:#?}", ret);
    assert_eq!(
        ret,
        Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
            BinaryOperation {
                left: Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName { name: "age".into() }),
                    operator: Operator::Gt,
                    right: Expr::Value(Value::Number(42.0))
                })),
                operator: Operator::And,
                right: Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName {
                        name: "is_active".into()
                    }),
                    operator: Operator::Eq,
                    right: Expr::Value(Value::Bool(true))
                }))
            }
        ))))
    );
}

#[test]
fn test_filter_simple_filter_and() {
    let input = to_chars("age=gt.42&is_active=true");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Column(ColumnName { name: "age".into() }),
                operator: Operator::Gt,
                right: Expr::Value(Value::Number(42.0))
            })),
            operator: Operator::And,
            right: Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Column(ColumnName {
                    name: "is_active".into()
                }),
                operator: Operator::Eq,
                right: Expr::Value(Value::Bool(true))
            }))
        }))
    );
}

#[test]
fn test_filter_simple_filter_one() {
    let input = to_chars("age=gt.42");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Column(ColumnName { name: "age".into() }),
            operator: Operator::Gt,
            right: Expr::Value(Value::Number(42.0))
        })),
    );
}
#[test]
fn test_filter_simple_filter_one_grouped() {
    let input = to_chars("(age=gt.42)");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
            BinaryOperation {
                left: Expr::Column(ColumnName { name: "age".into() }),
                operator: Operator::Gt,
                right: Expr::Value(Value::Number(42.0))
            }
        )))),
    );
}

#[test]
fn test_filter_simple_filter_2_grouped() {
    let input = to_chars("(age=gt.42)|(is_active=true)");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                BinaryOperation {
                    left: Expr::Column(ColumnName { name: "age".into() }),
                    operator: Operator::Gt,
                    right: Expr::Value(Value::Number(42.0))
                }
            )))),
            operator: Operator::Or,
            right: Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                BinaryOperation {
                    left: Expr::Column(ColumnName {
                        name: "is_active".into()
                    }),
                    operator: Operator::Eq,
                    right: Expr::Value(Value::Bool(true))
                }
            ))))
        }))
    );
}

#[test]
fn test_filter_simple_filter_or() {
    let input = to_chars("age=gt.42|is_active=true");
    let ret = filter_expr().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Column(ColumnName { name: "age".into() }),
                operator: Operator::Gt,
                right: Expr::Value(Value::Number(42.0))
            })),
            operator: Operator::Or,
            right: Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Column(ColumnName {
                    name: "is_active".into()
                }),
                operator: Operator::Eq,
                right: Expr::Value(Value::Bool(true))
            }))
        }))
    );
}

#[test]
fn test_condition_gt() {
    let input = to_chars("age=gt.42");
    let ret = binary_operation_expr()
        .parse(&input)
        .expect("must be parsed");
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Column(ColumnName { name: "age".into() }),
            operator: Operator::Gt,
            right: Expr::Value(Value::Number(42.0))
        }))
    );
}
#[test]
fn test_condition_lte() {
    let input = to_chars("age=lte.42");
    let ret = binary_operation_expr()
        .parse(&input)
        .expect("must be parsed");
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Column(ColumnName { name: "age".into() }),
            operator: Operator::Lte,
            right: Expr::Value(Value::Number(42.0))
        }))
    );
}

#[test]
fn test_condition_default_eq() {
    let input = to_chars("age=42");
    let ret = binary_operation_expr()
        .parse(&input)
        .expect("must be parsed");
    assert_eq!(
        ret,
        Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Column(ColumnName { name: "age".into() }),
            operator: Operator::Eq,
            right: Expr::Value(Value::Number(42.0))
        }))
    );
}

#[test]
fn test_function() {
    let input = to_chars("max(seq_no)");
    let ret = function().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Function {
            name: "max".into(),
            params: vec![Expr::Column(ColumnName {
                name: "seq_no".into()
            })]
        }
    );
}

#[test]
fn test_function_expr() {
    let input = to_chars("max(seq_no)");
    let ret = expr().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        Expr::Function(Function {
            name: "max".into(),
            params: vec![Expr::Column(ColumnName {
                name: "seq_no".into()
            })]
        })
    );
}

#[test]
fn test_column() {
    let input = to_chars("product_id");
    let ret = column().parse(&input).expect("must be parsed");
    assert_eq!(
        ret,
        ColumnName {
            name: "product_id".into()
        }
    );
}

#[test]
fn test_value_bool() {
    let input = to_chars("true");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::Bool(true));
}
#[test]
fn test_value_bool2() {
    let input = to_chars("false");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::Bool(false));
}
#[test]
fn test_value_number() {
    let input = to_chars("0.1312312");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::Number(0.1312312));
}
#[test]
fn test_value_number2() {
    let input = to_chars("3.14159");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::Number(3.14159));
}

#[test]
fn test_string() {
    let input = to_chars("product_id");
    let ret = string().parse(&input).expect("must be parsed");
    assert_eq!(ret, "product_id");
}

#[test]
fn test_iregular_string() {
    let input = to_chars("a string value\"pr'oduct_id");
    let ret = string().parse(&input).expect("must be parsed");
    assert_eq!(ret, "a string value\"pr\'oduct_id");
}

#[test]
fn test_bool_expr() {
    let input = to_chars("true");
    let ret = expr().parse(&input).expect("must be parsed");
    assert_eq!(ret, Expr::Value(Value::Bool(true)));
}
#[test]
fn test_null_expr() {
    let input = to_chars("null");
    let ret = expr().parse(&input).expect("must be parsed");
    assert_eq!(ret, Expr::Value(Value::Null));
}

#[test]
fn test_value_value() {
    let input = to_chars("91.56");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::Number(91.56));
}

#[test]
fn test_value_string() {
    let input = to_chars("fox");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::String("fox".to_string()));
}

#[test]
fn test_value_quoted_string() {
    let input = to_chars("\"from\"");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::String("from".to_string()));
}

#[test]
fn test_value_single_quoted_string() {
    let input = to_chars("'from'");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::String("from".to_string()));
}

#[test]
fn test_value_single_quoted_string_with_apostrophe() {
    let input = to_chars(r#"'from ivanceras\'s'"#);
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::String("from ivanceras's".to_string()));
}

#[test]
fn test_value_quoted_string2() {
    let input = to_chars("\"group_by\"");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::String("group_by".to_string()));
}

#[test]
fn test_value_quoted_string_with_inner_quote() {
    let input = to_chars(r#""group \"by""#);
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::String(r#"group "by"#.to_string()));
}

#[test]
fn test_value_single_quoted_string2() {
    let input = to_chars("'group_by'");
    let ret = value().parse(&input).expect("must be parsed");
    assert_eq!(ret, Value::String("group_by".to_string()));
}

#[test]
#[should_panic]
fn test_value_restricted_ident() {
    let input = to_chars("from");
    let _ret = value().parse(&input).expect("must be parsed");
}
