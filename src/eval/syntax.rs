use std::collections::{HashMap, HashSet};

use crate::{error::SchemeError, reader::Datum};

#[derive(Clone, Debug)]
pub struct SyntaxRules {
    literals: HashSet<String>,
    rules: Vec<SyntaxRule>,
}

#[derive(Clone, Debug)]
struct SyntaxRule {
    pattern: Datum,
    template: Datum,
}

#[derive(Clone, Debug)]
enum Binding {
    Single(Datum),
    Repeated(Vec<Datum>),
}

type Bindings = HashMap<String, Binding>;

impl SyntaxRules {
    pub fn compile(expr: &Datum) -> Result<Self, SchemeError> {
        let items = expr.collect_proper_list().ok_or_else(|| {
            SchemeError::syntax("'syntax-rules' form must be a proper list", None)
        })?;

        if items.len() < 3
            || items.first().and_then(|item| item.as_symbol()) != Some("syntax-rules")
        {
            return Err(SchemeError::syntax(
                "expected a '(syntax-rules ...)' transformer specification",
                None,
            ));
        }

        let literal_items = items[1].collect_proper_list().ok_or_else(|| {
            SchemeError::syntax("'syntax-rules' literals must be a proper list", None)
        })?;

        let mut literals = HashSet::new();
        for literal in literal_items {
            let Datum::Symbol(name) = literal else {
                return Err(SchemeError::syntax(
                    "'syntax-rules' literal identifiers must be symbols",
                    None,
                ));
            };
            literals.insert(name.clone());
        }

        let mut rules = Vec::new();
        for rule in &items[2..] {
            let parts = rule.collect_proper_list().ok_or_else(|| {
                SchemeError::syntax("each syntax rule must be a proper list", None)
            })?;
            if parts.len() != 2 {
                return Err(SchemeError::syntax(
                    "each syntax rule must contain a pattern and a template",
                    None,
                ));
            }
            rules.push(SyntaxRule {
                pattern: parts[0].clone(),
                template: parts[1].clone(),
            });
        }

        Ok(Self { literals, rules })
    }

    pub fn expand(&self, form: &Datum) -> Result<Datum, SchemeError> {
        for rule in &self.rules {
            let mut bindings = Bindings::new();
            if match_pattern(&rule.pattern, form, &self.literals, &mut bindings)? {
                return expand_template(&rule.template, &bindings, None);
            }
        }

        Err(SchemeError::syntax(
            format!("no syntax-rules pattern matched form: {form}"),
            None,
        ))
    }
}

fn match_pattern(
    pattern: &Datum,
    input: &Datum,
    literals: &HashSet<String>,
    bindings: &mut Bindings,
) -> Result<bool, SchemeError> {
    match pattern {
        Datum::Boolean(value) => Ok(matches!(input, Datum::Boolean(other) if other == value)),
        Datum::Number(value) => Ok(matches!(input, Datum::Number(other) if other == value)),
        Datum::Float(value) => Ok(matches!(input, Datum::Float(other) if other == value)),
        Datum::Character(value) => Ok(matches!(input, Datum::Character(other) if other == value)),
        Datum::String(value) => Ok(matches!(input, Datum::String(other) if other == value)),
        Datum::ByteVector(value) => Ok(matches!(input, Datum::ByteVector(other) if other == value)),
        Datum::EmptyList => Ok(matches!(input, Datum::EmptyList)),
        Datum::Symbol(name) => match name.as_str() {
            "_" => Ok(true),
            "..." => Err(SchemeError::syntax(
                "ellipsis cannot appear alone in a pattern",
                None,
            )),
            _ if literals.contains(name) => {
                Ok(matches!(input, Datum::Symbol(other) if other == name))
            }
            _ => bind_single(name, input, bindings),
        },
        Datum::Vector(pattern_items) => match input {
            Datum::Vector(input_items) => {
                let pattern_refs = pattern_items.iter().collect::<Vec<_>>();
                let input_refs = input_items.iter().collect::<Vec<_>>();
                match_sequence_pattern(&pattern_refs, &input_refs, literals, bindings)
            }
            _ => Ok(false),
        },
        Datum::Pair(_, _) => {
            if let (Some(pattern_items), Some(input_items)) =
                (pattern.collect_proper_list(), input.collect_proper_list())
            {
                match_sequence_pattern(&pattern_items, &input_items, literals, bindings)
            } else if let (
                Datum::Pair(pattern_car, pattern_cdr),
                Datum::Pair(input_car, input_cdr),
            ) = (pattern, input)
            {
                if !match_pattern(pattern_car, input_car, literals, bindings)? {
                    return Ok(false);
                }
                match_pattern(pattern_cdr, input_cdr, literals, bindings)
            } else {
                Ok(false)
            }
        }
    }
}

