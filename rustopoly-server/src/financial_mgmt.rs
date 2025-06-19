use crate::memory_mgmt::{FieldType, MemoryManager, PropertyData, UserData};
use thiserror::Error;

/// Errors that may occur during financial operations in the game.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FinancialManagerError {
    // === User-Related ===
    #[error("Player `{0}` not found")]
    PlayerNotFound(String),

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u32, available: u32 },

    #[error("Player `{0}` does not own field `{1}`")]
    NotOwnerOfField(String, u32),

    #[error("Player `{0}` cannot receive property `{1}`")]
    InvalidRecipient(String, u32),

    // === Field/Property Related ===
    #[error("Field `{0}` not found")]
    FieldNotFound(u32),

    #[error("Field `{0}` is not buyable")]
    FieldNotBuyable(u32),

    #[error("Field `{0}` is not sellable")]
    FieldNotSellable(u32),

    #[error("Field `{0}` is already owned")]
    PropertyAlreadyOwned(u32),

    #[error("Field `{0}` has no owner")]
    FieldHasNoOwner(u32),

    // === Action Errors ===
    #[error("Failed to complete transaction for player `{0}`")]
    TransactionFailed(String),

    #[error("Bankruptcy process failed for player `{0}`")]
    BankruptcyFailed(String),

    #[error("Transfer from `{0}` to `{1}` of `{2}` failed")]
    TransferFailed(String, String, u32),

    // === Catch-all ===
    #[error("Internal error")]
    InternalError,
}


/// FinancialManager
/// Acts as a centralized manager for handling financial actions,
/// including player account management and property transactions.
#[derive(Debug)]
pub struct FinancialManager<'a> {
    memory_manager: &'a mut MemoryManager,
}

impl<'a> FinancialManager<'a> {
    /// Create a new FinancialManager with access to MemoryManager.
    pub fn new(memory_manager: &'a mut MemoryManager) -> Self {
        FinancialManager { memory_manager }
    }

    /// Obtain an instance of AccountHandler for player account actions.
    pub fn accounts(&mut self) -> AccountHandler {
        AccountHandler {
            memory_manager: self.memory_manager,
        }
    }

    /// Obtain an instance of PropertyHandler for property-related actions.
    pub fn properties(&mut self) -> PropertyHandler {
        PropertyHandler {
            memory_manager: self.memory_manager,
        }
    }
}

// -----------------------------------
// Account Handler - handling player cash balances and transactions
// -----------------------------------
#[derive(Debug)]
pub struct AccountHandler<'a> {
    memory_manager: &'a mut MemoryManager,
}

impl<'a> AccountHandler<'a> {
    
    /// Retrieves the current cash balance of the given player.
    pub fn get_balance(&self, player_id: &str) -> Option<u32> {
        self.memory_manager.get_user(player_id).map(|u| u.cash())
    }

    /// Adds a specified amount of money to the player's balance.
    /// Returns an error if the player does not exist or if the modification fails.
    pub fn deposit(&mut self, player_id: &str, amount: u32) -> Result<(), FinancialManagerError> {
        let user = self
            .memory_manager
            .get_user(player_id)
            .ok_or_else(|| FinancialManagerError::PlayerNotFound(player_id.to_string()))?;

        let updated_user = UserData::new(user.cash() + amount, user.jail());

        self.memory_manager
            .modify_user(player_id, updated_user)
            .map_err(|_| FinancialManagerError::TransactionFailed(player_id.to_string()))
    }

    /// Deducts a specified amount of money from the player's balance.
    /// Returns an error if the player does not exist, has insufficient funds,
    /// or if the modification fails.
    pub fn withdraw(&mut self, player_id: &str, amount: u32) -> Result<(), FinancialManagerError> {
        let user = self
            .memory_manager
            .get_user(player_id)
            .ok_or_else(|| FinancialManagerError::PlayerNotFound(player_id.to_string()))?;

        if user.cash() < amount {
            return Err(FinancialManagerError::InsufficientFunds {
                required: amount,
                available: user.cash(),
            });
        }

        let updated_user = UserData::new(user.cash() - amount, user.jail());

        self.memory_manager
            .modify_user(player_id, updated_user)
            .map_err(|_| FinancialManagerError::TransactionFailed(player_id.to_string()))
    }

