#[derive(Clone, Debug, PartialEq)]
enum Statement {
    Boolean(bool),
    Integer(i64),
    String(String),
    Double(f64),
    None,
    Path(String),
}

#[derive(Clone, Default, Debug)]
pub struct Condition {
    pub left : String,
    pub sign : String,
    pub right : String,
}

#[derive(Clone, Default, Debug)]
pub struct PathEntry {
    pub key: Option<String>,
    pub condition: Option<Condition>,
}

#[derive(Clone, Default, Debug)]
pub struct Query {
	pub path: Vec<PathEntry>,
}


// #[derive(Default)]
// pub struct ParseError;
// impl std::fmt::Display for ParseError {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		write!(f, "A parsing error occurred.")
// 	}
// }
// impl std::fmt::Debug for ParseError {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		<ParseError as std::fmt::Display>::fmt(self, f)
// 	}
// }
// impl std::error::Error for ParseError { }

pub(self) mod parsers {
    use super::*;

    #[allow(unused)]
    fn path(i: &str) -> nom::IResult<&str, Vec<&str>> {
        nom::multi::separated_list(
            nom::character::complete::char('.'),
            nom::bytes::complete::escaped(
                nom::bytes::complete::is_not("\\. "),
                '\\',
                nom::bytes::complete::is_a("\\. "),
            )
        )(i)
    }

    #[allow(unused)]
    fn transform_path(i: &str) -> nom::IResult<&str, Vec<String>> {
        nom::multi::separated_list(
            nom::character::complete::char('.'),
            nom::bytes::complete::escaped_transform(
                nom::bytes::complete::is_not("\\. "),
                '\\',
                nom::bytes::complete::is_a("\\. "),
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
	fn value(i: &str) -> nom::IResult<&str, Statement> {
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("true"), |_| Statement::Boolean(true)),
            nom::combinator::map(nom::bytes::complete::tag("false"), |_| Statement::Boolean(false)),
            nom::combinator::map(nom::bytes::complete::tag("null"), |_| Statement::None),
            nom::combinator::map(quoted_string, |s: &str| Statement::String(s.to_owned())),
        ))(i)
	}

    #[allow(unused)]
    fn condition(i: &str) -> nom::IResult<&str, Condition> {
        match nom::combinator::all_consuming(nom::sequence::tuple((
            nom::character::complete::space0,
            nom::branch::alt((
                nom::bytes::complete::tag("true"),
                nom::bytes::complete::tag("false"),
                nom::bytes::complete::tag("null"),
            )),
			nom::character::complete::space0,
            nom::branch::alt((
                nom::bytes::complete::tag("=="),
                nom::bytes::complete::tag("!="),
                nom::bytes::complete::tag(">"),
                nom::bytes::complete::tag("<"),
            )),
			nom::character::complete::space0,
            nom::branch::alt((
                nom::bytes::complete::tag("true"),
                nom::bytes::complete::tag("false"),
                nom::bytes::complete::tag("null"),
            )),
			nom::character::complete::space0,
		)))(i) {
            Ok((remaining_input, (
                _,
                _,
                _,
                _,
                _,
                _,
                _,
            ))) => {
                Ok((remaining_input, Condition {
                    left: "adsf".to_owned(),
                    sign: "adsf".to_owned(),
                    right: "adsf".to_owned(),
                }))
            }
            Err(e) => Err(e)
        }
    }

    #[cfg(test)]
	mod tests {
        use super::*;

		#[test]
		fn test_path() {
			assert_eq!(path("first"), Ok(("", vec!["first"])));
			assert_eq!(path("fir\\\\st"), Ok(("", vec!["fir\\\\st"])));
			assert_eq!(path("first.second"), Ok(("", vec!["first", "second"])));
			assert_eq!(path("first.sec\\.ond"), Ok(("", vec!["first", "sec\\.ond"])));
        }
		
		#[test]
		fn test_transform_path() {
			assert_eq!(transform_path("first"), Ok(("", vec!["first".to_owned()])));
			assert_eq!(transform_path("fir\\\\st"), Ok(("", vec!["fir\\st".to_owned()])));
			assert_eq!(transform_path("first.second"), Ok(("", vec!["first".to_owned(), "second".to_owned()])));
			assert_eq!(transform_path("first.sec\\.ond"), Ok(("", vec!["first".to_owned(), "sec.ond".to_owned()])));
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
        }
		
		// #[test]
		// fn test_condition() -> Result<(), std::io::Error> {
        //     let cond = Condition {
        //         left: "asdf".to_owned(),
        //         sign: "asdf".to_owned(),
        //         right: "asdf".to_owned(),
        //     };
        //     let (_, cond2) = condition("first==1")?;
		// 	// assert_eq!(condition("first=1"), Ok(("", )));
        // }
		
		// #[test]
		// fn test_query() {
		// 	assert_eq!(query("first.second(third.fourth=1).third"), Ok(("", vec![])));
        // }
    }
}
