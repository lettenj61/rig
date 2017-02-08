use combine::*;
use combine::char::{alpha_num, char, string, spaces};
use combine::primitives::Consumed;

use super::template::*;

/// Intermediate state of parsing template
pub type Progress<'a> = (String, Option<Placeholder>, &'a str);

pub fn parse_template<'a>(tpl: &'a str, style: &'a Style)
    -> Result<Progress<'a>, ParseError<&'a str>>
{
    match *style {
        Style::ST => parse_st(tpl),
        Style::Path => parse_pathname(tpl),
        _ => unreachable!(),
    }
}

/// Parse template written in `StringTemplate` like format
fn parse_st(input: &str) -> Result<Progress, ParseError<&str>> {

    let ident = || many1::<String, _>(alpha_num().or(one_of("_-".chars())).skip(spaces()));
    let lex_char = |c| char(c).skip(spaces());

    let escape_ph = many::<String, _>(satisfy(|c| c != '$').then(|c| {
        parser(move |input| if c == '\\' {
            any()
                .map(|d| match d {
                    '\\' => '\\',
                    other => other
                })
                .parse_stream(input)
        } else {
            Ok((c, Consumed::Empty(input)))
        })
    }));

    let string_literal = between(char('"'), char('"'), many::<String, _>(satisfy(|c| c != '"')));
    let fmt_args = string("format").skip(spaces()).with(lex_char('=').with(string_literal));
    let placeholder = between(
        lex_char('$'),
        char('$'),
        ident().and(optional(char(';').with(fmt_args))))
        .map(|parsed| Placeholder::new(&parsed.0, parsed.1, Style::ST));

    let mut parser = escape_ph.and(optional(placeholder));
    parser.parse(input).map(|(result, rest)| (result.0, result.1, rest))
}

/// Parse template appears in path names.
fn parse_pathname(input: &str) -> Result<Progress, ParseError<&str>> {

    let ident = || many1::<String, _>(alpha_num().or(one_of("_-".chars())).skip(spaces()));
    let lex_char = |c| char(c).skip(spaces());

    let escape_ph = many::<String, _>(satisfy(|c| c != '$').then(|c| {
        parser(move |input| if c == '\\' {
            any()
                .map(|d| match d {
                    '\\' => '\\',
                    other => other
                })
                .parse_stream(input)
        } else {
            Ok((c, Consumed::Empty(input)))
        })
    }));

    let placeholder = between(
        lex_char('$'),
        char('$'),
        ident())
        .map(|parsed| {
            if let Some(i) = parsed.find("__") {
                let (name, args) = parsed.split_at(i);
                Placeholder::new(name, Some(args.into()), Style::Path)
            } else {
                Placeholder::new(&parsed, None, Style::Path)
            }
        });

    let mut parser = escape_ph.and(optional(placeholder));
    parser.parse(input).map(|(result, rest)| (result.0, result.1, rest))
}
