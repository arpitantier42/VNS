#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = MyEnvironment)]
pub mod priceoracle {
    #[derive(Clone)]
    pub struct MyEnvironment;

    impl ink::env::Environment for MyEnvironment {
        const MAX_EVENT_TOPICS: usize = 3;
        type AccountId = [u8; 20];
        type Balance = u128;
        type Hash = [u8; 32];
        type Timestamp = u64;
        type BlockNumber = u32;
        type ChainExtension = ::ink::env::NoChainExtension;
    }

    /// Define the Priceoracle contract.
    #[ink(storage)]
    pub struct Priceoracle {
        owner: AccountId,
        price_per_letter: Balance,
        price_per_year: Balance,
        premium_names: ink::prelude::vec::Vec<ink::prelude::string::String>,
    }

    impl Priceoracle {
        /// Constructor to initialize the owner and default values for `price_per_letter` and `price_per_year`.
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            // let caller = Self::env().caller();
            Self {
                owner,
                price_per_letter: 1u128.saturating_mul(10u128.pow(18)), // 1 Ether equivalent
                price_per_year: 20u128.saturating_mul(10u128.pow(18)),  // 20 Ether equivalent
                premium_names: ink::prelude::vec::Vec::new(),
            }
        }

        /// Modifier to allow only the contract owner to call certain functions.
        fn only_owner(&self) {
            let caller = Self::env().caller();
            assert_eq!(caller, self.owner, "Not the contract owner");
        }

        /// Function to update the price per letter (only owner).
        #[ink(message)]
        pub fn set_price_per_letter(&mut self, new_price_per_letter: Balance) {
            self.only_owner();
            self.price_per_letter = new_price_per_letter;
        }

        /// Function to update the price per year (only owner).
        #[ink(message)]
        pub fn set_price_per_year(&mut self, new_price_per_year: Balance) {
            self.only_owner();
            self.price_per_year = new_price_per_year;
        }
        /// Function to add a premium name (only owner).
        #[ink(message)]
        pub fn add_premium_name(&mut self, premium_name: ink::prelude::string::String) {
            self.only_owner();
            self.premium_names.push(premium_name);
        }

        /// Function to remove a premium name (only owner).
        #[ink(message)]
        pub fn remove_premium_name(&mut self, premium_name: ink::prelude::string::String) -> bool {
            self.only_owner();
            if let Some(pos) = self.premium_names.iter().position(|x| *x == premium_name) {
                self.premium_names.swap_remove(pos);
                true
            } else {
                false
            }
        }

        /// Function to get all premium names (public).
        #[ink(message)]
        pub fn get_premium_names(&self) -> ink::prelude::vec::Vec<ink::prelude::string::String> {
            self.premium_names.clone()
        }

        /// Function to get price per letter.
        #[ink(message)]
        pub fn get_price_per_letter(&self) -> Balance {
            self.price_per_letter
        }

        /// Function to get price per year.
        #[ink(message)]
        pub fn get_price_per_year(&self) -> Balance {
            self.price_per_year
        }

        /// Function to calculate the price.
        #[ink(message)]
        pub fn calculate_price(
            &self,
            name: ink::prelude::string::String,
            duration: Timestamp,
        ) -> Option<Balance> {
            let name_length = name.len() as u128; // Get the number of characters in the name

            assert!(duration > 0, "Duration cannot be zero");

            // Checked multiplication for price based on the number of letters
            let no_of_words_price = name_length.checked_mul(self.price_per_letter)?;

            // Checked multiplication for price based on the duration in years
            let num_years_price = (duration as u128)
                .checked_mul(self.price_per_year)?
                .checked_div(365 * 24 * 60 * 60)?;

            // Checked addition for total price
            let mut total_price = no_of_words_price.checked_add(num_years_price)?;

            // Check if the name is in the premium names list
            if self.premium_names.contains(&name) {
                total_price = total_price.checked_mul(10)?; // Multiply price by 10 if it's a premium name
            }

            // Return the total price, if all operations succeeded
            Some(total_price)
        }

        #[ink(message)]
        pub fn read_owner(&self) -> AccountId {
            self.owner
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let contract = Priceoracle::new(AccountId::from([0x1; 20]));
            assert_eq!(contract.price_per_letter, 1 * 10u128.pow(18));
            assert_eq!(contract.price_per_year, 20 * 10u128.pow(18));
        }

        #[ink::test]
        fn set_price_works() {
            let mut contract = Priceoracle::new(AccountId::from([0x1; 20]));
            contract.set_price_per_letter(2);
            contract.set_price_per_year(30);
            assert_eq!(contract.price_per_letter, 2);
            assert_eq!(contract.price_per_year, 30);
        }

        #[ink::test]
        fn calculate_price_works() {
            let contract = Priceoracle::new(AccountId::from([0x1; 20]));
            let price = contract.calculate_price("Alice".to_string(), 31536000); // 1 year
            assert_eq!(price, Some((5 * 1 + 20) * 10u128.pow(18))); // 5 letters * 1 Ether + 20 Ether for 1 year
        }

        #[ink::test]
        fn add_premium_name_works() {
            let mut contract = Priceoracle::new(AccountId::from([0x1; 20]));
            contract.add_premium_name("Alice".to_string());
            assert_eq!(contract.premium_names.len(), 1);
            assert_eq!(contract.premium_names[0], "Alice");
        }

        #[ink::test]
        fn remove_premium_name_works() {
            let mut contract = Priceoracle::new(AccountId::from([0x1; 20]));
            contract.add_premium_name("Alice".to_string());
            assert!(contract.remove_premium_name("Alice".to_string()));
            assert_eq!(contract.premium_names.len(), 0);
        }
    }
}
