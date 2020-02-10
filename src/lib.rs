#[derive(Clone, Debug, PartialEq)]
pub enum Relation {
    Or,
    And,
    Xor,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionListItem {
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
pub struct Condition {
    pub left : Statement,
    pub sign : String,
    pub right : Statement,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathEntry {
    pub key: Option<String>,
    pub condition: Option<Condition>,
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

pub(self) mod parsers {
    use super::*;

    #[allow(unused)]
    fn unescaped_path(i: &str) -> nom::IResult<&str, Vec<String>> {
        nom::multi::separated_list(
            nom::character::complete::char('.'),
            nom::bytes::complete::escaped_transform(
                nom::bytes::complete::is_not("\\. \t=<>!"),
                '\\',
                nom::bytes::complete::is_a("\\. \t"),
            )
        )(i)
    }

    #[allow(unused)]
    fn quoted_string(i: &str) -> nom::IResult<&str, &str> {
        nom::branch::alt((
            nom::sequence::delimited(
                nom::bytes::complete::tag("\""),
                nom::bytes::complete::escaped(
                    nom::bytes::complete::is_not("\\\""),
                    '\\',
                    nom::bytes::complete::is_a("\\\""),
                ),
                nom::bytes::complete::tag("\"")
            ),
            nom::sequence::delimited(
                nom::bytes::complete::tag("'"),
                nom::bytes::complete::escaped(
                    nom::bytes::complete::is_not("\\'"),
                    '\\',
                    nom::bytes::complete::is_a("\\'"),
                ),
                nom::bytes::complete::tag("'")
            ),
        ))(i)
    }

    #[allow(unused)]
    fn boolean(i: &str) -> nom::IResult<&str, bool> {
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("true"), |_| true),
            nom::combinator::map(nom::bytes::complete::tag("false"), |_| false),
        ))(i)
    }

    #[allow(unused)]
    fn value(i: &str) -> nom::IResult<&str, Statement> {
        nom::branch::alt((
            nom::combinator::map(boolean, |v| Statement::Boolean(v)),
            nom::combinator::map(nom::bytes::complete::tag("false"), |_| Statement::Boolean(false)),
            nom::combinator::map(nom::bytes::complete::tag("null"), |_| Statement::None),
            nom::combinator::map(nom::number::complete::recognize_float, |s: &str| {
                if s.chars().all(|c| c.is_numeric() || c == '-') {
                    Statement::Integer(s.parse().unwrap())
                } else {
                    Statement::Double(s.parse().unwrap())
                }
            }),
            nom::combinator::map(quoted_string, |s: &str| Statement::String(s.to_owned())),
            nom::combinator::map(unescaped_path, |path: Vec<String>| Statement::Path(path)),
        ))(i)
    }

    #[allow(unused)]
    fn condition(i: &str) -> nom::IResult<&str, Condition> {
        match nom::combinator::all_consuming(nom::sequence::tuple((
            nom::character::complete::space0,
            value,
            nom::character::complete::space0,
            nom::branch::alt((
                nom::bytes::complete::tag("=="),
                nom::bytes::complete::tag("!="),
                nom::bytes::complete::tag(">"),
                nom::bytes::complete::tag("<"),
            )),
            nom::character::complete::space0,
            value,
            nom::character::complete::space0,
        )))(i) {
            Ok((remaining_input, (
                _,
                left,
                _,
                relation,
                _,
                right,
                _,
            ))) => {
                Ok((remaining_input, Condition {
                    left: left,
                    sign: relation.to_owned(),
                    right: right,
                }))
            }
            Err(e) => Err(e)
        }
    }

    #[allow(unused)]
    fn relation(i: &str) -> nom::IResult<&str, Relation> {
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("||"), |_| Relation::Or),
            nom::combinator::map(nom::bytes::complete::tag("&&"), |_| Relation::And),
            nom::combinator::map(nom::bytes::complete::tag("^"), |_| Relation::And),
        ))(i)
    }

    #[allow(unused)]
    fn condition_list(i: &str) -> nom::IResult<&str, Vec<ConditionListItem>> {
        let mut list = Vec::new();
        nom::combinator::map(
        nom::sequence::tuple((
            nom::multi::fold_many0(
                nom::sequence::tuple((
                    nom::branch::alt((
                        nom::combinator::map(boolean, |v| ConditionListItem::Statement(Statement::Boolean(v))),
                        nom::combinator::map(
                            nom::sequence::delimited(
                                nom::bytes::complete::tag("("),
                                condition_list,
                                nom::bytes::complete::tag(")")
                            ),
                            |g| ConditionListItem::Group(g)
                        ),
                    )),
                    nom::combinator::map(relation, |v| ConditionListItem::Relation(v))
                )),
                list,
                |mut acc: Vec<_>, (st, rel)| {
                    acc.push(st);
                    acc.push(rel);
                    acc
                }
            ),
            nom::branch::alt((
                nom::combinator::map(boolean, |v| ConditionListItem::Statement(Statement::Boolean(v))),
                nom::combinator::map(
                    nom::sequence::delimited(
                        nom::bytes::complete::tag("("),
                        condition_list,
                        nom::bytes::complete::tag(")")
                    ),
                    |g| ConditionListItem::Group(g)
                ),
            ))
        )),
        |(mut acc, st)| {
            acc.push(st);
            acc
        }
        )
        (i)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_unescaped_path() {
            assert_eq!(unescaped_path("first"), Ok(("", vec!["first".to_owned()])));
            assert_eq!(unescaped_path("fir\\\\st"), Ok(("", vec!["fir\\st".to_owned()])));
            assert_eq!(unescaped_path("first.second"), Ok(("", vec!["first".to_owned(), "second".to_owned()])));
            assert_eq!(unescaped_path("first.sec\\.ond"), Ok(("", vec!["first".to_owned(), "sec.ond".to_owned()])));
        }

        #[test]
        fn test_quoted_string() {
            assert_eq!(quoted_string("\"hello\""), Ok(("", "hello")));
            assert_eq!(quoted_string("\"he\\\"llo\""), Ok(("", "he\\\"llo")));
            assert_eq!(quoted_string("'hello'"), Ok(("", "hello")));
            assert_eq!(quoted_string("'he\\'llo'"), Ok(("", "he\\'llo")));
        }

        #[test]
        fn test_value() {
            assert_eq!(value("true"), Ok(("", Statement::Boolean(true))));
            assert_eq!(value("false"), Ok(("", Statement::Boolean(false))));
            assert_eq!(value("null"), Ok(("", Statement::None)));
            assert_eq!(value("\"hello\""), Ok(("", Statement::String("hello".to_owned()))));
            assert_eq!(value("first_underscored"), Ok(("", Statement::Path(vec!["first_underscored".to_owned()]))));
            assert_eq!(value("first.second"), Ok(("", Statement::Path(vec!["first".to_owned(), "second".to_owned()]))));
            assert_eq!(value("10"), Ok(("", Statement::Integer(10))));
            assert_eq!(value("-10"), Ok(("", Statement::Integer(-10))));
            assert_eq!(value("1.1"), Ok(("", Statement::Double(1.1))));
            assert_eq!(value("-1.1"), Ok(("", Statement::Double(-1.1))));
        }
        
        #[test]
        fn test_condition() {
            assert_eq!(condition("true == null"), Ok(("", Condition {
                left: Statement::Boolean(true),
                sign: "==".to_owned(),
                right: Statement::None,
            })));
            assert_eq!(condition("first.second==null"), Ok(("", Condition {
                left: Statement::Path(vec!["first".to_owned(), "second".to_owned()]),
                sign: "==".to_owned(),
                right: Statement::None,
            })));
            assert_eq!(condition("10==10.1"), Ok(("", Condition {
                left: Statement::Integer(10),
                sign: "==".to_owned(),
                right: Statement::Double(10.1),
            })));
        }
        
        #[test]
        fn test_condition_list() {
            assert_eq!(condition_list("false"), Ok(("", 
                vec![
                    ConditionListItem::Statement(Statement::Boolean(false)),
                ]
            )));
            assert_eq!(condition_list("false||true&&false"), Ok(("", 
                vec![
                    ConditionListItem::Statement(Statement::Boolean(false)),
                    ConditionListItem::Relation(Relation::Or),
                    ConditionListItem::Statement(Statement::Boolean(true)),
                    ConditionListItem::Relation(Relation::And),
                    ConditionListItem::Statement(Statement::Boolean(false)),
                ]
            )));
            assert_eq!(condition_list("(false||true)&&false"), Ok(("", 
                vec![
                    ConditionListItem::Group(
                        vec![
                            ConditionListItem::Statement(Statement::Boolean(false)),
                            ConditionListItem::Relation(Relation::Or),
                            ConditionListItem::Statement(Statement::Boolean(true)),
                        ]
                    ),
                    ConditionListItem::Relation(Relation::And),
                    ConditionListItem::Statement(Statement::Boolean(false)),
                ]
            )));
        }
        
        // #[test]
        // fn test_query() {
        //     assert_eq!(query("first.second(third.fourth=1).third"), Ok(("", vec![])));
        // }
    }
}
