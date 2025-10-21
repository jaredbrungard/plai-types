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
                    "+ expects two strings, got {:?} + {:?}", // Use Debug format AND transcript's typo
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
                    "< expects two integers, got {:?} + {:?}", // Use Debug format
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

        Exp::Lam { var, body } => Ok(Value::Fun {
            var: var.clone(),
            body: body.clone(),
            nv: nv.clone(),
        }),

        Exp::App { fun, arg } => {
            let fun_val = interp(fun, nv)?;
            let arg_val = interp(arg, nv)?;

            match fun_val {
                Value::Fun { var, body, nv: closure_nv } => {
                    let mut new_nv = closure_nv.clone();
                    new_nv.insert(var, arg_val);
                    interp(&body, &new_nv)
                }
                v => Err(format!("function expected, found {:?}", v)), // Use Debug format
            }
        }
    }
}
