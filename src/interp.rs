use super::*;

pub fn interp(e: &Exp, nv: &Env) -> Result<Value, String> {
    match e {
        Exp::Int(n) => Ok(Value::Int(*n)),
        Exp::Bool(b) => Ok(Value::Bool(*b)),
        Exp::Str(s) => Ok(Value::Str(s.clone())),

        Exp::Var(var) => match nv.get(var) {
            Some(v) => Ok(v.clone()),
            None => Err(format!("{var} not bound")),
        },

        Exp::Plus { left, right } => {
            let l_val = interp(left, nv)?;
            let r_val = interp(right, nv)?;
            match (l_val, r_val) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
                (l, r) => Err(format!(
                    "+ expects two integers, got {:?} + {:?}", // Use Debug format
                    l, r
                )),
            }
        }

        Exp::Concat { left, right } => {
            let l_val = interp(left, nv)?;
            let r_val = interp(right, nv)?;
            match (l_val, r_val) {
                (Value::Str(l), Value::Str(r)) => {
                    Ok(Value::Str(format!("{l}{r}")))
                }
                (l, r) => Err(format!(
                    "++ expects two strings, got {:?} ++ {:?}", // Match operator
                    l, r
                )),
            }
        }

        Exp::LessThan { left, right } => {
            let l_val = interp(left, nv)?;
            let r_val = interp(right, nv)?;
            match (l_val, r_val) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l < r)),
                (l, r) => Err(format!(
                    "< expects two integers, got {:?} < {:?}", // Match operator
                    l, r
                )),
            }
        }

        Exp::Cnd { tst, thn, els } => {
            let tst_val = interp(tst, nv)?;
            match tst_val {
                Value::Bool(true) => interp(thn, nv),
                Value::Bool(false) => interp(els, nv),
                v => Err(format!("boolean expected, found {:?}", v)), // Use Debug format
            }
        }

        Exp::Let1 { var, value, body } => {
            let val = interp(value, nv)?;
            let mut new_nv = nv.clone();
            new_nv.insert(var.clone(), val);
            interp(body, &new_nv)
        }

        // Corrected to match Exp::Lam definition in main.rs
        Exp::Lam { var, var_type, body } => Ok(Value::Fun {
            var: var.clone(),
            var_type: var_type.clone(), // Store type in closure
            body: body.clone(),
            nv: nv.clone(),
        }),

        Exp::App { fun, arg } => {
            let fun_val = interp(fun, nv)?;
            let arg_val = interp(arg, nv)?;

            match fun_val {
                // Corrected to match Value::Fun definition
                Value::Fun { var, body, nv: closure_nv, .. } => {
                    let mut new_nv = closure_nv.clone();
                    new_nv.insert(var, arg_val);
                    interp(&body, &new_nv)
                }
                v => Err(format!("function expected, found {:?}", v)), // Use Debug format
            }
        }
    }
}

// Type Checker function
pub fn tc(e: &Exp, tnv: &TEnv) -> Result<Type, String> {
    match e {
        Exp::Int(_) => Ok(Type::Int),
        Exp::Bool(_) => Ok(Type::Bool),
        Exp::Str(_) => Ok(Type::Str),

        Exp::Var(var) => match tnv.get(var) {
            Some(t) => Ok(t.clone()),
            None => Err(format!("no known type for {var}")),
        },

        Exp::Plus { left, right } => {
            let l_type = tc(left, tnv)?;
            let r_type = tc(right, tnv)?;
            match (&l_type, &r_type) {
                (Type::Int, Type::Int) => Ok(Type::Int),
                _ => Err("not both integers".to_string()),
            }
        }

        Exp::Concat { left, right } => {
            let l_type = tc(left, tnv)?;
            let r_type = tc(right, tnv)?;
            match (&l_type, &r_type) {
                // This is the line that was cut off:
                (Type::Str, Type::Str) => Ok(Type::Str),
                _ => Err("not both strings".to_string()),
            }
        }

        Exp::LessThan { left, right } => {
            let l_type = tc(left, tnv)?;
            let r_type = tc(right, tnv)?;
            match (&l_type, &r_type) {
                (Type::Int, Type::Int) => Ok(Type::Bool),
                _ => Err("not both numbers".to_string()),
            }
        }

        Exp::Cnd { tst, thn, els } => {
            let tst_type = tc(tst, tnv)?;
            if tst_type != Type::Bool {
                return Err("condition must be a bool".to_string());
            }
            let thn_type = tc(thn, tnv)?;
            let els_type = tc(els, tnv)?;
            if thn_type == els_type {
                Ok(thn_type)
            } else {
                Err("then and else branches have different types".to_string())
            }
        }

        Exp::Let1 { var, value, body } => {
            let val_type = tc(value, tnv)?;
            let mut new_tnv = tnv.clone();
            new_tnv.insert(var.clone(), val_type);
            tc(body, &new_tnv)
        }

        Exp::Lam { var, var_type, body } => {
            let mut new_tnv = tnv.clone();
            new_tnv.insert(var.clone(), var_type.clone());
            let body_type = tc(body, &new_tnv)?;
            Ok(Type::Fun {
                param: Box::new(var_type.clone()),
                result: Box::new(body_type),
            })
        }

        Exp::App { fun, arg } => {
            let fun_type = tc(fun, tnv)?;
            let arg_type = tc(arg, tnv)?;
            match fun_type {
                Type::Fun { param, result } => {
                    if *param == arg_type {
                        Ok(*result)
                    } else {
                        Err(format!(
                            "function argument type mismatch: expected {param}, got {arg_type}"
                        ))
                    }
                }
                _ => Err(format!("function expected, found {fun_type}")),
            }
        }
    }
}
