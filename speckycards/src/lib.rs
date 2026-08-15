use std::{collections::HashMap, fs, sync::LazyLock};

use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{EnumIter, IntoEnumIterator};

pub static CARDS: LazyLock<ResultCardCollection> = LazyLock::new(|| {
    ResultCardCollection::read_from_json("assets/cards.json").expect("Cards are always valid")
});

pub mod userdata;

pub const fn levels(rarity: &Rarity) -> &'static [u16] {
    match rarity {
        Rarity::Common => &[2, 5, 15, 50, 100, 250, 600, 1500, 4000],
        Rarity::Rare => &[2, 5, 15, 50, 100, 250, 600, 1500, 4000],
        Rarity::Epic => &[2, 5, 15, 50, 100, 250, 600, 1500, 4000],
        Rarity::Legendary => &[2, 5, 10, 20, 60, 110],
        Rarity::Invalid => &[]
    }
}

#[derive(Debug, Copy, Clone, Serialize_repr, Deserialize_repr, Eq, Hash, PartialEq, EnumIter)]
#[repr(u8)]
pub enum Rarity {
    Common = 0,
    Rare = 1,
    Epic = 2,
    Legendary = 3,

    Invalid = 255,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResultCardCollection {
    #[serde(with = "cards_as_vec")]
    pub cards: HashMap<u16, SpeckyCard>,
}

mod cards_as_vec {
    use super::SpeckyCard;
    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserializer, Serializer,
    };
    use std::{collections::HashMap, fmt};

    pub fn serialize<S>(
        cards: &HashMap<u16, SpeckyCard>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(cards.len()))?;

        for card in cards.values() {
            seq.serialize_element(card)?;
        }

        seq.end()
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<u16, SpeckyCard>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CardsVisitor;

        impl<'de> Visitor<'de> for CardsVisitor {
            type Value = HashMap<u16, SpeckyCard>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of SpeckyCard")
            }

            fn visit_seq<A>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut cards =
                    HashMap::with_capacity(seq.size_hint().unwrap_or(0));

                while let Some(card) = seq.next_element::<SpeckyCard>()? {
                    cards.insert(card.id, card);
                }

                Ok(cards)
            }
        }

        deserializer.deserialize_seq(CardsVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SpeckyCard {
    id: u16,
    rarity: Rarity,

    #[serde(flatten)]
    res: ResultType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ResultType {
    Text(String),
    Image {
        text: String,
        image: String,
    },
    ImageOnly(String)
}

impl Rarity {
    pub fn text(&self) -> &str {
        match &self {
            Rarity::Common => "Comune",
            Rarity::Rare => "Rara",
            Rarity::Epic => "Epica",
            Rarity::Legendary => "Leggendaria",
            Rarity::Invalid => "Invalid",
        }
    }

    pub fn weight(&self) -> f64 {
        match &self {
            Rarity::Common => 50.0,
            Rarity::Rare => 13.0,
            Rarity::Epic => 5.0,
            Rarity::Legendary => 1.0,
            Rarity::Invalid => 0.0,
        }
    }

    pub fn sum_weights() -> f64 {
        Rarity::iter().map(|r| r.weight()).sum()
    }

    pub fn pick_random() -> Rarity {
        let score: f64 = rand::random_range(0.0..Rarity::sum_weights());

        Rarity::iter().fold((0.0, None), |sum, next| {
            let new_sum = sum.0 + next.weight();
            (new_sum, sum.1.or((score <= new_sum).then_some(next)))
        }).1
        .unwrap_or(Rarity::Invalid)
    }
}

impl ResultCardCollection {
    pub fn new() -> ResultCardCollection { 
        ResultCardCollection { cards: HashMap::new() }
    }

    pub fn by_id(&self, id: u16) -> Option<&SpeckyCard> {
        self.cards.get(&id)
    }

    pub fn has_id(&self, id: u16) -> bool {
        self.cards.contains_key(&id)
    }

    pub fn add_card(&mut self, card: SpeckyCard) {
        self.cards.insert(card.id, card);
    }

    pub fn read_from_json(file: &str) -> Option<ResultCardCollection> {
        let xml = fs::read_to_string(file).ok()?;
        serde_json::from_str(&xml).ok()
    }

    pub fn get_random(&self, rarity: Rarity) -> Option<&SpeckyCard> {
        let mut rng = rand::rng();
        self.cards.values()
            .filter(|card| card.rarity == rarity)
            .choose(&mut rng)
    }
}

impl SpeckyCard {
    pub fn of_text_c(id: u16, text: &str) -> SpeckyCard { SpeckyCard::of_text(id, Rarity::Common, text) }
    
    pub fn of_text(id: u16, rarity: Rarity, text: &str) -> SpeckyCard { 
        SpeckyCard { id, rarity, res: ResultType::Text(String::from(text)) } 
    }

    pub fn id(&self) -> &u16 { &self.id }
    pub fn rarity(&self) -> &Rarity { &self.rarity }
    pub fn res(&self) -> &ResultType { &self.res }}

#[test]
fn test_card() {
    assert_eq!(CARDS.cards.get(&52), Some(&SpeckyCard { id: 52, rarity: Rarity::Rare, res: ResultType::Text("Specky is NOT gay".to_string()) }));
}
