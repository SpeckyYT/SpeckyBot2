use std::sync::{LazyLock, Mutex};
#[deny(static_mut_refs)]

use std::collections::HashMap;

use arraydeque::{ArrayDeque, Wrapping};
use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode};
use serde::{Deserialize, Serialize};
use small_hash_map::SmallHashMap;

use crate::{CARDS, SpeckyCard};

type UserId = u64;

const DB_PATH: &'static str = "assets/test.db";
const USERS_KEYSPACE: &str = "users";

const USERS_CACHE_SIZE: usize = 3;

pub static INV_HANDLER: LazyLock<Mutex<InventoryHandler>> = LazyLock::new(
    || Mutex::new(InventoryHandler::new(DB_PATH.to_string()).unwrap())
);

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub user_id: UserId,
    pub cards: HashMap<u16, CardData>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardData {
    id: u16,
    xp: u16,
    level: usize,

    #[serde(skip)]
    card_ref: Option<&'static SpeckyCard>,
}

pub struct CardOpResult {
    result_type: CardOpResultType,
    fjall_result: Option<fjall::Result<()>>,
}

impl CardOpResult {

    pub fn invalid_card_id(id: u16) -> CardOpResult {
        CardOpResult { result_type: CardOpResultType::InvalidCardID { id }, fjall_result: None }
    }

    pub fn card_already_present(id: u16) -> CardOpResult {
        CardOpResult { result_type: CardOpResultType::CardAlreadyPresent { id }, fjall_result: None }
    }

    pub fn card_missing(id: u16) -> CardOpResult {
        CardOpResult { result_type: CardOpResultType::CardMissing { id }, fjall_result: None }
    }

}

// will be used for printing messages
#[derive(Debug)]
pub enum CardOpResultType {
    // The used card ID does not correspond to a valid card
    InvalidCardID { id: u16 },

    // For when the player finds new cards
    CardAlreadyPresent { id: u16 },
    RegisteredCard { id: u16 },

    // Modifying cards
    CardMissing { id: u16 },
    AddedXP { id: u16, added_xp: u16 },
    AddedXPAndLevel { id: u16, added_xp: u16, new_level: usize },

    // other result types
    InvalidRead,
}

impl CardData {

    pub fn of(id: u16, xp: u16) -> Option<CardData> {
        let mut temp = CardData {
            id, xp, level: 0,
            card_ref: CARDS.by_id(id)
        };

        if let Some(actual_level) = temp.level_from_xp(xp) {
            temp.level = actual_level;
            Some(temp)
        }
        else { None }
    }

    // lazy card reference
    pub fn card(&mut self) -> Option<&'static SpeckyCard> {
        match self.card_ref {
            Some(_) => self.card_ref,
            None => {
                self.card_ref = CARDS.by_id(self.id);
                self.card_ref
            }
        }
    }

    pub fn xp(&self) -> u16 {
        self.xp
    }

    pub fn level(&self) -> usize {
        self.level
    }

    fn level_from_xp(&mut self, xp: u16) -> Option<usize> {
        let levels = crate::levels(&self.card()?.rarity);

        Some(levels.iter()
            .rposition(|&level_xp| level_xp < xp)
            .map(|i| i + 1)
            .unwrap_or(1))
    }

    fn add_amount(&mut self, amount: u16) -> CardOpResultType {
        self.xp += amount;

        if let Some(updated_level) = self.level_from_xp(self.xp) {       
            if updated_level > self.level {
                self.level = updated_level;

                CardOpResultType::AddedXPAndLevel { id: self.id, added_xp: amount, new_level: updated_level }
            }
            else { CardOpResultType::AddedXP { id: self.id, added_xp: amount } }
        }
        else { CardOpResultType::InvalidCardID { id: self.id } }
    }
}

impl Inventory {

    pub fn new(user_id: UserId) -> Inventory {
        Inventory { user_id, cards: HashMap::new() }
    }

    pub fn by_id(&self, card_id: u16) -> Option<&CardData> {
        self.cards.get(&card_id)
    }

    pub fn load(&mut self, syncer: &Syncer) -> fjall::Result<()> {
        syncer.read(self)
    }

