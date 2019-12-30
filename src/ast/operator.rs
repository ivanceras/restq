use sqlparser::ast as sql;

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

/// convert restq to sqlparser operator
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
            Operator::Plus => sql::BinaryOperator::Plus,
            Operator::Minus => sql::BinaryOperator::Minus,
            Operator::Multiply => sql::BinaryOperator::Multiply,
            Operator::Divide => sql::BinaryOperator::Divide,
            Operator::Modulus => sql::BinaryOperator::Modulus,
            _ => panic!("unsupported conversion"),
        }
    }
}
