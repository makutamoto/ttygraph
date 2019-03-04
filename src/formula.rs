use std::sync::Once;
use regex::Regex;

const SYNTAX_ERROR: &str = "Formulae must conform this syntax: [Left] = [Right]";
const DIVIDED_BY_NEG: &str = "Numbers can't be powerd by negative values.";
const DIVIDED_BY_ZERO: &str = "Numbers can't be divided by 0.";
const NO_OPERAND_FOUND: &str = "No operand found.";
const INVALID_STACK: &str = "Invalid stack id.";
const INVALID_OPERAND: &str = "Invalid operand!";

#[derive(Debug, Clone)]
enum Function {
    Abs,
    Max,
    Min,
    Ln,
    Log,
    Log2,
    Log10,
    Root,
    Sqrt,
    Cbrt,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Ceil,
    Floor,
    Round,
}

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
    Function(Function, Vec<usize>),
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

    pub fn get_raw(self: &Self) -> &str {
        &self.raw
    }

    pub fn get_color_pair(self: &Self) -> i16 {
        self.color_pair
    }
}

static REGEX_INIT: Once = Once::new();
static mut UNARY: Option<Regex> = None;
static mut PARENTHESES: Option<Regex> = None;
static mut ARGUMENTS: Option<Regex> = None;
static mut POWER: Option<Regex> = None;
static mut MULTIPLE: Option<Regex> = None;
static mut DIVIDE: Option<Regex> = None;
static mut MODULUS: Option<Regex> = None;
static mut ADD: Option<Regex> = None;
impl Side {
    pub fn new(formula: &str) -> Result<Side, &str> {
        REGEX_INIT.call_once(|| {
            unsafe {
                UNARY = Some(Regex::new(r"(-?)\s*((?:[exy]|PI)|S*[\d\.]+)").unwrap());
                PARENTHESES = Some(Regex::new(r"([\w\d]*)\(([^\(\)]*)\)").unwrap());
                ARGUMENTS = Some(Regex::new(r"(?:\(.*?\)|[^(,]+)+").unwrap());
                POWER = Some(Regex::new(r"(-?)\s*((?:[exy]|PI)|S*[\d\.]+)\s*\^\s*(-?)\s*((?:[exy]|PI)|S*[\d\.]+)").unwrap());
                MULTIPLE = Some(Regex::new(r"(-?)\s*((?:[exy]|PI)|S*[\d\.]+)\s*\*\s*(-?)\s*((?:[exy]|PI)|S*[\d\.]+)").unwrap());
                DIVIDE = Some(Regex::new(r"(-?)\s*((?:[exy]|PI)|S*[\d\.]+)\s*/\s*(-?)\s*((?:[exy]|PI)|S*[\d\.]+)").unwrap());
                MODULUS = Some(Regex::new(r"(-?)\s*((?:[exy]|PI)|S*[\d\.]+)\s*%\s*(-?)\s*((?:[exy]|PI)|S*[\d\.]+)").unwrap());
                ADD = Some(Regex::new(r"(-?)\s*((?:[exy]|PI)|S*[\d\.]+)\s*(?:\+|(-))\s*((?:[exy]|PI)|S*[\d\.]+)").unwrap());
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
                Operation::Function(ref name, ref args) => {
                    match name {
                        Function::Abs => stacks[args[0]].abs(),
                        Function::Max => stacks[args[0]].max(stacks[args[1]]),
                        Function::Min => stacks[args[0]].min(stacks[args[1]]),
                        Function::Ln => stacks[args[0]].ln(),
                        Function::Log => stacks[args[1]].log(stacks[args[0]]),
                        Function::Log2 => stacks[args[0]].log2(),
                        Function::Log10 => stacks[args[0]].log10(),
                        Function::Root => stacks[args[1]].powf(1.0 / stacks[args[0]]),
                        Function::Sqrt => stacks[args[0]].sqrt(),
                        Function::Cbrt => stacks[args[0]].cbrt(),
                        Function::Sin => stacks[args[0]].sin(),
                        Function::Cos => stacks[args[0]].cos(),
                        Function::Tan => stacks[args[0]].tan(),
                        Function::Asin => stacks[args[0]].asin(),
                        Function::Acos => stacks[args[0]].acos(),
                        Function::Atan => stacks[args[0]].atan(),
                        Function::Sinh => stacks[args[0]].sinh(),
                        Function::Cosh => stacks[args[0]].cosh(),
                        Function::Tanh => stacks[args[0]].tanh(),
                        Function::Asinh => stacks[args[0]].asinh(),
                        Function::Acosh => stacks[args[0]].acosh(),
                        Function::Atanh => stacks[args[0]].atanh(),
                        Function::Ceil => stacks[args[0]].ceil(),
                        Function::Floor => stacks[args[0]].floor(),
                        Function::Round => stacks[args[0]].round(),
                    }
                },
            };
            stacks.insert(instruction.out, result);
        }
        Ok(stacks[stacks.len() - 1])
    }
}

fn parse(mut formula: String, nof_stacks: &mut usize, instructions: &mut Vec<Instruction>) -> Result<usize, &'static str> {
    loop {
        let fn_name;
        let range;
        let mut content: String;
        let id;
        match unsafe { PARENTHESES.as_ref().unwrap().captures(&formula) } {
            Some(captured) => {
                let content = captured.get(2).unwrap();
                range = (content.start(), content.end());
                fn_name = captured[1].to_string();
            },
            None => break,
        }
        content = formula.drain(range.0..range.1).collect();
        if fn_name.len() != 0 {
            let function = match fn_name.as_str() {
                "abs" => (Function::Abs, 1),
                "max" => (Function::Max, 2),
                "min" => (Function::Min, 2),
                "ln" => (Function::Ln, 1),
                "log" => (Function::Log, 2),
                "log2" => (Function::Log2, 1),
                "log10" => (Function::Log10, 1),
                "root" => (Function::Root, 2),
                "sqrt" => (Function::Sqrt, 1),
                "cbrt" => (Function::Cbrt, 1),
                "sin" => (Function::Sin, 1),
                "cos" => (Function::Cos, 1),
                "tan" => (Function::Tan, 1),
                "asin" => (Function::Asin, 1),
                "acos" => (Function::Acos, 1),
                "atan" => (Function::Atan, 1),
                "sinh" => (Function::Sinh, 1),
                "cosh" => (Function::Cosh, 1),
                "tanh" => (Function::Tanh, 1),
                "asinh" => (Function::Asinh, 1),
                "acosh" => (Function::Acosh, 1),
                "atanh" => (Function::Atanh, 1),
                "ceil" => (Function::Ceil, 1),
                "floor" => (Function::Floor, 1),
                "round" => (Function::Round, 1),
                _ => return Err("Unimplemented function."),
            };
            let mut args = Vec::new();
            loop {
                let range;
                match unsafe { ARGUMENTS.as_ref().unwrap().find(&content) } {
                    Some(matched) => range = (matched.start(), matched.end()),
                    None => break,
                }
                match parse(content.drain(range.0..range.1).collect(), nof_stacks, instructions) {
                    Ok(result) => args.push(result),
                    Err(e) => return Err(e),
                }
            }
            if args.len() < function.1 {
                return Err("Too few arguments.");
            } else if args.len() > function.1 {
                return Err("Too many arguments.");
            }
            instructions.push(Instruction {
                operand_left: Operand::Decimal(true, 0.0),
                operand_right: Operand::Decimal(true, 0.0),
                operation: Operation::Function(function.0, args),
                out: *nof_stacks,
            });
            formula.replace_range(range.0-1-fn_name.len()..range.0+1, &format!("S{}", *nof_stacks));
            *nof_stacks += 1;
        } else {
            match parse(content, nof_stacks, instructions) {
                Ok(result) => id = result,
                Err(e) => return Err(e),
            }
            formula.replace_range(range.0-1..range.0+1, &format!("S{}", id));
        }
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
                        Operand::Stack(sign, id) => {
                            let new_id = *nof_stacks;
                            instructions.push(Instruction {
                                operand_left: Operand::Stack(sign, id),
                                operand_right: Operand::Decimal(true, 0.0),
                                operation: Operation::Add,
                                out: new_id,
                            });
                            *nof_stacks += 1;
                            Ok(new_id)
                        },
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
        "PI" => Ok(Operand::Decimal(true, std::f64::consts::PI)),
        "-PI" => Ok(Operand::Decimal(false, std::f64::consts::PI)),
        "e" => Ok(Operand::Decimal(true, std::f64::consts::E)),
        "-e" => Ok(Operand::Decimal(false, std::f64::consts::E)),
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
                formula.replace_range(range, &result.to_string());
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
mod test {
    use formula::Formula;
    use std::f64;
    #[test]
    fn test() {
        let a = Formula::new("y = round(x) + sin(x - PI / 2)", 0).unwrap();
        println!("{:?}\n{}", a, a.right.calc(f64::consts::PI / 2.0, 0.0).unwrap());
    }
}
