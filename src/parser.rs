//! Parser module for STIX patterns.
//!
//! This module uses pest to parse STIX pattern strings and converts
//! the parse tree into our AST representation using recursive descent.

use chrono::{DateTime, Utc};
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use thiserror::Error;

use crate::ast::{
    BooleanOp, Comparison, ComparisonExpr, ComparisonOp, ComparisonRhs, CompositeComparison,
    CompositePattern, ListIndex, ObjectPath, ObservationOp, PathComponent, PatternExpr,
    QualifiedPattern, StixValue, UnaryOp,
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct StixParser;

/// Errors that can occur during STIX pattern parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Grammar error: {0}")]
    Grammar(#[from] pest::error::Error<Rule>),

    #[error("Invalid integer: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),

    #[error("Invalid float: {0}")]
    InvalidFloat(#[from] std::num::ParseFloatError),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Unexpected rule: {0:?}")]
    UnexpectedRule(Rule),

    #[error("Missing expected element: {0}")]
    MissingElement(&'static str),
}

pub type Result<T> = std::result::Result<T, ParseError>;

/// Parse a STIX pattern string into a PatternExpr AST.
pub fn parse_pattern(input: &str) -> Result<PatternExpr> {
    let pair = StixParser::parse(Rule::pattern, input)?
        .next()
        .ok_or(ParseError::MissingElement("pattern"))?;

    parse_pair(pair)
}

/// Main recursive dispatch based on rule type.
fn parse_pair(pair: Pair<Rule>) -> Result<PatternExpr> {
    match pair.as_rule() {
        Rule::pattern => parse_pattern_rule(pair),
        Rule::expression => parse_expression(pair),
        Rule::observation => parse_observation(pair),
        Rule::observation_group => parse_observation_group(pair),
        _ => Err(ParseError::UnexpectedRule(pair.as_rule())),
    }
}

fn parse_pattern_rule(pair: Pair<Rule>) -> Result<PatternExpr> {
    pair.into_inner()
        .find(|p| p.as_rule() == Rule::expression)
        .map(parse_expression)
        .ok_or(ParseError::MissingElement("expression"))?
}

fn parse_expression(pair: Pair<Rule>) -> Result<PatternExpr> {
    let mut inner = pair.into_inner();

    let first = inner
        .next()
        .ok_or(ParseError::MissingElement("expression"))?;
    let mut left = parse_pair(first)?;

    while let Some(op_pair) = inner.next() {
        let op = parse_obs_op(&op_pair)?;
        let right_pair = inner
            .next()
            .ok_or(ParseError::MissingElement("right operand"))?;
        let right = parse_pair(right_pair)?;
        left = CompositePattern::new(left, op, right).into();
    }

    Ok(left)
}

fn parse_observation(pair: Pair<Rule>) -> Result<PatternExpr> {
    let mut expr: Option<ComparisonExpr> = None;
    let mut pending_op: Option<BooleanOp> = None;
    let mut qualifiers = Qualifiers::default();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::comparison => {
                let comp = parse_comparison(p)?;
                expr = Some(merge_exprs(expr, comp, pending_op.take()));
            }
            Rule::and => pending_op = Some(BooleanOp::And),
            Rule::or => pending_op = Some(BooleanOp::Or),
            Rule::qualifier => parse_qualifier(p, &mut qualifiers)?,
            _ => {}
        }
    }

    let pattern: PatternExpr = expr.ok_or(ParseError::MissingElement("comparison"))?.into();
    Ok(qualifiers.apply_to(pattern))
}

fn parse_observation_group(pair: Pair<Rule>) -> Result<PatternExpr> {
    let mut inner_pattern: Option<PatternExpr> = None;
    let mut qualifiers = Qualifiers::default();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::expression => inner_pattern = Some(parse_expression(p)?),
            Rule::qualifier => parse_qualifier(p, &mut qualifiers)?,
            _ => {}
        }
    }

    let pattern = inner_pattern.ok_or(ParseError::MissingElement("expression"))?;
    Ok(qualifiers.apply_to(pattern))
}

