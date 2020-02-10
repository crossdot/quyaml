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

    pub fn trim<I, O2, E: nom::error::ParseError<I>, G>(sep: G) -> impl Fn(I) -> nom::IResult<I, O2, E>
    where
        I: nom::InputTakeAtPosition,
        <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
        G: Fn(I) -> nom::IResult<I, O2, E>,
    {
        move |input: I| {
            // let (input, _) = nom::character::complete::space0(input)?;
            // let (input, o2) = sep(input)?;
            // nom::character::complete::space0(input).map(|(i, _)| (i, o2))
            nom::sequence::delimited(
                nom::character::complete::space0,
                &sep,
                nom::character::complete::space0,
            )(input)
        }
    }

    #[allow(unused)]
    fn unescaped_path(i: &str) -> nom::IResult<&str, Vec<String>> {
        trim(
            nom::multi::separated_nonempty_list(
                nom::character::complete::char('.'),
                nom::bytes::complete::escaped_transform(
                    nom::bytes::complete::is_not("\\. \t=<>!&|^()"),
                    '\\',
                    nom::bytes::complete::is_a("\\. \t()"),
                )
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
        trim(
            nom::branch::alt((
                nom::combinator::map(boolean, |v| Statement::Boolean(v)),
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
            ))
        )(i)
    }

    #[allow(unused)]
    fn condition(i: &str) -> nom::IResult<&str, Condition> {
        trim(
            nom::combinator::map(
                nom::sequence::tuple((
                    value,
                    nom::branch::alt((
                        nom::bytes::complete::tag("=="),
                        nom::bytes::complete::tag("!="),
                        nom::bytes::complete::tag(">"),
                        nom::bytes::complete::tag("<"),
                    )),
                    value,
                )),
                |(left, relation, right)| Condition {
                    left: left,
                    sign: relation.to_owned(),
                    right: right,
                }
            )
        )(i)
    }

    #[allow(unused)]
    fn relation(i: &str) -> nom::IResult<&str, Relation> {
        trim(
            nom::branch::alt((
                nom::combinator::map(nom::bytes::complete::tag("||"), |_| Relation::Or),
                nom::combinator::map(nom::bytes::complete::tag("&&"), |_| Relation::And),
                nom::combinator::map(nom::bytes::complete::tag("^"), |_| Relation::Xor),
            ))
        )(i)
    }

    #[allow(unused)]
    fn condition_list_item(i: &str) -> nom::IResult<&str, ConditionListItem> {
        trim(
            nom::branch::alt((
                nom::combinator::map(
                    nom::sequence::delimited(
                        nom::bytes::complete::tag("("),
                        condition_list,
                        nom::bytes::complete::tag(")")
                    ),
                    |g| ConditionListItem::Group(g)
                ),
                nom::combinator::map(condition, |st| ConditionListItem::Condition(st)),
                nom::combinator::map(value, |v| ConditionListItem::Statement(v)),
            ))
        )(i)
    }

    #[allow(unused)]
    fn condition_list(i: &str) -> nom::IResult<&str, Vec<ConditionListItem>> {
        let mut list = Vec::new();
        nom::combinator::map(
            nom::sequence::tuple((
                nom::multi::fold_many0(
                    nom::sequence::tuple((
                        condition_list_item,
                        nom::combinator::map(relation, |v| ConditionListItem::Relation(v))
                    )),
                    list,
                    |mut acc: Vec<_>, (st, rel)| {
                        acc.push(st);
                        acc.push(rel);
                        acc
                    }
                ),
                condition_list_item
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
            assert_eq!(unescaped_path(""), Err(nom::Err::Error(("", nom::error::ErrorKind::SeparatedList))));
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
            assert_eq!(condition_list("first&&(true==false)"), Ok(("",
                vec![
                    ConditionListItem::Statement(Statement::Path(vec!["first".to_owned()])),
                    ConditionListItem::Relation(Relation::And),
                    ConditionListItem::Group(vec![
                        ConditionListItem::Condition(Condition {
                            left: Statement::Boolean(true),
                            sign: "==".to_owned(),
                            right: Statement::Boolean(false)
                        })
                    ])
                ]
            )));
            assert_eq!(condition_list("first.value && (false || true != false)"), Ok(("", 
                vec![
                    ConditionListItem::Statement(Statement::Path(vec![
                        "first".to_owned(),
                        "value".to_owned(),
                    ])),
                    ConditionListItem::Relation(Relation::And),
                    ConditionListItem::Group(
                        vec![
                            ConditionListItem::Statement(Statement::Boolean(false)),
                            ConditionListItem::Relation(Relation::Or),
                            ConditionListItem::Condition(Condition {
                                left: Statement::Boolean(true),
                                sign: "!=".to_owned(),
                                right: Statement::Boolean(false),
                            }),
                        ]
                    ),
                ]
            )));
        }
        
        // #[test]
        // fn test_query() {
        //     assert_eq!(query("first.second(third.fourth=1).third"), Ok(("", vec![])));
        // }
    }
}
