pub struct Pokemon {
    pub number: PokemonNumber,
    name: PokemonName,
    types: PokemonTypes,
}

impl Pokemon {
    pub fn new(number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Self {
        Self {
            number,
            name,
            types,
        }
    }
}
#[derive(PartialEq)]
pub struct PokemonNumber(u16);

impl TryFrom<u16> for PokemonNumber {
    type Error = ();

    fn try_from(n: u16) -> Result<Self, Self::Error> {
        if n > 0 && n < 899 {
            Ok(Self(n))
        } else {
            Err(())
        }
    }
}

impl From<PokemonNumber> for u16 {
    fn from(n: PokemonNumber) -> Self {
        n.0
    }
}

impl Clone for PokemonNumber {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
pub struct PokemonName(String);

impl TryFrom<String> for PokemonName {
    type Error = ();

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if name.is_empty() {
            Err(())
        } else {
            Ok(Self(name))
        }
    }
}

impl From<PokemonName> for String {
    fn from(name: PokemonName) -> Self {
        name.0
    }
}

enum PokemonType {
    Electric,
    Fire,
}

impl TryFrom<String> for PokemonType {
    type Error = ();

    fn try_from(tipe: String) -> Result<Self, Self::Error> {
        match tipe.to_uppercase().as_str() {
            "ELECTRIC" => Ok(PokemonType::Electric),
            "FIRE" => Ok(PokemonType::Fire),
            _ => Err(()),
        }
    }
}

pub struct PokemonTypes(Vec<PokemonType>);

impl TryFrom<Vec<String>> for PokemonTypes {
    type Error = ();
    fn try_from(types: Vec<String>) -> Result<Self, Self::Error> {
        if types.is_empty() {
            return Err(());
        }

        let mut pokemon_types: Vec<PokemonType> = Vec::with_capacity(types.len());
        for tipe in types.into_iter() {
            match PokemonType::try_from(tipe) {
                Ok(pokemon_type) => pokemon_types.push(pokemon_type),
                _ => return Err(()),
            }
        }

        Ok(Self(pokemon_types))
    }
}