    /// Transfers funds between two players.
    /// Returns an error if either player is not found, or the transfer fails.
    pub fn transfer(
        &mut self,
        from_player: &str,
        to_player: &str,
        amount: u32,
    ) -> Result<(), FinancialManagerError> {
        self.withdraw(from_player, amount)
            .map_err(|_| FinancialManagerError::TransferFailed(from_player.to_string(), to_player.to_string(), amount))?;
        self.deposit(to_player, amount)
            .map_err(|_| FinancialManagerError::TransferFailed(from_player.to_string(), to_player.to_string(), amount))?;
        Ok(())
    }

    /// Sets the player's balance to zero and marks them as no longer in jail.
    /// Returns an error if the user is not found or modification fails.
    pub fn declare_bankruptcy(&mut self, player_id: &str) -> Result<(), FinancialManagerError> {
        self.memory_manager
            .get_user(player_id)
            .ok_or_else(|| FinancialManagerError::PlayerNotFound(player_id.to_string()))?;

        let updated_user = UserData::new(0, false);
        self.memory_manager
            .modify_user(player_id, updated_user)
            .map_err(|_| FinancialManagerError::BankruptcyFailed(player_id.to_string()))
    }
}


// -----------------------------------
// Property Handler - property ownership and transactions
// -----------------------------------
#[derive(Debug)]
pub struct PropertyHandler<'a> {
    memory_manager: &'a mut MemoryManager,
}

impl<'a> PropertyHandler<'a> {
    
    /// Returns the name of the current owner of the given field, if any.
    pub fn get_owner(&self, field_id: u32) -> Option<String> {
        match self.memory_manager.get_field(field_id) {
            Some(FieldType::Property(data)) => data.owner().clone(),
            Some(FieldType::Railroad(data)) => data.owner().clone(),
            _ => None,
        }
    }

    /// Attempts to purchase a property.
    /// Validates ownership, player funds, and field type.
    pub fn buy_property(
        &mut self,
        player_id: &str,
        field_id: u32,
    ) -> Result<(), FinancialManagerError> {
        // Step 1: Extract field data immutably
        let (price, groupid) = match self.memory_manager.get_field(field_id) {
            Some(FieldType::Property(data)) => {
                if data.owner().is_some() {
                    return Err(FinancialManagerError::PropertyAlreadyOwned(field_id));
                }
                (data.price(), data.groupid())
            }
            Some(_) => return Err(FinancialManagerError::FieldNotBuyable(field_id)),
            None => return Err(FinancialManagerError::FieldNotFound(field_id)),
        };
    
        // Step 2: Extract user values immutably
        let (cash, jail) = self
            .memory_manager
            .get_user(player_id)
            .map(|u| (u.cash(), u.jail()))
            .ok_or_else(|| FinancialManagerError::PlayerNotFound(player_id.to_string()))?;
    
        if cash < price {
            return Err(FinancialManagerError::InsufficientFunds {
                required: price,
                available: cash,
            });
        }
    
        // Step 3: Proceed with mutation — safe now
        let updated_field = FieldType::Property(PropertyData::new(
            Some(player_id.to_string()),
            price,
            groupid,
        ));
        self.memory_manager
            .modify_field(field_id, updated_field)
            .map_err(|_| FinancialManagerError::TransactionFailed(player_id.to_string()))?;
    
        let updated_user = UserData::new(cash - price, jail);
        self.memory_manager
            .modify_user(player_id, updated_user)
            .map_err(|_| FinancialManagerError::TransactionFailed(player_id.to_string()))?;
    
        Ok(())
    }
    
    

    /// Sells a property and credits the player with the sell price.
    /// Returns an error if the player doesn't own the property or if the field type is invalid.
    pub fn sell_property(
        &mut self,
        player_id: &str,
        field_id: u32,
        sell_price: u32,
    ) -> Result<(), FinancialManagerError> {
        let field = self
            .memory_manager
            .get_field(field_id)
            .ok_or(FinancialManagerError::FieldNotFound(field_id))?;

        if let FieldType::Property(data) = field {
            if data.owner().as_deref() != Some(player_id) {
                return Err(FinancialManagerError::NotOwnerOfField(
                    player_id.to_string(),
                    field_id,
                ));
            }

            let updated_field =
                FieldType::Property(PropertyData::new(None, data.price(), data.groupid()));

            self.memory_manager
                .modify_field(field_id, updated_field)
                .map_err(|_| FinancialManagerError::TransactionFailed(player_id.to_string()))?;

            let user = self
                .memory_manager
                .get_user(player_id)
                .ok_or_else(|| FinancialManagerError::PlayerNotFound(player_id.to_string()))?;

            let updated_user = UserData::new(user.cash() + sell_price, user.jail());

            self.memory_manager
                .modify_user(player_id, updated_user)
                .map_err(|_| FinancialManagerError::TransactionFailed(player_id.to_string()))
        } else {
            Err(FinancialManagerError::FieldNotSellable(field_id))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_mgmt::{FieldType, MemoryManager, PropertyData, UserData};
    use crate::financial_mgmt::FinancialManagerError;

    #[test]
    fn test_account_get_balance() {
        // Verify basic balance retrieval works.
    let mut mm = MemoryManager::new();
        mm.insert_user("Alice".into(), UserData::new(500, false));

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.accounts().get_balance("Alice"), Some(500));
    }

    #[test]
    fn test_account_deposit() {
        // Deposit to user account and check updated balance.
        let mut mm = MemoryManager::new();
        mm.insert_user("Bob".into(), UserData::new(300, false));

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.accounts().deposit("Bob", 200), Ok(()));
        assert_eq!(fm.accounts().get_balance("Bob"), Some(500));
    }

