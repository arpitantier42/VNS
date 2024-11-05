#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = MyEnvironment)]
mod registry {

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

    use ink::env::call::{build_call, ExecutionInput, Selector};
    // use ink::env::DefaultEnvironment;
    use ink::prelude::string::String;

    #[ink(storage)]
    pub struct Registry {
        resolver_contract_address: AccountId,
    }

    impl Registry {
        #[ink(constructor)]
        pub fn new(resolver_contract_address: AccountId) -> Self {
            Self {
                resolver_contract_address,
            }
        }

        #[ink(message)]
        pub fn read_owner(&self, domain_name: String) -> AccountId {
            build_call::<MyEnvironment>()
                .call(AccountId::from(self.resolver_contract_address))
                .call_v1()
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("read_owner")))
                        .push_arg(domain_name),
                )
                .returns::<AccountId>()
                .invoke()
        }

        #[ink(message)]
        pub fn read_expiry_time(&self, domain_name: String) -> Timestamp {
            build_call::<MyEnvironment>()
                .call(AccountId::from(self.resolver_contract_address))
                .call_v1()
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("read_expiry_time")))
                        .push_arg(domain_name),
                )
                .returns::<Timestamp>()
                .invoke()
        }
        #[ink(message)]
        pub fn read_content_hash(&self, domain_name:String) -> String{
            build_call::<MyEnvironment>()
            .call(AccountId::from(self.resolver_contract_address))
            .call_v1()
            .gas_limit(0)
            .transferred_value(0)
            .exec_input(
                ExecutionInput::new(Selector::new(ink::selector_bytes!("read_content_hash")))
                    .push_arg(domain_name),
            )
            .returns::<String>()
            .invoke()
        }

    }
}