fn match_sequence_pattern(
    patterns: &[&Datum],
    inputs: &[&Datum],
    literals: &HashSet<String>,
    bindings: &mut Bindings,
) -> Result<bool, SchemeError> {
    if patterns.is_empty() {
        return Ok(inputs.is_empty());
    }

    if is_ellipsis_pattern(patterns, 0) {
        let repeated_pattern = patterns[0];
        for count in 0..=inputs.len() {
            let mut trial = bindings.clone();
            if !match_repeated_pattern(repeated_pattern, &inputs[..count], literals, &mut trial)? {
                continue;
            }
            if match_sequence_pattern(&patterns[2..], &inputs[count..], literals, &mut trial)? {
                *bindings = trial;
                return Ok(true);
            }
        }
        return Ok(false);
    }

    let Some((first_input, rest_inputs)) = inputs.split_first() else {
        return Ok(false);
    };

    if !match_pattern(patterns[0], first_input, literals, bindings)? {
        return Ok(false);
    }

    match_sequence_pattern(&patterns[1..], rest_inputs, literals, bindings)
}

fn match_repeated_pattern(
    pattern: &Datum,
    inputs: &[&Datum],
    literals: &HashSet<String>,
    bindings: &mut Bindings,
) -> Result<bool, SchemeError> {
    let repeated_vars = collect_pattern_variables(pattern, literals);

    for name in &repeated_vars {
        bindings
            .entry(name.clone())
            .or_insert_with(|| Binding::Repeated(Vec::new()));
    }

    for input in inputs {
        let mut iteration_bindings = HashMap::new();
        if !match_pattern_iteration(pattern, input, literals, bindings, &mut iteration_bindings)? {
            return Ok(false);
        }
        for (name, value) in iteration_bindings {
            append_repeated_binding(bindings, &name, value)?;
        }
    }

    Ok(true)
}

fn match_pattern_iteration(
    pattern: &Datum,
    input: &Datum,
    literals: &HashSet<String>,
    outer_bindings: &Bindings,
    iteration_bindings: &mut HashMap<String, Datum>,
) -> Result<bool, SchemeError> {
    match pattern {
        Datum::Boolean(value) => Ok(matches!(input, Datum::Boolean(other) if other == value)),
        Datum::Number(value) => Ok(matches!(input, Datum::Number(other) if other == value)),
        Datum::Float(value) => Ok(matches!(input, Datum::Float(other) if other == value)),
        Datum::Character(value) => Ok(matches!(input, Datum::Character(other) if other == value)),
        Datum::String(value) => Ok(matches!(input, Datum::String(other) if other == value)),
        Datum::ByteVector(value) => Ok(matches!(input, Datum::ByteVector(other) if other == value)),
        Datum::EmptyList => Ok(matches!(input, Datum::EmptyList)),
        Datum::Symbol(name) => match name.as_str() {
            "_" => Ok(true),
            "..." => Err(SchemeError::syntax(
                "ellipsis cannot appear alone inside a repeated pattern",
                None,
            )),
            _ if literals.contains(name) => {
                Ok(matches!(input, Datum::Symbol(other) if other == name))
            }
            _ => bind_iteration(name, input, outer_bindings, iteration_bindings),
        },
        Datum::Vector(pattern_items) => match input {
            Datum::Vector(input_items) => match_iteration_sequence(
                &pattern_items.iter().collect::<Vec<_>>(),
                &input_items.iter().collect::<Vec<_>>(),
                literals,
                outer_bindings,
                iteration_bindings,
            ),
            _ => Ok(false),
        },
        Datum::Pair(_, _) => {
            if let (Some(pattern_items), Some(input_items)) =
                (pattern.collect_proper_list(), input.collect_proper_list())
            {
                match_iteration_sequence(
                    &pattern_items,
                    &input_items,
                    literals,
                    outer_bindings,
                    iteration_bindings,
                )
            } else if let (
                Datum::Pair(pattern_car, pattern_cdr),
                Datum::Pair(input_car, input_cdr),
            ) = (pattern, input)
            {
                if !match_pattern_iteration(
                    pattern_car,
                    input_car,
                    literals,
                    outer_bindings,
                    iteration_bindings,
                )? {
                    return Ok(false);
                }
                match_pattern_iteration(
                    pattern_cdr,
                    input_cdr,
                    literals,
                    outer_bindings,
                    iteration_bindings,
                )
            } else {
                Ok(false)
            }
        }
    }
}

