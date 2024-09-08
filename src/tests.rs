use crate::*;

#[test]
fn single_char_regex() {
    let regex = "a";
    let mut ctx = Context::new(regex);
    let result = ctx.parse().expect("Failed to parse regex");
    assert_eq!(result, Regex::Char('a'));
}

#[test]
fn multiple_char_regex() {
    let regex = "ab";
    let mut ctx = Context::new(regex);
    let result = ctx.parse().expect("Failed to parse regex");
    assert_eq!(result, Regex::Multi(vec![Regex::Char('a'), Regex::Char('b')]));
}

#[test]
fn lots_of_char_regex() {
    let regex = "abhdsfjhgjasddff";
    let mut ctx = Context::new(regex);
    let result = ctx.parse().expect("Failed to parse regex");
    assert_eq!(
        result, 
        Regex::Multi(vec![
            Regex::Char('a'),
            Regex::Char('b'),
            Regex::Char('h'),
            Regex::Char('d'),
            Regex::Char('s'),
            Regex::Char('f'),
            Regex::Char('j'),
            Regex::Char('h'),
            Regex::Char('g'),
            Regex::Char('j'),
            Regex::Char('a'),
            Regex::Char('s'),
            Regex::Char('d'),
            Regex::Char('d'),
            Regex::Char('f'),
            Regex::Char('f'),
        ])
    );
}

#[test]
fn long_mix_regex() {
    let regex = "abhdsfj(hgj)as(dd+)ff";
    let mut ctx = Context::new(regex);
    let result = ctx.parse().expect("Failed to parse regex");
    assert_eq!(
        result, 
        Regex::Multi(vec![
            Regex::Char('a'),
            Regex::Char('b'),
            Regex::Char('h'),
            Regex::Char('d'),
            Regex::Char('s'),
            Regex::Char('f'),
            Regex::Char('j'),
            Regex::Multi(vec![
                Regex::Char('h'),
                Regex::Char('g'),
                Regex::Char('j'),
            ]),
            Regex::Char('a'),
            Regex::Char('s'),
            Regex::Multi(vec![
                Regex::Char('d'),
                Regex::Many(0),
            ]),
            Regex::Char('f'),
            Regex::Char('f'),
        ])
    );
}

#[test]
fn any_char_regex() {
    let regex = "a*";
    let mut ctx = Context::new(regex);
    let result = ctx.parse().expect("Failed to parse regex");
    assert_eq!(result, Regex::Any(0));
}

#[test]
fn many_char_regex() {
    let regex = "a+";
    let mut ctx = Context::new(regex);
    let result = ctx.parse().expect("Failed to parse regex");
    assert_eq!(result, Regex::Many(0));
}

#[test]
fn invalid_regex() {
    let regex = "(";
    let mut ctx = Context::new(regex);
    let result = ctx.parse();

    assert!(result.is_err());
}
