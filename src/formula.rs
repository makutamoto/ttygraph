use std::sync::Once;
use regex::Regex;

const SYNTAX_ERROR: &str = "Formulae must conform this syntax: [Left] = [Right]";
const DIVIDED_BY_NEG: &str = "Numbers can't be powerd by negative values.";
const DIVIDED_BY_ZERO: &str = "Numbers can't be divided by 0.";
const NO_OPERAND_FOUND: &str = "No operand found.";
const INVALID_STACK: &str = "Invalid stack id.";
const INVALID_OPERAND: &str = "Invalid operand!";

#[derive(Debug)]
enum Operand {
    Decimal(bool, f64),
    Stack(bool, usize),
    X(bool),
    Y(bool),
}

#[derive(Debug, Clone)]
enum Operation {
    Power,
    Multiple,
    Divide,
    Modulus,
    Add,
}

#[derive(Debug)]
struct Instruction {
    operand_left: Operand,
    operand_right: Operand,
    operation: Operation,
    out: usize,
}

#[derive(Debug)]
pub struct Side {
    instructions: Vec<Instruction>
}

#[derive(Debug)]
pub struct Formula {
    raw: String,
    color_pair: i16,
    pub left: Side,
    pub right: Side,
}

static INIT_SIDES: Once = Once::new();
static mut SIDES: Option<Regex> = None;
impl Formula {
    pub fn new(formula: &str, color_pair: i16) -> Result<Formula, &str> {
        INIT_SIDES.call_once(|| {
            unsafe {
                SIDES = Some(Regex::new(r"([^=]+)=([^=]+)").unwrap());
            }
        });
        let sides = match unsafe { SIDES.as_ref().unwrap().captures(formula) } {
            Some(matched) => {
                if matched[0].len() != formula.len() {
                    return Err(SYNTAX_ERROR);
                }
                matched
            },
            None => return Err(SYNTAX_ERROR),
        };
        let left = match Side::new(sides.get(1).unwrap().as_str()) {
            Ok(side) => side,
            Err(e) => return Err(e),
        };
        let right = match Side::new(sides.get(2).unwrap().as_str()) {
            Ok(side) => side,
            Err(e) => return Err(e),
        };
        Ok(Formula {
            raw: formula.to_string(),
            color_pair, left, right,
        })
    }

    pub fn raw(self: &Self) -> &str {
        &self.raw
    }

    pub fn color_pair(self: &Self) -> i16 {
        self.color_pair
    }
}

static REGEX_INIT: Once = Once::new();
static mut UNARY: Option<Regex> = None;
static mut PARENTHESES: Option<Regex> = None;
static mut POWER: Option<Regex> = None;
static mut MULTIPLE: Option<Regex> = None;
static mut DIVIDE: Option<Regex> = None;
static mut MODULUS: Option<Regex> = None;
static mut ADD: Option<Regex> = None;
impl Side {
    pub fn new(formula: &str) -> Result<Side, &str> {
        REGEX_INIT.call_once(|| {
            unsafe {
                UNARY = Some(Regex::new(r"(-?)\s*([xy]|S*\d+)").unwrap());
                PARENTHESES = Some(Regex::new(r"\((.*?)\)").unwrap());
                POWER = Some(Regex::new(r"(-?)\s*([xy]|S*\d+)\s*\^\s*(-?)\s*([xy]|S*\d+)").unwrap());
                MULTIPLE = Some(Regex::new(r"(-?)\s*([xy]|S*\d+)\s*\*\s*(-?)\s*([xy]|S*\d+)").unwrap());
                DIVIDE = Some(Regex::new(r"(-?)\s*([xy]|S*\d+)\s*/\s*(-?)\s*([xy]|S*\d+)").unwrap());
                MODULUS = Some(Regex::new(r"(-?)\s*([xy]|S*\d+)\s*%\s*(-?)\s*([xy]|S*\d+)").unwrap());
                ADD = Some(Regex::new(r"(-?)\s*([xy]|S*\d+)\s*(?:\+|(-))\s*([xy]|S*\d+)").unwrap());
            }
        });
        let mut instructions = Vec::new();
        let mut nof_stacks = 0;
        let formula = String::from(formula);
        match parse(formula, &mut nof_stacks, &mut instructions) {
            Ok(_) => {
                Ok(Side {
                    instructions,
                })
            },
            Err(e) => Err(e),
        }
    }