fn match_iteration_sequence(
    patterns: &[&Datum],
    inputs: &[&Datum],
    literals: &HashSet<String>,
    outer_bindings: &Bindings,
    iteration_bindings: &mut HashMap<String, Datum>,
) -> Result<bool, SchemeError> {
    if patterns.is_empty() {
        return Ok(inputs.is_empty());
    }

    if is_ellipsis_pattern(patterns, 0) {
        return Err(SchemeError::syntax(
            "nested ellipsis in syntax-rules patterns is not supported yet",
            None,
        ));
    }

    let Some((first_input, rest_inputs)) = inputs.split_first() else {
        return Ok(false);
    };

    if !match_pattern_iteration(
        patterns[0],
        first_input,
        literals,
        outer_bindings,
        iteration_bindings,
    )? {
        return Ok(false);
    }

    match_iteration_sequence(
        &patterns[1..],
        rest_inputs,
        literals,
        outer_bindings,
        iteration_bindings,
    )
}

fn expand_template(
    template: &Datum,
    bindings: &Bindings,
    repeat_index: Option<usize>,
) -> Result<Datum, SchemeError> {
    match template {
        Datum::Boolean(_)
        | Datum::Number(_)
        | Datum::Float(_)
        | Datum::Character(_)
        | Datum::String(_)
        | Datum::ByteVector(_)
        | Datum::EmptyList => Ok(template.clone()),
        Datum::Symbol(name) => match bindings.get(name) {
            Some(Binding::Single(value)) => Ok(value.clone()),
            Some(Binding::Repeated(values)) => {
                let Some(index) = repeat_index else {
                    return Err(SchemeError::syntax(
                        format!("repeated template variable '{name}' used outside ellipsis"),
                        None,
                    ));
                };
                values.get(index).cloned().ok_or_else(|| {
                    SchemeError::syntax(
                        format!("template repetition index out of range for '{name}'"),
                        None,
                    )
                })
            }
            None => Ok(Datum::Symbol(name.clone())),
        },
        Datum::Vector(items) => {
            let item_refs = items.iter().collect::<Vec<_>>();
            Ok(Datum::Vector(expand_sequence_template(
                &item_refs,
                bindings,
                repeat_index,
            )?))
        }
        Datum::Pair(_, _) => {
            if let Some(items) = template.collect_proper_list() {
                Ok(Datum::list(expand_sequence_template(
                    &items,
                    bindings,
                    repeat_index,
                )?))
            } else if let Datum::Pair(car, cdr) = template {
                Ok(Datum::pair(
                    expand_template(car, bindings, repeat_index)?,
                    expand_template(cdr, bindings, repeat_index)?,
                ))
            } else {
                unreachable!()
            }
        }
    }
}

fn expand_sequence_template(
    items: &[&Datum],
    bindings: &Bindings,
    repeat_index: Option<usize>,
) -> Result<Vec<Datum>, SchemeError> {
    let mut expanded = Vec::new();
    let mut index = 0;

    while index < items.len() {
        if is_ellipsis_pattern(items, index) {
            if repeat_index.is_some() {
                return Err(SchemeError::syntax(
                    "nested ellipsis in syntax-rules templates is not supported yet",
                    None,
                ));
            }

            let repetitions = repetition_count(items[index], bindings)?;
            for rep_index in 0..repetitions {
                expanded.push(expand_template(items[index], bindings, Some(rep_index))?);
            }
            index += 2;
            continue;
        }

        expanded.push(expand_template(items[index], bindings, repeat_index)?);
        index += 1;
    }

    Ok(expanded)
}

