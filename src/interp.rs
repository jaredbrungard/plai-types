use super::*;

pub fn interp(e: &Exp, nv: &Env) -> Result<Value, String> {
    match e {
        Exp::Int(n) => Ok(Value::Int(*n)),

        Exp::Bool(b) => Ok(Value::Bool(*b)),

        Exp::Var(s) => match nv.get(s) {
            Some(v) => Ok(v.clone()),
            None => Err(format!("{s} not bound")), // Corrected error message
        },

        Exp::Plus { left, right } => add(interp(left, nv)?, interp(right, nv)?),

        Exp::Cnd { tst, thn, els } => {
            if boolean_decision(interp(tst, nv)?)? {
                interp(thn, nv)
            } else {
                interp(els, nv)
            }
        }

        Exp::Let1 { var, value, body } => {
            let val = interp(value, nv)?;
            let mut new_nv = nv.clone();
            new_nv.insert(var.clone(), val);
            interp(body, &new_nv)
        }

        Exp::Lam { var, body } => Ok(Value::Fun {
            var: var.clone(),
            body: body.clone(),
            nv: nv.clone(),
        }),

        Exp::LessThan { left, right } => {
            let left_val = interp(left, nv)?;
            let right_val = interp(right, nv)?;
            if let (Value::Int(n1), Value::Int(n2)) = (&left_val, &right_val) {
                Ok(Value::Bool(*n1 < *n2))
            } else {
                Err(format!(
                    "< expects two integers, got {:?} and {:?}",
                    left_val, right_val
                ))
            }
        }

        Exp::App { fun, arg } => {
            let fun_val = interp(fun, nv)?;
            let arg_val = interp(arg, nv)?;

            if let Value::Fun { var, body, nv: fun_nv } = fun_val {
                let mut new_nv = fun_nv;
                new_nv.insert(var, arg_val);
                interp(&body, &new_nv)
            } else {
                Err("Expected a function in application".to_string())
            }
        }
    }
}

fn add(v1: Value, v2: Value) -> Result<Value, String> {
    if let (Value::Int(n1), Value::Int(n2)) = (&v1, &v2) {
        Ok(Value::Int(n1 + n2))
    } else {
        Err(format!("+ expects two integers, got {:?} + {:?}", v1, v2))
    }
}

fn boolean_decision(v: Value) -> Result<bool, String> {
    if let Value::Bool(b) = v {
        Ok(b)
    } else {
        Err(format!("boolean expected, found {:?}", v))
    }
}
