use crate::convert::*;

#[derive(Debug)]
pub struct Builtins {
    pub nat: String,
    pub nat_zero: String,
    pub nat_succ: String,
}

/// identify builtin types
impl Builtins {
    pub fn new(program: &Program) -> Result<Self, crate::Error<'static>> {
        let nat = program
            .builtins
            .iter()
            .find(|b| b.builtin == BuiltinTypes::NAT);
        let s = program
            .builtins
            .iter()
            .find(|b| b.builtin == BuiltinTypes::NAT_SUCC);
        let z = program
            .builtins
            .iter()
            .find(|b| b.builtin == BuiltinTypes::NAT_ZERO);

        // we want all three or nothin
        let nat = nat
            .ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT))?
            .name
            .clone();
        let nat_zero = z
            .ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT_ZERO))?
            .name
            .clone();
        let nat_succ = s
            .ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT_SUCC))?
            .name
            .clone();

        Ok(Builtins {
            nat,
            nat_zero,
            nat_succ,
        })
    }
}
