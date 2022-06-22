
#[derive(Clone)]
pub struct Pokemon {
    pub number: PokemonNumber,
    pub name: PokemonName,
    pub types: PokemonTypes,
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

#[derive(Clone)]
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


#[derive(Clone)]
enum PokemonType {
    Electric,
    Fire,
}

impl TryFrom<String> for PokemonType {
    type Error = ();

    fn try_from(tipe: String) -> Result<Self, Self::Error> {
        match tipe.as_str() {
            "Electric" => Ok(PokemonType::Electric),
            "Fire" => Ok(PokemonType::Fire),
            _ => Err(()),
        }
    }
}

impl From<PokemonType> for String {
    fn from(tipe: PokemonType) -> Self {
        match tipe {
            PokemonType::Electric => "Electric".to_owned(),
            PokemonType::Fire => "Fire".to_owned(),
        }
    }
}

#[derive(Clone)]
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

impl From<PokemonTypes> for Vec::<String> {
    fn from(types: PokemonTypes) -> Self {
        types.0.into_iter().map(String::from).collect()
    }
}
