#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod priceoracle {

    #[ink(storage)]
    pub struct Priceoracle {
        owner: AccountId,
        price_per_letter: Balance,
        price_per_year: Balance,
        premium_names: ink::prelude::vec::Vec<ink::prelude::string::String>,
    }

    impl Priceoracle {
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            Self {
                owner,
                price_per_letter: 1u128.saturating_mul(10u128.pow(18)), 
                price_per_year: 20u128.saturating_mul(10u128.pow(18)), 
                premium_names: ink::prelude::vec::Vec::new(),
            }
        }

        fn only_owner(&self) {
            let caller = Self::env().caller();
            assert_eq!(caller, self.owner, "Not the contract owner");
        }

        #[ink(message)]
        pub fn set_price_per_letter(&mut self, new_price_per_letter: Balance) {
            self.only_owner();
            self.price_per_letter = new_price_per_letter;
        }

        #[ink(message)]
        pub fn set_price_per_year(&mut self, new_price_per_year: Balance) {
            self.only_owner();
            self.price_per_year = new_price_per_year;
        }

        #[ink(message)]
        pub fn add_premium_name(&mut self, premium_name: ink::prelude::string::String) {
            self.only_owner();
            self.premium_names.push(premium_name);
        }

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

        #[ink(message)]
        pub fn get_premium_names(&self) -> ink::prelude::vec::Vec<ink::prelude::string::String> {
            self.premium_names.clone()
        }

        #[ink(message)]
        pub fn get_price_per_letter(&self) -> Balance {
            self.price_per_letter
        }

        #[ink(message)]
        pub fn get_price_per_year(&self) -> Balance {
            self.price_per_year
        }

        #[ink(message)]
        pub fn calculate_price(
            &self,
            name: ink::prelude::string::String,
            duration: Timestamp,
        ) -> Option<Balance> {
            let name_length = name.len() as u128; 

            assert!(duration > 0, "Duration cannot be zero");

            let no_of_words_price = name_length.checked_mul(self.price_per_letter)?;

            let num_years_price = (duration as u128)
                .checked_mul(self.price_per_year)?
                .checked_div(365 * 24 * 60 * 60)?;

            let mut total_price = no_of_words_price.checked_add(num_years_price)?;

            if self.premium_names.contains(&name) {
                total_price = total_price.checked_mul(10)?; 
            }

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
        fn test_initial_state() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let contract = Priceoracle::new(accounts.alice);

            assert_eq!(contract.read_owner(), accounts.alice);

            assert_eq!(
                contract.get_price_per_letter(),
                1u128.saturating_mul(10u128.pow(18))
            );
            assert_eq!(
                contract.get_price_per_year(),
                20u128.saturating_mul(10u128.pow(18))
            );

            assert_eq!(contract.get_premium_names().len(), 0);
        }

        #[ink::test]
        fn test_owner_only_restriction() {

            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let mut contract = Priceoracle::new(accounts.alice);

            contract.set_price_per_letter(2u128.saturating_mul(10u128.pow(18)));
            assert_eq!(
                contract.get_price_per_letter(),
                2u128.saturating_mul(10u128.pow(18))
            );
        }

        #[ink::test]
        fn test_add_remove_premium_name() {

            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let mut contract = Priceoracle::new(accounts.alice);

            contract.add_premium_name("premium_name".to_string());

            let premium_names = contract.get_premium_names();
            assert_eq!(premium_names.len(), 1);
            assert_eq!(premium_names[0], "premium_name");

            let result = contract.remove_premium_name("premium_name".to_string());
            assert!(result);

            let premium_names = contract.get_premium_names();
            assert_eq!(premium_names.len(), 0);
        }

        #[ink::test]
        fn test_calculate_price_without_premium() {

            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let contract = Priceoracle::new(accounts.alice);

            let name = "test".to_string();
            let duration = 365 * 24 * 60 * 60; 

            let calculated_price = contract.calculate_price(name, duration);
            let expected_price =
                (4u128 * contract.get_price_per_letter()) + contract.get_price_per_year();
            assert_eq!(calculated_price, Some(expected_price));
        }

        #[ink::test]
        fn test_calculate_price_with_premium() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let mut contract = Priceoracle::new(accounts.alice);

            contract.add_premium_name("premium".to_string());

            let name = "premium".to_string();
            let duration = 365 * 24 * 60 * 60; 

            let calculated_price = contract.calculate_price(name, duration);

            let expected_price =
                (7u128 * contract.get_price_per_letter()) + contract.get_price_per_year();

            let expected_price_with_premium = expected_price * 10;

            assert_eq!(calculated_price, Some(expected_price_with_premium));
        }

        #[ink::test]
        fn test_remove_non_existing_premium_name() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let mut contract = Priceoracle::new(accounts.alice);

            let result = contract.remove_premium_name("non_existing_name".to_string());

            assert_eq!(result, false);
        }
    }
}
