use crate::convert::*;

#[derive(Debug)]
pub struct CeptreTypes {
    pub player: CType,
    pub character: CType,
    pub room: CType,
    pub link: CType,
    pub rsrc: CType,
    pub builtins: Builtins,
}

impl CeptreTypes {
    pub fn find_type(&self, s: &str) -> Option<&CType> {
        if self.player.name == s {
            Some(&self.player)
        } else if self.character.name == s {
            Some(&self.character)
        } else if self.room.name == s {
            Some(&self.room)
        } else if self.link.name == s {
            Some(&self.link)
        } else if self.rsrc.name == s {
            Some(&self.rsrc)
        } else {
            None
        }
    }

    /// match asterism types with the ceptre ones
    pub fn get_types<'a, 'b: 'a>(program: &'a Program) -> Result<CeptreTypes, crate::Error<'b>> {
        let builtins = Builtins::new(program)?;

        let mut syntheses = BTreeSet::new();
        let mut data = BTreeSet::new();

        for ty in program.header.types.iter() {
            match ty.annote {
                Some(Annote::Synthesis(_)) => {
                    syntheses.insert(ty);
                }
                Some(Annote::Data(_)) => {
                    data.insert(ty);
                }
                Some(Annote::SynthData(_)) => {
                    syntheses.insert(ty);
                    data.insert(ty);
                }
                _ => {}
            }
        }

        // find these types
        let mut game_types = [
            GameType::Player,
            GameType::Character,
            GameType::Room,
            GameType::Link,
            GameType::Rsrc,
        ];

        let mut ctypes = [None, None, None, None, None];

        for t in syntheses {
            let a = t.annote.as_ref().unwrap();
            for (i, gametype) in game_types.iter().enumerate() {
                if a.get_logics() == &gametype.associated_logics() {
                    ctypes[i] = Some(
                        (*t).clone()
                            .try_into()
                            .map_err(|_| crate::Error::TypeNotFound(*gametype))?,
                    );
                }
            }
        }

        for t in data.iter() {
            let a = t.annote.as_ref().unwrap();
            for (i, gametype) in game_types.iter().enumerate() {
                if ctypes[i].is_none() && a.get_logics() == &gametype.associated_logics() {
                    ctypes[i] = Some(
                        (*t).clone()
                            .try_into()
                            .map_err(|_| crate::Error::TypeNotFound(*gametype))?,
                    );
                }
            }
        }

        let mut ctypes: Vec<CType> = ctypes.into_iter().map(|ty| ty.unwrap()).collect();

        let find_data_types = |cty: &mut CType| -> Result<(), crate::Error<'b>> {
            let all_include = data
                .iter()
                .find(|d| cty.tp.iter().all(|tp| tp.args.iter().any(|a| d.name == *a)));
            if let Some(ty) = all_include {
                let data = (*ty).clone().try_into()?;
                cty.data = Some(Box::new(data));
            }
            Ok(())
        };

        for (gt, cty) in game_types.iter().zip(ctypes.iter_mut()) {
            find_data_types(cty)?;
        }

        Ok(CeptreTypes {
            builtins,
            player: ctypes[0].clone(),
            character: ctypes[1].clone(),
            room: ctypes[2].clone(),
            link: ctypes[3].clone(),
            rsrc: ctypes[4].clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CType {
    pub name: String,
    pub data: Option<Box<CType>>,
    pub tp: Vec<Tp>,
    pub annote: Annote,
}

impl TryFrom<Type> for CType {
    type Error = crate::Error<'static>;
    fn try_from(t: Type) -> Result<Self, Self::Error> {
        Ok(Self {
            name: t.name,
            data: None,
            tp: t.tp,
            annote: t.annote.ok_or(crate::Error::Custom("no annote"))?,
        })
    }
}

impl CType {
    pub fn find_tp(&self, name: &str) -> Option<(usize, &Tp)> {
        self.data.as_ref().and_then(|d| d.find_tp(name)).or(self
            .tp
            .iter()
            .enumerate()
            .find(|(_, tp)| tp.name == name))
    }
}
