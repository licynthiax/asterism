#![allow(unused)]
use crate::boxsy_info::*;
use crate::json::*;
use boxsy::*;

pub struct Ceptre<'t> {
    types: CeptreTypes<'t>,
}

pub struct CeptreTypes<'t> {
    player: &'t Type,
    tile: &'t Type,
    character: &'t Type,
    room: &'t Type,
    rsrc_id: &'t Type,
}

impl<'t> TryFrom<Program> for Ceptre<'t> {
    type Error = crate::Error<'t>;
    fn try_from(program: Program) -> Result<Self, Self::Error> {
        let builtins = get_builtins(&program);
        let types = CeptreTypes::get_types(&program);

        // honestly possible that we can just ignore these preds??
        let mut queries = Vec::new();
        for pred in program.header.preds.iter() {
            if let Some(Annote::Query(_)) = pred.annote {
                queries.push(pred);
            }
        }

        let mut integrations = Vec::new();
        for stage in program.stages.iter() {
            for rule in stage.body.iter() {
                if let Some(Annote::Integration(_)) = rule.annote {
                    integrations.push(rule);
                }
            }
        }

        for i in integrations {
            use std::collections::BTreeSet;
            if let Some(a) = &i.annote {
                let logics = a.get_logics();
                let lhs_terms: BTreeSet<&Predicate> = i
                    .lhs
                    .iter()
                    .filter_map(|a| queries.iter().find(|q| q.name == a.name).copied())
                    .collect();
                let rhs_terms: BTreeSet<&Predicate> = i
                    .rhs
                    .iter()
                    .filter_map(|a| queries.iter().find(|q| q.name == a.name).copied())
                    .collect();

                let preds = lhs_terms.union(&rhs_terms);
                println!("{}", i.name);
                for p in preds {
                    dbg!(&p.name);
                }
                // get the atoms in the statement
                // get the terms in the atoms
                // intersection of the types in annote with each
            }
        }

        // do something with initial stage/state
        todo!()
    }
}

impl From<Ceptre<'_>> for Game {
    #[allow(unused)]
    fn from(c: Ceptre) -> Self {
        todo!()
    }
}

/// identify builtin types
fn get_builtins(program: &'_ Program) -> Result<Vec<&'_ Builtin>, crate::Error<'_>> {
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
    let nat = nat.ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT))?;
    let s = s.ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT_SUCC))?;
    let z = z.ok_or(crate::Error::BuiltinNotFound(BuiltinTypes::NAT_ZERO))?;

    Ok(vec![nat, s, z])
}

impl<'t> CeptreTypes<'t> {
    /// match asterism types with the ceptre ones
    fn get_types(program: &'_ Program) -> Result<CeptreTypes<'_>, crate::Error<'_>> {
        // grab annotes on types. legally speaking data and syntheses could be one thing here but
        // i like maintaining the difference because rsrcs aren't like coherent things distinct
        // from the things theyre attached to (?) at least in boxsy
        let mut syntheses = Vec::new();
        let mut data = Vec::new();
        for ty in program.header.types.iter() {
            match ty.annote {
                Some(Annote::Synthesis(_)) => syntheses.push(ty),
                Some(Annote::Data(_)) => data.push(ty),
                _ => {}
            }
        }

        let mut player = None;
        let mut character = None;
        let mut tile = None;
        let mut room = None;
        for s in syntheses {
            if let Some(a) = s.annote.as_ref() {
                if a.get_logics() == &GameType::Player.associated_logics() {
                    player = Some(s);
                }
                if a.get_logics() == &GameType::Character.associated_logics() {
                    character = Some(s);
                }
                if a.get_logics() == &GameType::Tile.associated_logics() {
                    tile = Some(s);
                }
                if a.get_logics() == &GameType::Room.associated_logics() {
                    room = Some(s);
                }
            }
        }

        let mut rsrc_id = None;
        for d in data {
            if let Some(a) = d.annote.as_ref() {
                if a.get_logics() == &GameType::Rsrc.associated_logics() {
                    rsrc_id = Some(d);
                }
            }
        }

        let player = player.ok_or(crate::Error::TypeNotFound(GameType::Player))?;
        let character = character.ok_or(crate::Error::TypeNotFound(GameType::Character))?;
        let tile = tile.ok_or(crate::Error::TypeNotFound(GameType::Tile))?;
        let room = room.ok_or(crate::Error::TypeNotFound(GameType::Room))?;
        let rsrc_id = rsrc_id.ok_or(crate::Error::TypeNotFound(GameType::Rsrc))?;

        Ok(CeptreTypes {
            player,
            tile,
            character,
            room,
            rsrc_id,
        })
    }
}

impl Type {
    fn find_tp(&self, name: String) -> Option<&Tp> {
        self.tp.iter().find(|tp| tp.name == name)
    }
}
