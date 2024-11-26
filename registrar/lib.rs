#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = MyEnvironment)]
pub mod registrar {
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

    use core::ops::Add;
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::prelude::string::String;
    use ink::{
        env::hash::{HashOutput, Sha2x256},
        storage::Mapping,
    };

    #[ink(storage)]
    pub struct Registrar {
        commitments: Mapping<Hash, Timestamp>,
        commit_info: Mapping<Hash, CommitInfo>,
        admin: AccountId,
        max_commit_age: u64,
        min_commit_age: u64,
        min_registration_duration: u64,
        resolver_contract_address: AccountId,
        price_oracle_contract_address: AccountId,
        erc721: AccountId,
        token_id: u64,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct CommitInfo {
        domain_name: String,
        domain_owner: AccountId,
        duration: Timestamp,
        secret: [u8; 32],
        resolver: AccountId,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct DomainInfo {
        domain_name: String,
        domain_owner: AccountId,
        domain_expiry_time: Timestamp,
    }
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        UnexpiredCommitmentExists,
        CommitmentNotFound,
        CommitmentTooNew,
        CommitmentTooOld,
        DurationTooShort,
        UndefinedError,
        AlreadyRegistered,
        DomainNotRegistered,
        DomainNotExpired,
    }
    #[ink(event)]
    pub struct Register {
        domain_name: String,
        domain_owner: AccountId,
        registration_fee: Balance,
        duration: Timestamp,
        domain_creation_time: Timestamp,
        domain_expiry_time: Timestamp,
        domain_grace_period: Timestamp,
        resolver: AccountId,
    }

    #[ink(event)]
    pub struct Commit {
        commit_hash: Hash,
        caller: AccountId,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Registrar {
        #[ink(constructor)]
        pub fn new(
            admin: AccountId,
            max_commit_age: u64,
            min_commit_age: u64,
            min_registration_duration: u64,
            resolver_contract_address: AccountId,
            price_oracle_contract_address: AccountId,
            erc721: AccountId,
        ) -> Self {
            Self {
                admin,
                commitments: Mapping::default(),
                commit_info: Mapping::default(),
                max_commit_age,
                min_commit_age,
                min_registration_duration,
                resolver_contract_address,
                price_oracle_contract_address,
                erc721,
                token_id: u64::default(),
            }
        }

        #[ink(message)]
        pub fn commit(&mut self, commit_hash: Hash) -> Result<()> {
            if self.commitments.get(commit_hash).is_none() {
                self.commitments
                    .insert(commit_hash, &self.env().block_timestamp());

                self.env().emit_event(Commit {
                    commit_hash,
                    caller: Self::env().caller(),
                });

                return Ok(());
            }

            if self
                .commitments
                .get(commit_hash)
                .unwrap()
                .add(self.max_commit_age)
                >= self.env().block_timestamp()
            {
                Err(Error::UnexpiredCommitmentExists)
            } else {
                Err(Error::UndefinedError)
            }
        }

        #[ink(message, payable)]
        pub fn register(
            &mut self,
            domain_name: String,
            domain_owner: AccountId,
            duration: Timestamp,
            secret: [u8; 32],
            resolver: AccountId,
        ) -> Result<()> {
            assert!(
                self.env().transferred_value()
                    == self
                        .read_domain_price(domain_name.clone(), duration)
                        .unwrap(),
                "Insufficient domain registration cost"
            );

            let commit_hash = self.make_commitment(
                domain_name.clone(),
                domain_owner,
                duration,
                secret,
                resolver,
            );
            self.consume_commitment(domain_name.clone(), duration, commit_hash)?;

            // to create label hash
            let mut label = <Sha2x256 as HashOutput>::Type::default();
            let domain_expiry_time = self.env().block_timestamp().add(duration);
            let domain_info: DomainInfo =
                self.create_domain_info(domain_name.clone(), domain_owner, domain_expiry_time);
            ink::env::hash_encoded::<Sha2x256, _>(&domain_info, &mut label);
            let label_hash = Hash::from(label);

            let set_record = self.set_record(
                label_hash,
                domain_name.clone(),
                domain_owner,
                duration,
                secret,
                resolver,
                domain_expiry_time,
            );

            if !set_record {
                return Err(Error::AlreadyRegistered);
            }

            self.env()
                .transfer(
                    self.read_admin(),
                    self.read_domain_price(domain_name.clone(), duration)
                        .unwrap(),
                )
                .unwrap();

            self.env().emit_event(Register {
                domain_name,
                domain_owner,
                registration_fee: self.env().transferred_value(),
                duration,
                domain_creation_time: self.env().block_timestamp(),
                domain_expiry_time,
                domain_grace_period: 120000,
                resolver,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn register_subdomian(
            &mut self,
            parent_domain: String,
            sub_domain: String,
        ) -> Result<()> {
            build_call::<MyEnvironment>()
                .call(AccountId::from(self.resolver_contract_address))
                .call_v1()
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("register_subdomain")))
                        .push_arg(parent_domain)
                        .push_arg(sub_domain),
                )
                .returns::<bool>()
                .invoke();
            Ok(())
        }

        #[ink(message)]
        pub fn mint_nft(
            &mut self,
            domain_name: String,
            domain_owner: AccountId,
            token_uri: String,
        ) -> Result<()> {
            let token_id = self.token_id.add(1);

            build_call::<MyEnvironment>()
                .call(AccountId::from(self.erc721))
                .call_v1()
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("mint")))
                        .push_arg(token_id)
                        .push_arg(domain_name)
                        .push_arg(domain_owner)
                        .push_arg(token_uri),
                )
                .returns::<Result<()>>()
                .invoke()
        }