fn repetition_count(template: &Datum, bindings: &Bindings) -> Result<usize, SchemeError> {
    let repeated_names = collect_template_repeated_variables(template, bindings);
    let Some(first_name) = repeated_names.first() else {
        return Err(SchemeError::syntax(
            "ellipsis template segment must reference at least one repeated pattern variable",
            None,
        ));
    };

    let expected = match bindings.get(first_name) {
        Some(Binding::Repeated(values)) => values.len(),
        _ => 0,
    };

    for name in &repeated_names[1..] {
        let current = match bindings.get(name) {
            Some(Binding::Repeated(values)) => values.len(),
            _ => 0,
        };
        if current != expected {
            return Err(SchemeError::syntax(
                "repeated template variables must have the same length",
                None,
            ));
        }
    }

    Ok(expected)
}

fn collect_pattern_variables(pattern: &Datum, literals: &HashSet<String>) -> Vec<String> {
    let mut vars = Vec::new();
    collect_pattern_variables_inner(pattern, literals, &mut vars);
    vars
}

fn collect_pattern_variables_inner(
    pattern: &Datum,
    literals: &HashSet<String>,
    vars: &mut Vec<String>,
) {
    match pattern {
        Datum::Symbol(name) => {
            if name != "_" && name != "..." && !literals.contains(name) && !vars.contains(name) {
                vars.push(name.clone());
            }
        }
        Datum::Pair(car, cdr) => {
            collect_pattern_variables_inner(car, literals, vars);
            collect_pattern_variables_inner(cdr, literals, vars);
        }
        Datum::Vector(items) => {
            for item in items {
                collect_pattern_variables_inner(item, literals, vars);
            }
        }
        _ => {}
    }
}

fn collect_template_repeated_variables(template: &Datum, bindings: &Bindings) -> Vec<String> {
    let mut vars = Vec::new();
    collect_template_repeated_variables_inner(template, bindings, &mut vars);
    vars
}

fn collect_template_repeated_variables_inner(
    template: &Datum,
    bindings: &Bindings,
    vars: &mut Vec<String>,
) {
    match template {
        Datum::Symbol(name) => {
            if matches!(bindings.get(name), Some(Binding::Repeated(_))) && !vars.contains(name) {
                vars.push(name.clone());
            }
        }
        Datum::Pair(car, cdr) => {
            collect_template_repeated_variables_inner(car, bindings, vars);
            collect_template_repeated_variables_inner(cdr, bindings, vars);
        }
        Datum::Vector(items) => {
            for item in items {
                collect_template_repeated_variables_inner(item, bindings, vars);
            }
        }
        _ => {}
    }
}

fn bind_single(name: &str, input: &Datum, bindings: &mut Bindings) -> Result<bool, SchemeError> {
    match bindings.get(name) {
        Some(Binding::Single(existing)) => Ok(existing == input),
        Some(Binding::Repeated(_)) => Err(SchemeError::syntax(
            format!("pattern variable '{name}' used in incompatible repeated and non-repeated positions"),
            None,
        )),
        None => {
            bindings.insert(name.to_string(), Binding::Single(input.clone()));
            Ok(true)
        }
    }
}

fn bind_iteration(
    name: &str,
    input: &Datum,
    outer_bindings: &Bindings,
    iteration_bindings: &mut HashMap<String, Datum>,
) -> Result<bool, SchemeError> {
    if let Some(Binding::Single(existing)) = outer_bindings.get(name) {
        return Ok(existing == input);
    }

    match iteration_bindings.get(name) {
        Some(existing) => Ok(existing == input),
        None => {
            iteration_bindings.insert(name.to_string(), input.clone());
            Ok(true)
        }
    }
}

fn append_repeated_binding(
    bindings: &mut Bindings,
    name: &str,
    value: Datum,
) -> Result<(), SchemeError> {
    match bindings.get_mut(name) {
        Some(Binding::Repeated(values)) => {
            values.push(value);
            Ok(())
        }
        Some(Binding::Single(_)) => Err(SchemeError::syntax(
            format!("pattern variable '{name}' used in incompatible repeated and non-repeated positions"),
            None,
        )),
        None => {
            bindings.insert(name.to_string(), Binding::Repeated(vec![value]));
            Ok(())
        }
    }
}

fn is_ellipsis_pattern(items: &[&Datum], index: usize) -> bool {
    index + 1 < items.len() && items[index + 1].as_symbol() == Some("...")
}