fn parse_comparison(pair: Pair<Rule>) -> Result<ComparisonExpr> {
    let mut inner = pair.into_inner().peekable();

    // Check what kind of comparison this is
    match inner.peek().map(|p| p.as_rule()) {
        // Parenthesized comparison expression
        Some(Rule::comparison) => {
            let mut expr: Option<ComparisonExpr> = None;
            let mut pending_op: Option<BooleanOp> = None;

            for p in inner {
                match p.as_rule() {
                    Rule::comparison => {
                        let comp = parse_comparison(p)?;
                        expr = Some(merge_exprs(expr, comp, pending_op.take()));
                    }
                    Rule::and => pending_op = Some(BooleanOp::And),
                    Rule::or => pending_op = Some(BooleanOp::Or),
                    _ => {}
                }
            }
            expr.ok_or(ParseError::MissingElement("comparison"))
        }

        // EXISTS comparison
        Some(Rule::exists) => {
            inner.next(); // consume exists
            let path_pair = inner.next().ok_or(ParseError::MissingElement("path"))?;
            let path = parse_object_path(path_pair)?;
            Ok(Comparison::new(path, UnaryOp::Exists, None, false).into())
        }

        // Normal comparison: path [NOT] op value
        Some(Rule::path) => {
            let path_pair = inner.next().unwrap();
            let path = parse_object_path(path_pair)?;

            let mut negated = false;
            let mut op: Option<ComparisonOp> = None;
            let mut rhs: Option<ComparisonRhs> = None;

            for p in inner {
                match p.as_rule() {
                    Rule::not => negated = true,
                    Rule::value => rhs = Some(parse_value(p)?.into()),
                    Rule::list => rhs = Some(parse_list(p)?.into()),
                    rule => {
                        if let Some(parsed_op) = try_parse_comp_op(rule) {
                            op = Some(parsed_op);
                        }
                    }
                }
            }

            let op = op.ok_or(ParseError::MissingElement("operator"))?;
            Ok(Comparison::new(path, op, rhs, negated).into())
        }

        _ => Err(ParseError::MissingElement("comparison content")),
    }
}

fn try_parse_comp_op(rule: Rule) -> Option<ComparisonOp> {
    match rule {
        Rule::equal => Some(ComparisonOp::Eq),
        Rule::not_equal => Some(ComparisonOp::Neq),
        Rule::gt => Some(ComparisonOp::Gt),
        Rule::lt => Some(ComparisonOp::Lt),
        Rule::ge => Some(ComparisonOp::Ge),
        Rule::le => Some(ComparisonOp::Le),
        Rule::r#in => Some(ComparisonOp::In),
        Rule::like => Some(ComparisonOp::Like),
        Rule::r#match => Some(ComparisonOp::Matches),
        Rule::issubset => Some(ComparisonOp::IsSubset),
        Rule::issuperset => Some(ComparisonOp::IsSuperset),
        _ => None,
    }
}

fn parse_obs_op(pair: &Pair<Rule>) -> Result<ObservationOp> {
    match pair.as_rule() {
        Rule::and => Ok(ObservationOp::And),
        Rule::or => Ok(ObservationOp::Or),
        Rule::followedby => Ok(ObservationOp::FollowedBy),
        _ => Err(ParseError::UnexpectedRule(pair.as_rule())),
    }
}

fn merge_exprs(
    left: Option<ComparisonExpr>,
    right: ComparisonExpr,
    op: Option<BooleanOp>,
) -> ComparisonExpr {
    match left {
        None => right,
        Some(l) => CompositeComparison::new(l, op.unwrap_or_default(), right).into(),
    }
}

fn parse_object_path(pair: Pair<Rule>) -> Result<ObjectPath> {
    let mut object_type = String::new();
    let mut property_path = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::object => object_type = p.as_str().to_owned(),
            Rule::step => property_path.push(parse_step(p)?),
            _ => {}
        }
    }

    Ok(ObjectPath::new(object_type, property_path))
}

