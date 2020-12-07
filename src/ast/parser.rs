use crate::ast::*;
use pom::parser::{
    call,
    is_a,
    one_of,
    sym,
    tag,
    Parser,
};
use std::iter::FromIterator;
pub use utils::list_fail;
use utils::*;

pub mod utils;

/// a valid identifier
pub(crate) fn ident<'a>() -> Parser<'a, char, String> {
    (is_a(alpha_or_underscore) + is_a(alphanum_or_underscore).repeat(0..))
        .map(|(ch1, rest_ch)| format!("{}{}", ch1, String::from_iter(rest_ch)))
}

pub(crate) fn table_name<'a>() -> Parser<'a, char, String> {
    (strict_ident() - sym('.') + strict_ident())
        .map(|(table, column)| format!("{}.{}", table, column))
        | strict_ident()
}

fn restricted_ident<'a>() -> Parser<'a, char, &'a str> {
    tag("from")
        | tag("group_by")
        | tag("having")
        | tag("order_by")
        | tag("limit")
        | tag("asc")
        | tag("desc")
        | tag("page")
        | tag("page_size")
}

pub(crate) fn strict_ident<'a>() -> Parser<'a, char, String> {
    !(restricted_ident() - (end_or_ln() | one_of(",&=").map(|_| ()))) * ident()
}

/// column name can not be followed with direction: asc, desc
fn column_name<'a>() -> Parser<'a, char, String> {
    (strict_ident() - sym('.') + strict_ident())
        .map(|(table, column)| format!("{}.{}", table, column))
        | strict_ident()
}

pub(crate) fn column<'a>() -> Parser<'a, char, ColumnName> {
    column_name().map(|name| ColumnName { name })
}

pub(crate) fn table<'a>() -> Parser<'a, char, TableName> {
    table_name().map(|name| TableName { name })
}

fn bool<'a>() -> Parser<'a, char, bool> {
    tag("true").map(|_| true) | tag("false").map(|_| false)
}

fn null<'a>() -> Parser<'a, char, Value> {
    tag("null").map(|_| Value::Null)
}

pub(crate) fn value<'a>() -> Parser<'a, char, Value> {
    null()
        | bool().map(|v| Value::Bool(v))
        | number().map(|n| Value::Number(n))
        | quoted_string().map(|v| Value::String(v))
        | single_quoted_string().map(|v| Value::String(v))
        | back_quoted_string().map(|v| Value::String(v))
        | (!restricted_ident() * string()).map(|s| Value::String(s))
}

pub(crate) fn multi_values<'a>() -> Parser<'a, char, Vec<Value>> {
    sym('[') * list_fail(value(), sym(',')) - sym(']')
}

fn connector<'a>() -> Parser<'a, char, Operator> {
    tag("|").map(|_| Operator::Or) | tag("&").map(|_| Operator::And)
}

fn math_operator<'a>() -> Parser<'a, char, Operator> {
    tag("+").map(|_| Operator::Plus)
        | tag("-").map(|_| Operator::Minus)
        | tag("*").map(|_| Operator::Multiply)
        | tag("/").map(|_| Operator::Divide)
        | tag("%").map(|_| Operator::Modulus)
}

fn operator<'a>() -> Parser<'a, char, Operator> {
    tag("eq").map(|_| Operator::Eq)
        | tag("neq").map(|_| Operator::Neq)
        | tag("lte").map(|_| Operator::Lte)
        | tag("lt").map(|_| Operator::Lt)
        | tag("gte").map(|_| Operator::Gte)
        | tag("gt").map(|_| Operator::Gt)
        | tag("in").map(|_| Operator::In)
        | tag("not_in").map(|_| Operator::NotIn)
        | tag("is_not").map(|_| Operator::IsNot)
        | tag("like").map(|_| Operator::Like)
        | tag("ilike").map(|_| Operator::Ilike)
        | tag("starts").map(|_| Operator::Starts)
        | connector()
        | math_operator()
}

fn expr<'a>() -> Parser<'a, char, Expr> {
    (sym('(') * call(expr) - sym(')')).map(|expr| Expr::Nested(Box::new(expr)))
        | multi_values().map(Expr::MultiValue)
        | null().map(Expr::Value)
        | bool().map(|v| Expr::Value(Value::Bool(v)))
        | number().map(|v| Expr::Value(Value::Number(v)))
        | function().map(Expr::Function)
        | column().map(Expr::Column)
        | value().map(Expr::Value)
}

fn exprs_with_renames<'a>() -> Parser<'a, char, Vec<ExprRename>> {
    list_fail(call(expr_rename), sym(','))
}

/// column=>new_column
///
/// or
///
/// column=^new_column
fn expr_rename<'a>() -> Parser<'a, char, ExprRename> {
    (expr() + ((tag("=>") | tag("=^")) * strict_ident()).opt())
        .map(|(expr, rename)| ExprRename { rename, expr })
}