    #[test]
    fn test_account_withdraw() {
        // Withdraw funds and verify the balance is updated.
        let mut mm = MemoryManager::new();
        mm.insert_user("Charlie".into(), UserData::new(400, false));

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.accounts().withdraw("Charlie", 150), Ok(()));
        assert_eq!(fm.accounts().get_balance("Charlie"), Some(250));
    }

    #[test]
    fn test_account_transfer() {
        // Transfer between users and validate balances.
        let mut mm = MemoryManager::new();
        mm.insert_user("Dave".into(), UserData::new(500, false));
        mm.insert_user("Eve".into(), UserData::new(300, false));

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.accounts().transfer("Dave", "Eve", 200), Ok(()));
        assert_eq!(fm.accounts().get_balance("Dave"), Some(300));
        assert_eq!(fm.accounts().get_balance("Eve"), Some(500));
    }

    #[test]
    fn test_buy_property() {
        // Successful property purchase: check ownership and balance deduction.
        let mut mm = MemoryManager::new();
        mm.insert_user("Alice".into(), UserData::new(500, false));
        mm.insert_field(1, FieldType::Property(PropertyData::new(None, 200, 1)));

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.properties().buy_property("Alice", 1), Ok(()));
        assert_eq!(fm.properties().get_owner(1).unwrap(), "Alice");
        assert_eq!(fm.accounts().get_balance("Alice"), Some(300));
    }

    #[test]
    fn test_sell_property() {
        // Selling property credits the player's balance and clears ownership.
        let mut mm = MemoryManager::new();
        mm.insert_user("Alice".into(), UserData::new(300, false));
        mm.insert_field(
            1,
            FieldType::Property(PropertyData::new(Some("Alice".into()), 200, 1)),
        );

        // Retrieve owner name of already-owned field.
        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.properties().sell_property("Alice", 1, 150), Ok(()));
        assert_eq!(fm.properties().get_owner(1), None);
        assert_eq!(fm.accounts().get_balance("Alice"), Some(450));
    }

    #[test]
    fn test_property_get_owner() {
        // Retrieve owner name of already-owned field.
        let mut mm = MemoryManager::new();
        mm.insert_field(
            2,
            FieldType::Property(PropertyData::new(Some("Bob".into()), 300, 1)),
        );

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.properties().get_owner(2), Some("Bob".into()));
    }

    #[test]
    fn test_buy_property_insufficient_funds() {
        // Attempt to buy property with too little money should fail with error.
        let mut mm = MemoryManager::new();
        mm.insert_user("Charlie".into(), UserData::new(100, false));
        mm.insert_field(3, FieldType::Property(PropertyData::new(None, 200, 2)));

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(
            fm.properties().buy_property("Charlie", 3),
            Err(FinancialManagerError::InsufficientFunds {
                required: 200,
                available: 100
            })
        );
        assert_eq!(fm.properties().get_owner(3), None);
    }

    #[test]
    fn test_declare_bankruptcy() {
        // Declaring bankruptcy should reset balance and jail status.
        let mut mm = MemoryManager::new();
        mm.insert_user("Eve".into(), UserData::new(250, true));

        let mut fm = FinancialManager::new(&mut mm);
        assert_eq!(fm.accounts().declare_bankruptcy("Eve"), Ok(()));
        assert_eq!(fm.accounts().get_balance("Eve"), Some(0));
        assert!(!mm.get_user("Eve").unwrap().jail());
    }
}

