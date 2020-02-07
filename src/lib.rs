#[derive(Clone, Default, Debug)]
pub struct PathEntry {
    pub key: Option<String>,
    pub condition: Option<String>,
}

#[derive(Clone, Default, Debug)]
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
    use super::Query;

	fn escaped(i: &str) -> nom::IResult<&str, &str> {
		nom::bytes::complete::escaped(
            nom::bytes::complete::is_not("\\. "),
            '\\',
            nom::bytes::complete::is_a("\\. "),
            // nom::branch::alt((
            //     nom::character::complete::char('.'),
            //     nom::character::complete::char('\\'),
            //     nom::character::complete::char(' '),
            // ))
        )(i)
	}

	fn transform_escaped(i: &str) -> nom::IResult<&str, String> {
		nom::bytes::complete::escaped_transform(
            nom::bytes::complete::is_not("\\. "),
            '\\',
            nom::bytes::complete::is_a("\\. "),
            // nom::branch::alt((
            //     nom::character::complete::char('.'),
            //     nom::character::complete::char('\\'),
            //     nom::character::complete::char(' '),
            // ))
        )(i)
	}
        
    fn path(i: &str) -> nom::IResult<&str, Vec<&str>> {
        nom::multi::separated_list(
            nom::character::complete::char('.'),
            escaped
        )(i)
    }
        
    fn transform_path(i: &str) -> nom::IResult<&str, Vec<String>> {
        nom::multi::separated_list(
            nom::character::complete::char('.'),
            transform_escaped
        )(i)
    }

    #[cfg(test)]
	mod tests {
        use super::*;

		#[test]
		fn test_escaped() {
			assert_eq!(escaped("first"), Ok(("", "first")));
			assert_eq!(escaped("first\\ second"), Ok(("", "first\\ second")));
			assert_eq!(escaped("first\\.second"), Ok(("", "first\\.second")));
        }

		#[test]
		fn test_transform_escaped() {
			assert_eq!(transform_escaped("first"), Ok(("", "first".to_owned())));
			assert_eq!(transform_escaped("first\\ second"), Ok(("", "first second".to_owned())));
			assert_eq!(transform_escaped("first\\.second"), Ok(("", "first.second".to_owned())));
        }
		
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
    }
}