fn expr_projection<'a>() -> Parser<'a, char, Vec<ExprRename>> {
    sym('{') * exprs_with_renames() - sym('}')
}

fn exprs<'a>() -> Parser<'a, char, Vec<Expr>> {
    list_fail(expr(), sym(','))
}

pub(crate) fn function<'a>() -> Parser<'a, char, Function> {
    (strict_ident() - sym('(') - sym(')')).map(|name| {
        Function {
            name,
            params: vec![],
        }
    }) | (strict_ident() - sym('(') + call(exprs) - sym(')'))
        .map(|(name, params)| Function { name, params })
}

#[allow(dead_code)]
fn math_operation<'a>() -> Parser<'a, char, BinaryOperation> {
    (expr() + math_operator() + expr()).map(|((left, operator), right)| {
        BinaryOperation {
            left,
            operator,
            right,
        }
    })
}

fn simple_operation<'a>() -> Parser<'a, char, BinaryOperation> {
    (expr() + operator() + expr()).map(|((left, operator), right)| {
        BinaryOperation {
            left,
            operator,
            right,
        }
    })
}

fn simple_operation_expr<'a>() -> Parser<'a, char, Expr> {
    (sym('(') * call(simple_operation_expr) - sym(')'))
        .map(|expr| Expr::Nested(Box::new(expr)))
        | call(simple_operation)
            .map(|binop| Expr::BinaryOperation(Box::new(binop)))
}

fn binary_operation_expr<'a>() -> Parser<'a, char, Expr> {
    (sym('(') * call(binary_operation_expr) - sym(')'))
        .map(|expr| Expr::Nested(Box::new(expr)))
        | (expr() - sym('=') + (operator() - sym('.')).opt() + expr()).map(
            |((left, operator), right)| {
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left,
                    operator: operator.unwrap_or(Operator::Eq),
                    right,
                }))
            },
        )
        | (simple_operation_expr() + connector() + call(binary_operation_expr))
            .map(|((left, operator), right)| {
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left,
                    operator,
                    right,
                }))
            })
        | (expr() + connector() + call(binary_operation_expr)).map(
            |((left, operator), right)| {
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left,
                    operator,
                    right,
                }))
            },
        )
        | call(simple_operation_expr)
}

fn simple_filter_expr<'a>() -> Parser<'a, char, Expr> {
    (sym('(') * call(simple_filter_expr) - sym(')'))
        .map(|expr| Expr::Nested(Box::new(expr)))
        | (call(binary_operation_expr) + operator() + call(simple_filter_expr))
            .map(|((left, operator), right)| {
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left,
                    operator,
                    right,
                }))
            })
        | (call(binary_operation_expr)
            + operator()
            + call(binary_operation_expr))
        .map(|((left, operator), right)| {
            Expr::BinaryOperation(Box::new(BinaryOperation {
                left,
                operator,
                right,
            }))
        })
        | (call(binary_operation_expr) + operator() + expr()).map(
            |((left, operator), right)| {
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left,
                    operator,
                    right,
                }))
            },
        )
        | call(binary_operation_expr)
}

/// parse filter as Expr
pub fn filter_expr<'a>() -> Parser<'a, char, Expr> {
    (call(simple_filter_expr) + operator() + call(filter_expr)).map(
        |((left, operator), right)| {
            Expr::BinaryOperation(Box::new(BinaryOperation {
                left,
                operator,
                right,
            }))
        },
    ) | call(simple_filter_expr)
}

fn from_table<'a>() -> Parser<'a, char, FromTable> {
    (table().expect("Expecting a valid table name")
        + (join_type() + call(from_table)))
    .map(|(from, (join_type, from_table))| {
        FromTable {
            from,
            join: Some((join_type, Box::new(from_table))),
        }
    }) | (table().expect("Expecting a valid table name")
        + (join_type() + call(from_table)).opt())
    .map(|(from, join)| {
        FromTable {
            from,
            join: join.map(|(join_type, from_table)| {
                (join_type, Box::new(from_table))
            }),
        }
    })
}

fn join_type<'a>() -> Parser<'a, char, JoinType> {
    tag("-><-").map(|_| JoinType::InnerJoin)
        | tag("<-->").map(|_| JoinType::FullJoin)
        | tag("<-").map(|_| JoinType::LeftJoin)
        | tag("->").map(|_| JoinType::RightJoin)
}

fn page_size<'a>() -> Parser<'a, char, i64> {
    (tag("page_size") - sym('='))
        * integer().expect("Expecting an integer value for page_size")
}

/// page=2&page_size=10
fn page<'a>() -> Parser<'a, char, Page> {
    ((tag("page") - sym('='))
        * integer().expect("Expecting an integer value for page")
        - sym('&')
        + page_size().expect("must specify a page_size"))
    .map(|(page, page_size)| Page { page, page_size })
}

