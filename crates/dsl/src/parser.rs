use crate::ast::*;
use winnow::ascii::{alphanumeric1, digit1, hex_digit1, multispace0, multispace1};
use winnow::combinator::{alt, delimited, eof, opt, preceded, repeat, separated, seq, terminated};
use winnow::error::ModalResult;
use winnow::prelude::*;
use winnow::token::take_while;

type PResult<O> = ModalResult<O>;

// Top-level parser
pub fn parse_script(input: &mut &str) -> PResult<Script> {
    let (global_settings, macros, blocks) = seq!(
        _: multispace0,
        repeat(0.., terminated(parse_global_setting, multispace0)),
        repeat(0.., terminated(parse_macro, multispace0)),
        repeat(0.., terminated(parse_block, multispace0)),
        _: eof
    )
    .parse_next(input)?;

    Ok(Script {
        global_settings,
        macros,
        blocks,
    })
}

// Global Settings
fn parse_global_setting(input: &mut &str) -> PResult<GlobalSetting> {
    seq!(
        _: "CLI",
        _: multispace0,
        _: "=",
        _: multispace0,
        parse_string_literal,
        _: multispace0,
        _: ";"
    )
    .map(|(val,)| GlobalSetting::Cli(val))
    .parse_next(input)
}

fn parse_identifier(input: &mut &str) -> PResult<String> {
    take_while(1.., |c: char| c.is_alphanumeric() || c == '_')
        .map(String::from)
        .parse_next(input)
}

// Macros
fn parse_macro(input: &mut &str) -> PResult<Macro> {
    seq!(
        _: "macro",
        _: multispace1,
        parse_identifier,
        _: multispace0,
        parse_body_block
    )
    .map(|(name, body)| Macro { name, body })
    .parse_next(input)
}

// Blocks
fn parse_block(input: &mut &str) -> PResult<Block> {
    let triggers = separated(
        1..,
        parse_trigger_combinations,
        (multispace0, "+", multispace0),
    )
    .parse_next(input)?;
    let body = parse_body_block(input)?;
    Ok(Block { triggers, body })
}

fn parse_body_block(input: &mut &str) -> PResult<Vec<Statement>> {
    delimited(
        (multispace0, "{", multispace0),
        repeat(0.., terminated(parse_statement, multispace0)),
        (multispace0, "}"),
    )
    .parse_next(input)
}

fn parse_trigger_combinations(input: &mut &str) -> PResult<TriggerCombinations> {
    let keys: Vec<TriggerKey> =
        separated(1.., parse_trigger_key, (multispace0, "+", multispace0)).parse_next(input)?;
    Ok(TriggerCombinations(keys))
}

// Keys
fn parse_trigger_key(input: &mut &str) -> PResult<TriggerKey> {
    alt((
        parse_extended_physical_key,
        parse_physical_key,
        parse_virtual_key,
    ))
    .parse_next(input)
}

fn parse_extended_physical_key(input: &mut &str) -> PResult<TriggerKey> {
    ("#E0/0x", hex_digit1)
        .map(|(_, hex)| TriggerKey::ExtendedPhysical(u16::from_str_radix(hex, 16).unwrap_or(0)))
        .parse_next(input)
}

fn parse_physical_key(input: &mut &str) -> PResult<TriggerKey> {
    ("#0x", hex_digit1)
        .map(|(_, hex)| TriggerKey::Physical(u16::from_str_radix(hex, 16).unwrap_or(0)))
        .parse_next(input)
}

fn parse_virtual_key(input: &mut &str) -> PResult<TriggerKey> {
    alt((
        preceded("Code_", alphanumeric1), // Strict Code_...
        alphanumeric1,                    // Or just Name (like A, Escape)
    ))
    .map(|s: &str| TriggerKey::Virtual(s.to_string()))
    .parse_next(input)
}

// Statements
fn parse_statement(input: &mut &str) -> PResult<Statement> {
    alt((
        parse_try_run,
        parse_try_execute,
        parse_run,
        parse_execute,
        parse_send,
        parse_wait_stmt,
        parse_if,
        parse_loop,
        parse_macro_call,
    ))
    .parse_next(input)
}

fn parse_run(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "Run", _: multispace0, _: ":", _: multispace0,
        parse_string_literal,
        _: multispace0, _: ";"
    )
    .map(|(val,)| Statement::Run(val))
    .parse_next(input)
}