        #[ink(message)]
        pub fn set_max_commit_age(&mut self, max_commit_age: u64) {
            self.only_admin();
            self.max_commit_age = max_commit_age;
        }

        #[ink(message)]
        pub fn set_min_commit_age(&mut self, min_commit_age: u64) {
            self.only_admin();
            self.min_commit_age = min_commit_age;
        }

        #[ink(message)]
        pub fn set_min_registration_duration(&mut self, min_registration_duration: u64) {
            self.only_admin();
            self.min_registration_duration = min_registration_duration;
        }

        #[ink(message)]
        pub fn make_commitment(
            &self,
            domain_name: String,
            domain_owner: AccountId,
            duration: Timestamp,
            secret: [u8; 32],
            resolver: AccountId,
        ) -> Hash {
            assert!(
                duration > self.min_registration_duration,
                "duration should me more than minimum registration duration"
            );
            assert!(
                self.contains_vne(domain_name.as_bytes()),
                "Not a valid domain"
            );

            let mut commit = <Sha2x256 as HashOutput>::Type::default();
            let commit_info =
                self.create_commit_info(domain_name, domain_owner, duration, secret, resolver);

            ink::env::hash_encoded::<Sha2x256, _>(&commit_info, &mut commit);
            Hash::from(commit)
        }

        #[ink(message)]
        pub fn check_domain_availablility(&self, domain_name: String) -> bool {
            build_call::<MyEnvironment>()
                .call(AccountId::from(self.resolver_contract_address))
                .call_v1()
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "check_domain_availablility"
                    )))
                    .push_arg(domain_name),
                )
                .returns::<bool>()
                .invoke()
        }

        #[ink(message)]
        pub fn read_domain_price(
            &self,
            domain_name: String,
            duration: Timestamp,
        ) -> Option<Balance> {
            build_call::<MyEnvironment>()
                .call(AccountId::from(self.price_oracle_contract_address))
                .call_v1()
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("calculate_price")))
                        .push_arg(domain_name)
                        .push_arg(duration),
                )
                .returns::<Option<Balance>>()
                .invoke()
        }

        #[ink(message)]
        pub fn read_admin(&self) -> AccountId {
            self.admin
        }

        #[ink(message)]
        pub fn read_resolver(&self) -> AccountId {
            self.resolver_contract_address
        }

        #[ink(message)]
        pub fn read_max_commit_age(&self) -> Timestamp {
            self.max_commit_age
        }
        #[ink(message)]
        pub fn read_min_commit_age(&self) -> Timestamp {
            self.min_commit_age
        }
        #[ink(message)]
        pub fn read_min_registration_duration(&self) -> Timestamp {
            self.min_registration_duration
        }

        #[ink(message)]
        pub fn current_timestamp(&self) -> Timestamp {
            self.env().block_timestamp()
        }

        fn only_admin(&self) {
            let caller = Self::env().caller();
            assert_eq!(caller, self.admin, "Not the contract owner");
        }

        fn consume_commitment(
            &self,
            _domain_name: String,
            duration: u64,
            commit_hash: Hash,
        ) -> Result<()> {
            let current_time = self.env().block_timestamp();
            let commitment = self
                .commitments
                .get(commit_hash)
                .ok_or(Error::CommitmentNotFound)?;

            if commitment.add(self.min_commit_age) > current_time {
                return Err(Error::CommitmentTooNew);
            }
            if commitment.add(self.max_commit_age) <= current_time {
                return Err(Error::CommitmentTooOld);
            }
            if duration < self.min_registration_duration {
                return Err(Error::DurationTooShort);
            }
            if !self.check_domain_availablility(_domain_name) {
                return Err(Error::AlreadyRegistered);
            }
            Ok(())
        }

        fn create_commit_info(
            &self,
            domain_name: String,
            domain_owner: AccountId,
            duration: Timestamp,
            secret: [u8; 32],
            resolver: AccountId,
        ) -> CommitInfo {
            CommitInfo {
                domain_name,
                domain_owner,
                duration,
                secret,
                resolver,
            }
        }

        fn create_domain_info(
            &self,
            domain_name: String,
            domain_owner: AccountId,
            domain_expiry_time: Timestamp,
        ) -> DomainInfo {
            DomainInfo {
                domain_name,
                domain_owner,
                domain_expiry_time,
            }
        }

        fn contains_vne(&self, input: &[u8]) -> bool {
            let substring: &[u8] = b".vne";
            let input_len = input.len();
            let substring_len = substring.len();

            if input_len < substring_len {
                return false;
            }

            for i in 0..=(input_len.checked_sub(substring_len)).unwrap() {
                if &input[i..i.add(substring_len)] == substring {
                    return true;
                }
            }
            false
        }

        fn set_record(
            &self,
            label_hash: Hash,
            domain_name: String,
            domain_owner: AccountId,
            duration: Timestamp,
            secret: [u8; 32],
            resolver: AccountId,
            domain_expiry_time: Timestamp,
        ) -> bool {
            build_call::<MyEnvironment>()
                .call(AccountId::from(self.resolver_contract_address))
                .call_v1()
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("set_record")))
                        .push_arg(label_hash)
                        .push_arg(domain_name.clone())
                        .push_arg(domain_owner)
                        .push_arg(duration)
                        .push_arg(secret)
                        .push_arg(resolver)
                        .push_arg(domain_expiry_time),
                )
                .returns::<bool>()
                .invoke()
        }
    }

}