fn parse_step(pair: Pair<Rule>) -> Result<PathComponent> {
    let mut property = String::new();
    let mut index = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::property => property = strip_quotes(p.as_str()),
            Rule::index => {
                let idx_str = p.as_str();
                index = Some(if idx_str == "*" {
                    ListIndex::Star
                } else {
                    ListIndex::Index(idx_str.parse()?)
                });
            }
            _ => {}
        }
    }

    Ok(PathComponent::new(property, index))
}

fn strip_quotes(s: &str) -> String {
    s.strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .unwrap_or(s)
        .to_owned()
}

fn parse_value(pair: Pair<Rule>) -> Result<StixValue> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or(ParseError::MissingElement("value content"))?;

    match inner.as_rule() {
        Rule::string => Ok(StixValue::String(unescape_string(inner.as_str()))),
        Rule::bool => Ok(StixValue::Bool(inner.as_str() == "true")),
        Rule::float => Ok(StixValue::Float(inner.as_str().parse()?)),
        Rule::int => Ok(StixValue::Int(inner.as_str().parse()?)),
        Rule::time => parse_timestamp(inner.as_str()).map(StixValue::Timestamp),
        Rule::hex => Ok(StixValue::Hex(inner.as_str().to_owned())),
        Rule::bin => Ok(StixValue::Binary(inner.as_str().to_owned())),
        _ => Err(ParseError::UnexpectedRule(inner.as_rule())),
    }
}

fn parse_list(pair: Pair<Rule>) -> Result<Vec<StixValue>> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::value)
        .map(parse_value)
        .collect()
}

#[derive(Default)]
struct Qualifiers {
    repeat: Option<u32>,
    within: Option<f64>,
    start: Option<DateTime<Utc>>,
    stop: Option<DateTime<Utc>>,
}

impl Qualifiers {
    fn is_empty(&self) -> bool {
        self.repeat.is_none()
            && self.within.is_none()
            && self.start.is_none()
            && self.stop.is_none()
    }

    fn apply_to(self, pattern: PatternExpr) -> PatternExpr {
        if self.is_empty() {
            pattern
        } else {
            QualifiedPattern::new(pattern, self.repeat, self.within, self.start, self.stop).into()
        }
    }
}

fn parse_qualifier(pair: Pair<Rule>, q: &mut Qualifiers) -> Result<()> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or(ParseError::MissingElement("qualifier content"))?;

    match inner.as_rule() {
        Rule::repeat => {
            for p in inner.into_inner() {
                if p.as_rule() == Rule::pos_int {
                    q.repeat = Some(p.as_str().parse()?);
                }
            }
        }
        Rule::within => {
            for p in inner.into_inner() {
                if matches!(p.as_rule(), Rule::pos_float | Rule::pos_int) {
                    q.within = Some(p.as_str().parse()?);
                }
            }
        }
        Rule::interval => {
            for p in inner.into_inner() {
                if p.as_rule() == Rule::time {
                    let ts = parse_timestamp(p.as_str())?;
                    if q.start.is_none() {
                        q.start = Some(ts);
                    } else {
                        q.stop = Some(ts);
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some(c @ ('\\' | '\'')) => result.push(c),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push(c),
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").map(|dt| dt.and_utc())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").map(|dt| dt.and_utc())
        })
        .map_err(|_| ParseError::InvalidTimestamp(s.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_comparison() {
        assert!(parse_pattern("[file:name = 'foo.dll']").is_ok());
    }

    #[test]
    fn test_exists() {
        assert!(parse_pattern("[EXISTS file:name]").is_ok());
    }

    #[test]
    fn test_composite_comparison() {
        assert!(parse_pattern("[file:name = 'foo' AND file:size > 100]").is_ok());
    }

    #[test]
    fn test_observation_with_qualifier() {
        assert!(parse_pattern("[file:name = 'foo'] REPEATS 5 TIMES").is_ok());
    }

    #[test]
    fn test_followedby() {
        assert!(parse_pattern("[file:name = 'a'] FOLLOWEDBY [file:name = 'b']").is_ok());
    }
}