    pub fn calc(self: &Self, x: f64, y: f64) -> Result<f64, &'static str> {
        let mut stacks = Vec::<f64>::new();
        for instruction in &self.instructions {
            let operands = (operand_into_decimal(&stacks, &instruction.operand_left, x, y), operand_into_decimal(&stacks, &instruction.operand_right, x, y));
            let result = match instruction.operation {
                Operation::Power => {
                    let left_operand = operand_into_signed_decimal(&stacks, &instruction.operand_left, x, y);
                    let result;
                    if operands.1 < 0.0 {
                        return Err(DIVIDED_BY_NEG);
                    }
                    result = operands.0.powf(operands.1);
                    if left_operand.0 {
                        result
                    } else {
                        - result
                    }
                },
                Operation::Multiple => operands.0 * operands.1,
                Operation::Divide => {
                    if operands.1 == 0.0 {
                        return Err(DIVIDED_BY_ZERO);
                    }
                    operands.0 / operands.1
                },
                Operation::Modulus => operands.0 % operands.1,
                Operation::Add => operands.0 + operands.1,
            };
            stacks.insert(instruction.out, result);
        }
        Ok(stacks[stacks.len() - 1])
    }
}

fn parse(mut formula: String, nof_stacks: &mut usize, instructions: &mut Vec<Instruction>) -> Result<usize, &'static str> {
    loop {
        let range;
        let id;
        match unsafe { PARENTHESES.as_ref().unwrap().find(&formula) } {
            Some(matched) => {
                range = (matched.start(), matched.end());
            },
            None => break,
        }
        match parse(formula.drain(range.0+1..range.1-1).collect(), nof_stacks, instructions) {
            Ok(result) => id = result,
            Err(e) => return Err(e),
        }
        formula.replace_range(range.0..range.0+2, &format!("S{}", id));
    }
    generate_instructions(&mut formula, instructions, nof_stacks, unsafe { POWER.as_ref().unwrap() }, |a, b| a.powf(b), Operation::Power);
    generate_instructions(&mut formula, instructions, nof_stacks, unsafe { MULTIPLE.as_ref().unwrap() }, |a, b| a * b, Operation::Multiple);
    generate_instructions(&mut formula, instructions, nof_stacks, unsafe { DIVIDE.as_ref().unwrap() }, |a, b| a / b, Operation::Divide);
    generate_instructions(&mut formula, instructions, nof_stacks, unsafe { MODULUS.as_ref().unwrap() }, |a, b| a % b, Operation::Modulus);
    generate_instructions(&mut formula, instructions, nof_stacks, unsafe { ADD.as_ref().unwrap() }, |a, b| a + b, Operation::Add);
    match unsafe { UNARY.as_ref().unwrap().captures(&formula) } {
        Some(matched) => {
            match parse_operand(&format!("{}{}", &matched[1], &matched[2])) {
                Ok(operand) => {
                    match operand {
                        Operand::Decimal(sign, value) => {
                            let id = *nof_stacks;
                            instructions.push(Instruction {
                                operand_left: Operand::Decimal(sign, value),
                                operand_right: Operand::Decimal(true, 0.0),
                                operation: Operation::Add,
                                out: id,
                            });
                            *nof_stacks += 1;
                            Ok(id)
                        },
                        Operand::Stack(_, id) => Ok(id),
                        Operand::X(sign) => {
                            let id = *nof_stacks;
                            instructions.push(Instruction {
                                operand_left: Operand::X(sign),
                                operand_right: Operand::Decimal(true, 0.0),
                                operation: Operation::Add,
                                out: id,
                            });
                            *nof_stacks += 1;
                            Ok(id)
                        },
                        Operand::Y(sign) => {
                            let id = *nof_stacks;
                            instructions.push(Instruction {
                                operand_left: Operand::Y(sign),
                                operand_right: Operand::Decimal(true, 0.0),
                                operation: Operation::Add,
                                out: id,
                            });
                            *nof_stacks += 1;
                            Ok(id)
                        },
                    }
                },
                Err(e) => Err(e),
            }
        },
        None => Err(NO_OPERAND_FOUND),
    }
}

