use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct PropertyData {
    owner: Option<String>,
    price: u32,
    groupid: u32,
}

impl PropertyData {
    pub fn new(owner: Option<String>, price: u32, groupid: u32) -> Self {
        PropertyData {
            owner,
            price,
            groupid,
        }
    }

    pub fn owner(&self) -> &Option<String> {
        &self.owner
    }

    pub fn price(&self) -> u32 {
        self.price
    }

    pub fn groupid(&self) -> u32 {
        self.groupid
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RailroadData {
    owner: Option<String>,
    price: u32,
    groupid: u32,
}

impl RailroadData {
    pub fn owner(&self) -> &Option<String> {
        &self.owner
    }

    pub fn price(&self) -> u32 {
        self.price
    }

    pub fn groupid(&self) -> u32 {
        self.groupid
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FieldType {
    Property(PropertyData),
    Jail,
    Chance,
    Start,
    Railroad(RailroadData),
}

#[derive(Debug, Clone)]
pub(crate) struct UserData {
    cash: u32,
    jail: bool,
}

impl UserData {
    pub fn new(cash: u32, jail: bool) -> Self {
        Self { cash, jail }
    }

    pub fn cash(&self) -> u32 {
        self.cash
    }

    pub fn jail(&self) -> bool {
        self.jail
    }
}

#[derive(Debug)]
pub(crate) struct MemoryManager {
    field_map: HashMap<u32, FieldType>,
    user_map: HashMap<String, UserData>,
}

impl MemoryManager {
    pub(crate) fn new() -> Self {
        MemoryManager {
            field_map: HashMap::new(),
            user_map: HashMap::new(),
        }
    }

    pub(crate) fn insert_field(&mut self, key: u32, value: FieldType) {
        self.field_map.insert(key, value);
    }

    pub(crate) fn get_field(&self, key: u32) -> Option<&FieldType> {
        self.field_map.get(&key)
    }

    pub(crate) fn modify_field(&mut self, key: u32, value: FieldType) -> Result<(), ()> {
        use FieldType::*;

        let field = self.field_map.get_mut(&key);
        if field.is_none() {
            return Err(());
        }
        match (field.unwrap(), value) {
            (Railroad(ref mut data_ref), Railroad(new_data)) => {
                *data_ref = new_data;
                Ok(())
            }
            (Property(ref mut data_ref), Property(new_data)) => {
                *data_ref = new_data;
                Ok(())
            }
            (Chance, Chance) => Ok(()),
            (Start, Start) => Ok(()),
            (Jail, Jail) => Ok(()),
            _ => Err(()),
        }
    }

    pub(crate) fn insert_user(&mut self, key: String, value: UserData) {
        self.user_map.insert(key, value);
    }

    pub(crate) fn get_user(&self, key: &str) -> Option<&UserData> {
        self.user_map.get(key)
    }

    pub(crate) fn modify_user(&mut self, key: &str, value: UserData) -> Result<(), ()> {
        match (self.user_map.get_mut(key)) {
            None => Err(()),
            Some(data_ref) => {
                *data_ref = value;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_new() {
        let manager = MemoryManager::new();
        assert!(manager.field_map.is_empty());
        assert!(manager.user_map.is_empty());
    }

    #[test]
    fn test_insert_field() {
        let mut manager = MemoryManager::new();
        let property = FieldType::Property(PropertyData {
            owner: None,
            price: 100,
            groupid: 1,
        });
        manager.insert_field(1, property);
        assert!(manager.field_map.contains_key(&1));
    }

    #[test]
    fn test_get_field_existing() {
        let mut manager = MemoryManager::new();
        let property = FieldType::Property(PropertyData {
            owner: None,
            price: 100,
            groupid: 1,
        });
        manager.insert_field(1, property);
        assert!(manager.get_field(1).is_some());
    }

    #[test]
    fn test_get_field_non_existing() {
        let manager = MemoryManager::new();
        assert!(manager.get_field(1).is_none());
    }

    #[test]
    fn test_modify_field_success() {
        let mut manager = MemoryManager::new();
        let property = FieldType::Property(PropertyData {
            owner: None,
            price: 100,
            groupid: 1,
        });
        manager.insert_field(1, property);

        let new_property = FieldType::Property(PropertyData {
            owner: Some("Alice".to_string()),
            price: 150,
            groupid: 1,
        });
        assert!(manager.modify_field(1, new_property).is_ok());

        if let Some(FieldType::Property(PropertyData {
            owner: Some(ref name),
            price,
            ..
        })) = manager.get_field(1)
        {
            assert_eq!(name, "Alice");
            assert_eq!(*price, 150);
        } else {
            panic!("Field was not updated correctly.");
        }
    }

    #[test]
    fn test_modify_field_failure() {
        let mut manager = MemoryManager::new();
        let property = FieldType::Property(PropertyData {
            owner: None,
            price: 100,
            groupid: 1,
        });
        manager.insert_field(1, property);

        let railroad = FieldType::Railroad(RailroadData {
            owner: None,
            price: 200,
            groupid: 2,
        });
        assert!(manager.modify_field(1, railroad).is_err()); // Different field type
    }

    #[test]
    fn test_insert_user() {
        let mut manager = MemoryManager::new();
        let user_data = UserData {
            cash: 500,
            jail: false,
        };
        manager.insert_user("Alice".to_string(), user_data);
        assert!(manager.user_map.contains_key("Alice"));
    }

    #[test]
    fn test_get_user_existing() {
        let mut manager = MemoryManager::new();
        let user_data = UserData {
            cash: 500,
            jail: false,
        };
        manager.insert_user("Alice".to_string(), user_data);
        assert!(manager.get_user("Alice").is_some());
    }

    #[test]
    fn test_get_user_non_existing() {
        let manager = MemoryManager::new();
        assert!(manager.get_user("Alice").is_none());
    }
}