fn offset<'a>() -> Parser<'a, char, i64> {
    (tag("offset") - sym('='))
        * integer().expect("Expecting an integer value for offset")
}

/// limit=10&offset=20
fn limit<'a>() -> Parser<'a, char, Limit> {
    ((tag("limit") - sym('='))
        * integer().expect("Expecting an integer value for limit")
        + (sym('&') * offset()).opt())
    .map(|(limit, offset)| Limit { limit, offset })
}

fn range<'a>() -> Parser<'a, char, Range> {
    page().map(Range::Page) | limit().map(Range::Limit)
}

fn direction<'a>() -> Parser<'a, char, Direction> {
    tag("asc").map(|_| Direction::Asc) | tag("desc").map(|_| Direction::Desc)
}

/// height.asc
fn order<'a>() -> Parser<'a, char, Order> {
    (expr() + (sym('.') * direction()).opt())
        .map(|(expr, direction)| Order { expr, direction })
}

/// person{name,age,class}?(age=gt.42&student=eq.true)|(gender=eq.M)&group_by=(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10
pub fn select<'a>() -> Parser<'a, char, Select> {
    (from_table() + expr_projection().opt() - sym('?').opt()
        + filter_expr().opt()
        + (sym('&') * tag("group_by=") * list_fail(expr(), sym(','))).opt()
        + (sym('&') * tag("having=") * filter_expr()).opt()
        + (sym('&') * tag("order_by=") * list_fail(call(order), sym(',')))
            .opt()
        + (sym('&') * range()).opt()
        - end_or_ln())
    .map(
        |(
            (
                ((((from_table, projection), filter), group_by), having),
                order_by,
            ),
            range,
        )| {
            Select {
                from_table,
                filter,
                group_by,
                having,
                projection,
                order_by,
                range,
            }
        },
    )
}

#[cfg(test)]
mod test_private {
    use super::*;

    #[test]
    fn test_add_operation() {
        let input = to_chars("age+year");
        let ret = simple_operation().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            BinaryOperation {
                left: Expr::Column(ColumnName { name: "age".into() }),
                operator: Operator::Plus,
                right: Expr::Column(ColumnName {
                    name: "year".into()
                }),
            }
        );
    }

    #[test]
    fn test_math_operation() {
        let input = to_chars("1+1");
        let ret = math_operation().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            BinaryOperation {
                left: Expr::Value(Value::Number(1.0)),
                operator: Operator::Plus,
                right: Expr::Value(Value::Number(1.0))
            }
        );
    }

    #[test]
    fn test_binary_operation() {
        let input = to_chars("age=gt.42");
        let ret = binary_operation_expr()
            .parse(&input)
            .expect("must be parsed");
        println!("{:#?}", ret);
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
    fn test_binary_operation_grouped() {
        let input = to_chars("((age=gt.42))");
        let ret = binary_operation_expr()
            .parse(&input)
            .expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            Expr::Nested(Box::new(Expr::Nested(Box::new(
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName { name: "age".into() }),
                    operator: Operator::Gt,
                    right: Expr::Value(Value::Number(42.0))
                }))
            ))))
        );
    }

    #[test]
    fn test_complex_expr() {
        let input = to_chars("age=gt.42|true");
        let ret = filter_expr().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName { name: "age".into() }),
                    operator: Operator::Gt,
                    right: Expr::Value(Value::Number(42.0))
                })),
                operator: Operator::Or,
                right: Expr::Value(Value::Bool(true))
            }))
        );
    }
    #[test]
    fn test_complex_expr_rev() {
        let input = to_chars("false|age=gt.42");
        let ret = filter_expr().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            Expr::BinaryOperation(Box::new(BinaryOperation {
                left: Expr::Value(Value::Bool(false)),
                operator: Operator::Or,
                right: Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(ColumnName { name: "age".into() }),
                    operator: Operator::Gt,
                    right: Expr::Value(Value::Number(42.0))
                })),
            }))
        );
    }
    #[test]
    fn test_complex_filter_grouped() {
        let input = to_chars("(false|age=gt.42)");
        let ret = filter_expr().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            Expr::Nested(Box::new(Expr::BinaryOperation(Box::new(
                BinaryOperation {
                    left: Expr::Value(Value::Bool(false)),
                    operator: Operator::Or,
                    right: Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: Expr::Column(ColumnName { name: "age".into() }),
                        operator: Operator::Gt,
                        right: Expr::Value(Value::Number(42.0))
                    })),
                }
            ))))
        );
    }

    #[test]
    fn test_complex_filter_grouped_of_grouped() {
        let input = to_chars("(age=gt.42)|(is_active=true)");
        let ret = filter_expr().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
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
}

#[cfg(test)]
mod tests;