fn parse_operand(operand: &str) -> Result<Operand, &'static str> {
    match operand {
        "x" => Ok(Operand::X(true)),
        "-x" => Ok(Operand::X(false)),
        "y" => Ok(Operand::Y(true)),
        "-y" => Ok(Operand::Y(false)),
        _ => {
            let sign = &operand[0..1] != "-";
            let operand = if sign {
                operand
            } else {
                &operand[1..]
            };
            if &operand[0..1] == "S" {
                match operand[1..].parse() {
                    Ok(id) => Ok(Operand::Stack(sign, id)),
                    Err(_) => Err(INVALID_STACK),
                }
            } else {
                match operand.parse::<f64>() {
                    Ok(value) => Ok(Operand::Decimal(sign, value)),
                    Err(_) => Err(INVALID_OPERAND),
                }
            }
        },
    }
}

fn operand_into_decimal(stacks: &Vec<f64>, operand: &Operand, x: f64, y: f64) -> f64 {
    match *operand {
        Operand::Decimal(sign, value) => {
            if sign {
                value
            } else {
                - value
            }
        },
        Operand::Stack(sign, id) => {
            if sign {
                stacks[id]
            } else {
                - stacks[id]
            }
        },
        Operand::X(sign) => {
            if sign {
                x
            } else {
                - x
            }
        },
        Operand::Y(sign) => {
            if sign {
                y
            } else {
                - y
            }
        },
    }
}

fn operand_into_signed_decimal(stacks: &Vec<f64>, operand: &Operand, x: f64, y: f64) -> (bool, f64) {
    match *operand {
        Operand::Decimal(sign, value) => (sign, value),
        Operand::Stack(sign, id) => (sign, stacks[id]),
        Operand::X(sign) => (sign, x),
        Operand::Y(sign) => (sign, y),
    }
}

fn generate_instructions<F: Fn(f64, f64) -> f64>(formula: &mut String, instructions: &mut Vec<Instruction>, nof_stacks: &mut usize, regex: &Regex, closure: F, operation: Operation) {
    loop {
        let range;
        let operands;
        let left_sign;
        match regex.captures(&formula) {
            Some(matched) => {
                let whole = matched.get(0).unwrap();
                let right_operand_sign = match matched.get(3) {
                    Some(value) => value.as_str(),
                    None => "",
                };
                range = whole.start()..whole.end();
                left_sign = &matched[1] != "-";
                operands = (parse_operand(&format!("{}{}", &matched[1], &matched[2])).unwrap(), parse_operand(&format!("{}{}", right_operand_sign, &matched[4])).unwrap());
            },
            None => break,
        }
        if let Operand::Decimal(left_sign, left_value) = operands.0 {
            if let Operand::Decimal(right_sign, right_value) = operands.1 {
                let result = closure(if left_sign { left_value } else { - left_value }, if right_sign { right_value } else { - right_value });
                if left_sign {
                    formula.replace_range(range, &result.to_string());
                } else {
                    formula.replace_range(range, &format!("+ {}", result.to_string()));
                }
                continue;
            }
        }
        instructions.push(Instruction {
            operand_left: operands.0,
            operand_right: operands.1,
            operation: operation.clone(),
            out: *nof_stacks,
        });
        if left_sign {
            formula.replace_range(range, &format!("S{}", nof_stacks));
        } else {
            formula.replace_range(range, &format!("+ S{}", nof_stacks));
        }
        *nof_stacks += 1;
    }
}

#[cfg(test)]
mod tests {
    use formula::Side;
    #[test]
    fn init_formula() {
        let side = Side::new("x -  2 * x").unwrap();
        println!("{:?}\nResult: {}", side, side.calc(10.0, 10.0).unwrap());
    }
}
