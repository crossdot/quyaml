// pub(self) mod parsers;
mod parsers;
pub use parsers::query as parse_query;

#[derive(Clone, Debug, PartialEq)]
pub enum Relation {
    Or,
    And,
    Xor,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionListItem {
    Condition(Condition),
    Statement(Statement),
    Not,
    Relation(Relation),
    Group(Vec<ConditionListItem>)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Boolean(bool),
    Integer(i64),
    String(String),
    Double(f64),
    None,
    Path(Vec<String>),
}


#[derive(Clone, Debug, PartialEq)]
pub enum CompareSign {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub left: Statement,
    pub sign: CompareSign,
    pub right: Statement,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathEntry {
    pub key: Option<String>,
    pub condition: Option<Vec<ConditionListItem>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub path: Vec<PathEntry>,
}


#[derive(Default)]
pub struct ParseError;
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A parsing error occurred.")
    }
}
impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <ParseError as std::fmt::Display>::fmt(self, f)
    }
}
impl std::error::Error for ParseError { }