    fn register_card(&mut self, syncer: *const Syncer, id: u16) -> CardOpResult {
        if !self.cards.contains_key(&id) {
            // adds card data with level 1 and 1 xp
            if let Some(card) = CARDS.by_id(id) {
                // instantiates the CardData and adds it
                self.cards.insert(id, CardData { id, xp: 1, level: 1, card_ref: Some(card) });

                let sync_result = unsafe { &*syncer }.write(self);
                CardOpResult { result_type: CardOpResultType::RegisteredCard { id }, fjall_result: Some(sync_result) }
            }
            else { CardOpResult::invalid_card_id(id) }
        }
        else { CardOpResult::card_already_present(id) }
    }

    fn add_amount(&mut self, syncer: *const Syncer, id: u16, amount: u16) -> CardOpResult {
        if let Some(card_data) = self.cards.get_mut(&id) {
            let result_type = card_data.add_amount(amount);

            let sync_result = unsafe { &*syncer }.write(self);
            CardOpResult { result_type, fjall_result: Some(sync_result) }
        }
        else { CardOpResult { result_type: CardOpResultType::CardMissing { id }, fjall_result: None } }
    }

}

impl From<UserId> for Inventory {
    fn from(user_id: UserId) -> Self {
        Self {
            user_id,
            cards: Default::default(),
        }
    }
}

pub struct Syncer {
    db: Database,
    users: Keyspace,
}

impl Syncer {

    pub fn open(db_path: String) -> fjall::Result<Syncer> {
        let db = Database::builder(&db_path).open()?;
        let users = db.keyspace(USERS_KEYSPACE, KeyspaceCreateOptions::default)?;

        Ok(Syncer { db, users })
    }

    fn read(&self, inv: &mut Inventory) -> fjall::Result<()> {
        if let Some(inv_json_slice) = self.users.get(inv.user_id.to_string())? {
            if let Ok(inv_json) = serde_json::from_slice(inv_json_slice.as_slice()) {
                *inv = inv_json;
            }
        }

        Ok(())
    }

    fn write(&self, inv: &Inventory) -> fjall::Result<()> {
        let serialized_data = serde_json::to_string(inv).unwrap_or(String::new()); // per ora la mette vuota
        self.users.insert(inv.user_id.to_string(), serialized_data)?;

        self.db.persist(PersistMode::SyncAll)?;
        Ok(())
    }

}

pub struct InventoryHandler {
    syncer: Syncer,
    
    users_map: SmallHashMap<UserId, Inventory, USERS_CACHE_SIZE>,
    users_deque: ArrayDeque<UserId, USERS_CACHE_SIZE, Wrapping>,
}

impl InventoryHandler {
    
    pub fn new(db_path: String) -> fjall::Result<InventoryHandler> {
        Ok(InventoryHandler {
            syncer: Syncer::open(db_path)?, 
            users_map: SmallHashMap::new(),
            users_deque: ArrayDeque::new(),
        })
    }

    fn prepare_for_new_id(&mut self, user_id: &UserId) -> fjall::Result<()> {
        let cloned_user_id = user_id.clone();

        // aggiunge l'user id alla deque e pulisce la mappa nel caso
        if let Some(oldest_id) = self.users_deque.push_front(cloned_user_id) {
            self.users_map.remove(&oldest_id);
        }

        // si occupa della mappa
        let mut to_add_inv: Inventory = cloned_user_id.into();
        self.syncer.read(&mut to_add_inv)?;
        self.users_map.insert(cloned_user_id, to_add_inv);

        Ok(())
    }

    pub fn get_inv(&mut self, user_id: &UserId) -> fjall::Result<&mut Inventory> {
        if !self.users_map.contains_key(&user_id) {
            self.prepare_for_new_id(user_id)?;
        }

        Ok(self.users_map.get_mut(&user_id).expect("Always valid"))
    }

    pub fn register_card(&mut self, user_id: &UserId, id: u16) -> CardOpResult {
        let syncer_ptr = &raw const self.syncer;

        match self.get_inv(user_id) {
            Ok(inv) => inv.register_card(syncer_ptr, id),
            Err(fjall_error) => CardOpResult {
                result_type: CardOpResultType::InvalidRead,
                fjall_result: Some(Err(fjall_error))
            }
        }
    }

    pub fn add_amount(&mut self, user_id: &UserId, id: u16, amount: u16) -> CardOpResult {
        let syncer_ptr = &raw const self.syncer;

        match self.get_inv(user_id) {
            Ok(inv) => inv.add_amount(syncer_ptr, id, amount),
            Err(fjall_error) => CardOpResult {
                result_type: CardOpResultType::InvalidRead,
                fjall_result: Some(Err(fjall_error))
            }
        }
    }

}
