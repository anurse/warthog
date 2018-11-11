use std::collections::VecDeque;

use crate::{
    module::Instruction,
    text::{
        parser::utils,
        sexpr::{SExpr, SVal},
        ParserError, ParserErrorKind,
    },
    Value,
};

pub fn parse_instructions(
    rest: &mut VecDeque<SExpr>,
    list: &mut Vec<Instruction>,
) -> Result<(), ParserError> {
    loop {
        match rest.pop_front() {
            Some(SExpr(SVal::Expr(body), _, _)) => unfold_instructions(body, list)?,
            Some(SExpr(SVal::Atom(inst), _, _)) => list.push(parse_instruction(inst, rest)?),
            Some(SExpr(val, start, end)) => {
                return Err(err!(
                    (start, end),
                    ParserErrorKind::UnexpectedToken,
                    format!("Expected an instruction, but found: '{:?}'", val),
                ))
            }
            None => return Ok(()),
        }
    }
}

fn unfold_instructions(
    mut body: VecDeque<SExpr>,
    list: &mut Vec<Instruction>,
) -> Result<(), ParserError> {
    unimplemented!();
}

fn parse_instruction(name: String, rest: &mut VecDeque<SExpr>) -> Result<Instruction, ParserError> {
    match name.as_str() {
        "i32.const" => {
            // Next token should be the constant value
            let val = utils::pop_int(rest)?;
            Ok(Instruction::Const(Value::Integer32(val as i32)))
        },
        "call" => {
            // Next token should be the callee
            let func_id = utils::pop_int(rest)?;
            Ok(Instruction::Call(func_id as usize))
        },
        x => panic!("Instruction not yet implemented: {}", x),
    }
}