use super::*;
use nom::character::complete as character;
use nom::bytes::complete as bytes;
use nom::multi as multi;
use nom::combinator as combinator;
use nom::sequence as sequence;
use nom::branch as branch;

fn trim<I, O2, E: nom::error::ParseError<I>, G>(sep: G) -> impl Fn(I) -> nom::IResult<I, O2, E>
where
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
    G: Fn(I) -> nom::IResult<I, O2, E>,
{
    move |input: I| {
        sequence::delimited(
            character::space0,
            &sep,
            character::space0,
        )(input)
    }
}

#[allow(unused)]
fn unescaped_path(i: &str) -> nom::IResult<&str, Vec<String>> {
    trim(
        multi::separated_nonempty_list(
            character::char('.'),
            bytes::escaped_transform(
                bytes::is_not("\\. \t=<>!&|^()"),
                '\\',
                bytes::is_a("\\. \t()"),
            )
        )
    )(i)
}

#[allow(unused)]
fn quoted_string(i: &str) -> nom::IResult<&str, &str> {
    branch::alt((
        sequence::delimited(
            bytes::tag("\""),
            bytes::escaped(
                bytes::is_not("\\\""),
                '\\',
                bytes::is_a("\\\""),
            ),
            bytes::tag("\"")
        ),
        sequence::delimited(
            bytes::tag("'"),
            bytes::escaped(
                bytes::is_not("\\'"),
                '\\',
                bytes::is_a("\\'"),
            ),
            bytes::tag("'")
        ),
    ))(i)
}

#[allow(unused)]
fn boolean(i: &str) -> nom::IResult<&str, bool> {
    branch::alt((
        combinator::map(bytes::tag("true"), |_| true),
        combinator::map(bytes::tag("false"), |_| false),
    ))(i)
}

#[allow(unused)]
fn value(i: &str) -> nom::IResult<&str, Statement> {
    trim(
        branch::alt((
            combinator::map(boolean, |v| Statement::Boolean(v)),
            combinator::map(bytes::tag("null"), |_| Statement::None),
            combinator::map(nom::number::complete::recognize_float, |s: &str| {
                if s.chars().all(|c| c.is_numeric() || c == '-') {
                    Statement::Integer(s.parse().unwrap())
                } else {
                    Statement::Double(s.parse().unwrap())
                }
            }),
            combinator::map(quoted_string, |s: &str| Statement::String(s.to_owned())),
            combinator::map(unescaped_path, |path: Vec<String>| Statement::Path(path)),
        ))
    )(i)
}

#[allow(unused)]
fn compare_sign(i: &str) -> nom::IResult<&str, CompareSign> {
    branch::alt((
        combinator::value(CompareSign::Eq, bytes::tag("==")),
        combinator::value(CompareSign::Ne, bytes::tag("!=")),
        combinator::value(CompareSign::Ge, bytes::tag(">=")),
        combinator::value(CompareSign::Le, bytes::tag("<=")),
        combinator::value(CompareSign::Gt, bytes::tag(">")),
        combinator::value(CompareSign::Lt, bytes::tag("<")),
    ))(i)
}

#[allow(unused)]
fn condition(i: &str) -> nom::IResult<&str, Condition> {
    trim(
        combinator::map(
            sequence::tuple((
                value,
                compare_sign,
                value,
            )),
            |(left, relation, right)| Condition {
                left: left,
                sign: relation,
                right: right,
            }
        )
    )(i)
}

#[allow(unused)]
fn relation(i: &str) -> nom::IResult<&str, Relation> {
    trim(
        branch::alt((
            combinator::map(bytes::tag("||"), |_| Relation::Or),
            combinator::map(bytes::tag("&&"), |_| Relation::And),
            combinator::map(bytes::tag("^"), |_| Relation::Xor),
        ))
    )(i)
}

#[allow(unused)]
fn condition_list_item(i: &str) -> nom::IResult<&str, ConditionListItem> {
    trim(
        branch::alt((
            combinator::map(
                sequence::delimited(
                    bytes::tag("("),
                    condition_list,
                    bytes::tag(")")
                ),
                |g| ConditionListItem::Group(g)
            ),
            combinator::map(condition, |st| ConditionListItem::Condition(st)),
            combinator::map(value, |v| ConditionListItem::Statement(v)),
        ))
    )(i)
}

#[allow(unused)]
fn condition_list(i: &str) -> nom::IResult<&str, Vec<ConditionListItem>> {
    let mut list = Vec::new();
    combinator::map(
        sequence::tuple((
            multi::fold_many0(
                sequence::tuple((
                    condition_list_item,
                    combinator::map(relation, |v| ConditionListItem::Relation(v))
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

#[allow(unused)]
fn query(i: &str) -> nom::IResult<&str, Query> {
    combinator::map(
        trim(
            sequence::tuple((
                multi::separated_nonempty_list(
                    character::char('.'),
                    combinator::map(
                        sequence::tuple((
                            combinator::opt(
                                bytes::escaped_transform(
                                    bytes::is_not("\\. \t=<>!&|^()"),
                                    '\\',
                                    bytes::is_a("\\. \t()"),
                                ),
                            ),
                            combinator::opt(
                                trim(
                                    sequence::delimited(
                                        bytes::tag("("),
                                        condition_list,
                                        bytes::tag(")"),
                                    )
                                )
                            )
                        )),
                        |(p, c)| {
                            PathEntry {
                                key: p,
                                condition: c
                            }
                        }
                    )
                ),
                combinator::opt(
                    sequence::tuple((
                        compare_sign,
                        value,
                    ))
                )
            ))
        ),
        |(path, opt)| Query { path: path }
    )(i)
}

#[allow(unused)]
pub fn parse_query(i: &str) -> Result<Query, ParseError> {
    let parse_result = combinator::all_consuming(query)(i);
    match parse_result {
        Ok((i, q)) => Result::Ok(q),
        Err(e) => Result::Err(ParseError)
    }
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
            sign: CompareSign::Eq,
            right: Statement::None,
        })));
        assert_eq!(condition("first.second==null"), Ok(("", Condition {
            left: Statement::Path(vec!["first".to_owned(), "second".to_owned()]),
            sign: CompareSign::Eq,
            right: Statement::None,
        })));
        assert_eq!(condition("10==10.1"), Ok(("", Condition {
            left: Statement::Integer(10),
            sign: CompareSign::Eq,
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
                        sign: CompareSign::Eq,
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
                            sign: CompareSign::Ne,
                            right: Statement::Boolean(false),
                        }),
                    ]
                ),
            ]
        )));
    }
    
    #[test]
    fn test_query() {
        assert_eq!(query("first.second"), Ok(("",
            Query { 
                path: vec![
                    PathEntry {
                        key: Some("first".to_owned()),
                        condition: None,
                    },
                    PathEntry {
                        key: Some("second".to_owned()),
                        condition: None,
                    },
                ] 
            }
        )));
        assert_eq!(query("first.*(aaa.bbb == 'some_value').third"), Ok(("",
            Query { 
                path: vec![
                    PathEntry {
                        key: Some("first".to_owned()),
                        condition: None,
                    },
                    PathEntry {
                        key: Some("*".to_owned()),
                        condition: Some(vec![
                            ConditionListItem::Condition(Condition {
                                left: Statement::Path(vec!["aaa".to_owned(), "bbb".to_owned()]),
                                sign: CompareSign::Eq,
                                right: Statement::String("some_value".to_owned()),
                            })
                        ]),
                    },
                    PathEntry {
                        key: Some("third".to_owned()),
                        condition: None,
                    },
                ] 
            }
        )));
    }
}