fn parse_execute(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "Execute", _: multispace0, _: ":", _: multispace0,
        parse_string_literal,
        _: multispace0, _: ";"
    )
    .map(|(val,)| Statement::Execute(val))
    .parse_next(input)
}

fn parse_try_run(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "TryRun", _: multispace0, _: ":", _: multispace0,
        parse_string_literal,
        _: multispace0, _: ":", _: multispace0,
        parse_fail_stmt,
        _: multispace0, _: ";"
    )
    .map(|(cmd, fallback)| Statement::TryRun {
        command: cmd,
        failure: Some(Box::new(fallback)),
    })
    .parse_next(input)
}

fn parse_try_execute(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "TryExecute", _: multispace0, _: ":", _: multispace0,
        parse_string_literal,
        _: multispace0, _: ":", _: multispace0,
        parse_fail_stmt,
        _: multispace0, _: ";"
    )
    .map(|(cmd, fallback)| Statement::TryExecute {
        command: cmd,
        failure: Some(Box::new(fallback)),
    })
    .parse_next(input)
}

fn parse_fail_stmt(input: &mut &str) -> PResult<Statement> {
    alt((parse_fail_run, parse_fail_execute)).parse_next(input)
}

fn parse_fail_run(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "FailRun", _: multispace0, _: ":", _: multispace0,
        parse_string_literal
    )
    .map(|(val,)| Statement::Run(val))
    .parse_next(input)
}

fn parse_fail_execute(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "FailExecute", _: multispace0, _: ":", _: multispace0,
        parse_string_literal
    )
    .map(|(val,)| Statement::Execute(val))
    .parse_next(input)
}

fn parse_send(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "Send", _: multispace0, _: ":", _: multispace0,
        separated(1.., parse_send_expression, (multispace0, "+", multispace0)),
        _: multispace0, _: ";"
    )
    .map(|(exprs,)| Statement::Send(exprs))
    .parse_next(input)
}

fn parse_send_expression(input: &mut &str) -> PResult<SendExpression> {
    alt((parse_string_literal_expr, parse_key_expr)).parse_next(input)
}

fn parse_string_literal_expr(input: &mut &str) -> PResult<SendExpression> {
    seq!(
        _: "String", _: multispace0, _: "(", _: multispace0,
        parse_string_literal,
        _: multispace0, _: ")"
    )
    .map(|(s,)| SendExpression::String(s))
    .parse_next(input)
}

fn parse_key_expr(input: &mut &str) -> PResult<SendExpression> {
    parse_trigger_key
        .map(|k| SendExpression::Key(k))
        .parse_next(input)
}

fn parse_wait_stmt(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "wait", _: multispace0, _: "(", _: multispace0,
        digit1,
        _: multispace0, _: ")", _: multispace0, _: ";"
    )
    .map(|(val,): (&str,)| Statement::Wait(val.parse::<u64>().unwrap_or(0)))
    .parse_next(input)
}

fn parse_if(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "if", _: multispace1,
        parse_condition,
        _: multispace0,
        parse_body_block,
        _: multispace0,
        repeat(0.., seq!(
            _: "elif", _: multispace1,
            parse_condition,
            _: multispace0,
            parse_body_block
        )),
        _: multispace0,
        opt(preceded(("else", multispace0), parse_body_block))
    )
    .map(|(cond, then_b, elif_bs, else_b)| Statement::If {
        condition: cond,
        then_branch: then_b,
        else_if_branches: elif_bs,
        else_branch: else_b,
    })
    .parse_next(input)
}

fn parse_loop(input: &mut &str) -> PResult<Statement> {
    seq!(
        _: "loop", _: multispace1,
        digit1,
        _: multispace0,
        parse_body_block
    )
    .map(|(count, body): (&str, Vec<Statement>)| Statement::Loop {
        count: count.parse().unwrap_or(1),
        body,
    })
    .parse_next(input)
}

fn parse_macro_call(input: &mut &str) -> PResult<Statement> {
    seq!(
        parse_identifier,
        _: "!", _: multispace0, _: ";"
    )
    .map(|(name,)| Statement::MacroCall(name))
    .parse_next(input)
}

// Conditions
fn parse_condition(input: &mut &str) -> PResult<Condition> {
    alt((
        parse_wait_input_time,
        parse_wait_input,
        parse_now_input,
        parse_wait_released_time,
        parse_wait_released,
    ))
    .parse_next(input)
}

