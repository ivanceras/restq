use sql_ast::ast as sql;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Eq,     // = ,  eq
    Neq,    // != , neq
    Lt,     // <,  lt
    Lte,    // <=, lte
    Gt,     // >, gt
    Gte,    // >=, gte
    And,    // AND
    Or,     // OR
    Like,   // LIKE, like
    In,     // (expr) IN, in
    NotIn,  // (expr) NOT IN, not_in
    Is,     // (expr) IS, is
    IsNot,  // (expr) IS NOT, is_not
    Ilike,  // ILIKE case insensitive like, postgresql specific
    Starts, // Starts with, which will become ILIKE 'value%'
}

impl Operator {
    pub(crate) fn needs_separator(&self) -> bool {
        match self {
            Operator::And
            | Operator::Or
            | Operator::Plus
            | Operator::Minus
            | Operator::Multiply
            | Operator::Divide
            | Operator::Modulus => false,

            Operator::Eq
            | Operator::Neq
            | Operator::Lt
            | Operator::Lte
            | Operator::Gt
            | Operator::Gte
            | Operator::Like
            | Operator::In
            | Operator::NotIn
            | Operator::Is
            | Operator::IsNot
            | Operator::Ilike
            | Operator::Starts => true,
        }
    }
}

/// convert restq to sql_ast operator
impl Into<sql::BinaryOperator> for &Operator {
    fn into(self) -> sql::BinaryOperator {
        match self {
            Operator::Eq => sql::BinaryOperator::Eq,
            Operator::Neq => sql::BinaryOperator::NotEq,
            Operator::Lt => sql::BinaryOperator::Lt,
            Operator::Lte => sql::BinaryOperator::LtEq,
            Operator::Gt => sql::BinaryOperator::Gt,
            Operator::Gte => sql::BinaryOperator::GtEq,
            Operator::And => sql::BinaryOperator::And,
            Operator::Or => sql::BinaryOperator::Or,
            Operator::Like => sql::BinaryOperator::Like,
            Operator::Ilike => sql::BinaryOperator::Ilike,
            Operator::Plus => sql::BinaryOperator::Plus,
            Operator::Minus => sql::BinaryOperator::Minus,
            Operator::Multiply => sql::BinaryOperator::Multiply,
            Operator::Divide => sql::BinaryOperator::Divide,
            Operator::Modulus => sql::BinaryOperator::Modulus,
            Operator::In => sql::BinaryOperator::In,
            _ => panic!("unsupported conversion"),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
            Operator::Modulus => write!(f, "%"),
            Operator::Eq => write!(f, "eq"),
            Operator::Neq => write!(f, "neq"),
            Operator::Lt => write!(f, "lt"),
            Operator::Lte => write!(f, "lte"),
            Operator::Gt => write!(f, "gt"),
            Operator::Gte => write!(f, "gte"),
            Operator::And => write!(f, "&"),
            Operator::Or => write!(f, "|"),
            Operator::Like => write!(f, "like"),
            Operator::In => write!(f, "in"),
            Operator::NotIn => write!(f, "not_in"),
            Operator::Is => write!(f, "is"),
            Operator::IsNot => write!(f, "is_not"),
            Operator::Ilike => write!(f, "ilike"),
            Operator::Starts => write!(f, "starts"),
        }
    }
}