fn parse_wait_input(input: &mut &str) -> PResult<Condition> {
    seq!(
        _: "wait_input", _: multispace0, _: "(", _: multispace0,
        parse_condition_args,
        _: multispace0, _: ")"
    )
    .map(|(args,)| Condition::WaitInput(args))
    .parse_next(input)
}

fn parse_wait_input_time(input: &mut &str) -> PResult<Condition> {
    seq!(
        _: "wait_input_time", _: multispace0, _: "(", _: multispace0,
        parse_condition_args,
        _: multispace0, _: ",", _: multispace0,
        digit1,
        _: multispace0, _: ")"
    )
    .map(|(args, time): (Vec<TriggerCombinations>, &str)| {
        Condition::WaitInputTime(args, time.parse().unwrap_or(0))
    })
    .parse_next(input)
}

fn parse_now_input(input: &mut &str) -> PResult<Condition> {
    seq!(
        _: "now_input", _: multispace0, _: "(", _: multispace0,
        parse_condition_args,
        _: multispace0, _: ")"
    )
    .map(|(args,)| Condition::NowInput(args))
    .parse_next(input)
}

fn parse_wait_released(input: &mut &str) -> PResult<Condition> {
    seq!(
        _: "wait_released", _: multispace0, _: "(", _: multispace0,
        parse_condition_args,
        _: multispace0, _: ")"
    )
    .map(|(args,)| Condition::WaitReleased(args))
    .parse_next(input)
}

fn parse_wait_released_time(input: &mut &str) -> PResult<Condition> {
    seq!(
        _: "wait_released_time", _: multispace0, _: "(", _: multispace0,
        parse_condition_args,
        _: multispace0, _: ",", _: multispace0,
        digit1,
        _: multispace0, _: ")"
    )
    .map(|(args, time): (Vec<TriggerCombinations>, &str)| {
        Condition::WaitReleasedTime(args, time.parse().unwrap_or(0))
    })
    .parse_next(input)
}

fn parse_condition_args(input: &mut &str) -> PResult<Vec<TriggerCombinations>> {
    let combo = parse_trigger_combinations(input)?;
    Ok(vec![combo])
}

// Utilities
fn parse_string_literal(input: &mut &str) -> PResult<String> {
    delimited('"', take_while(0.., |c| c != '"' && c != '\\'), '"')
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_macro() {
        let mut input = r#"macro MY_MACRO { Run: "echo"; }"#;
        let m = parse_macro.parse_next(&mut input).unwrap();
        assert_eq!(m.name, "MY_MACRO");
    }

    #[test]
    fn test_parse_try_run() {
        let mut input = r#"TryRun: "calc":FailExecute: "error";"#;
        let s = parse_try_run.parse_next(&mut input).unwrap();
        match s {
            Statement::TryRun { command, failure } => {
                assert_eq!(command, "calc");
                assert!(failure.is_some());
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_parse_simple_block() {
        let mut input = r#"#0x02 { Run: "cmd"; }"#;
        let b = parse_block.parse_next(&mut input).unwrap();
        assert_eq!(b.triggers.len(), 1); // 1 combination
        assert_eq!(b.triggers[0].0.len(), 1); // containing 1 key
    }

    #[test]
    fn test_parse_complex_script() {
        let mut input = r#"
            CLI = "PowerShell";
            
            macro MY_MACRO {
                Run: "echo macro";
            }

            #0x02 + Code_A {
                TryRun: "calc.exe" :FailExecute: "error.exe";
                wait(100);
                if now_input(#E0/0x10) {
                   Send: String("Pressed") + Code_Enter;
                   MY_MACRO!;
                }
            }
        "#;
        let script = parse_script.parse_next(&mut input).unwrap();
        assert_eq!(script.macros.len(), 1);
        assert_eq!(script.blocks.len(), 1);

        // Extended key check
        if let Statement::If { condition, .. } = &script.blocks[0].body[2] {
            match condition {
                Condition::NowInput(combos) => match &combos[0].0[0] {
                    TriggerKey::ExtendedPhysical(code) => assert_eq!(*code, 0x10),
                    _ => panic!("Expected ExtendedPhysical"),
                },
                _ => panic!("Expected NowInput"),
            }
        } else {
            panic!("Expected If statement");
        }
    }
}
