#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = MyEnvironment)]
pub mod resolver {
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
    use ink::env::hash::{HashOutput, Sha2x256};
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Record {
        records: Mapping<String, Records>,
        domain_content_text: Mapping<String, ContentText>,
        sub_domain_content_text: Mapping<String, SubDomainContentText>,
        sub_domain_manager: Mapping<String, AccountId>,
        admin: AccountId,
        manager: AccountId,
        grace_period: Timestamp,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Records {
        domain_name: String,
        domain_owner: AccountId,
        duration: Timestamp,
        secret: [u8; 32],
        resolver: AccountId,
        domain_expiry_time: Timestamp,
        sub_domain: String,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Domain {
        domain_name: String,
        domain_owner: AccountId,
        domain_expiry_time: Timestamp,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Default)]
    pub struct ContentText {
        social: Vec<String>,
        general: Vec<String>,
        address: Vec<String>,
        avatar: String,
        abi: String,
        ipfs: String,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Default)]
    pub struct SubDomainContentText {
        social: Vec<String>,
        general: Vec<String>,
        address: Vec<String>,
        avatar: String,
        abi: String,
        ipfs: String,
    }

    #[ink(event)]
    pub struct ContentTextInfo {
        domain_name: String,
        content_text: ContentText,
    }

    #[ink(event)]
    pub struct GracePeriod {
        grace_period: Timestamp,
    }

    #[ink(event)]
    pub struct SubDomainContentTextInfo {
        sub_domain_name: String,
        sub_domain_content_text: SubDomainContentText,
    }

    #[ink(event)]
    pub struct DomainOwnerInfo {
        domain_name: String,
        domain_owner: AccountId,
        records_availability: bool,
    }

    #[ink(event)]
    pub struct RenewDomainInfo {
        domain_name: String,
        domain_expiry_time: Timestamp,
        domain_duration: Timestamp,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        UndefinedError,
        DomainNotRegistered,
        InvalidCaller,
        DomainNotExpired,
        RenewTimeExpired,
        InvalidContentKey,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    impl Record {
        #[ink(constructor)]
        pub fn new(admin: AccountId, manager: AccountId, grace_period: Timestamp) -> Self {
            Self {
                records: Mapping::default(),
                domain_content_text: Mapping::default(),
                sub_domain_content_text: Mapping::default(),
                sub_domain_manager: Mapping::default(),
                admin,
                manager,
                grace_period,
            }
        }

        #[ink(message)]
        pub fn set_record(
            &mut self,
            label: Hash,
            domain_name: String,
            domain_owner: AccountId,
            duration: Timestamp,
            secret: [u8; 32],
            resolver: AccountId,
            domain_expiry_time: Timestamp,
        ) -> bool {
            let mut _label = <Sha2x256 as HashOutput>::Type::default();
            let domain_info =
                self.create_domain_info(domain_name.clone(), domain_owner, domain_expiry_time);
            ink::env::hash_encoded::<Sha2x256, _>(&domain_info, &mut _label);
            let label_hash = Hash::from(_label);

            if !self.records.contains(domain_name.clone()) && label_hash == label {
                let record_info = self.create_record_info(
                    domain_name.clone(),
                    domain_owner,
                    duration,
                    secret,
                    resolver,
                    domain_expiry_time,
                );
                self.records.insert(domain_name.clone(), &record_info);
                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn set_domain_content_text(
            &mut self,
            domain_name: String,
            content_key: String,
            domain_content_text: String,
        ) -> Result<()> {
            self.only_domain_owner(domain_name.clone());

            let mut texts: ContentText = self
                .domain_content_text
                .get(domain_name.clone())
                .unwrap_or_default();
            match content_key.as_str() {
                "social" => texts.social.push(domain_content_text),
                "general" => texts.general.push(domain_content_text),
                "address" => texts.address.push(domain_content_text),
                "avatar" => texts.avatar = domain_content_text,
                "abi" => texts.abi = domain_content_text,
                _ => return Err(Error::InvalidContentKey),
            }
            self.domain_content_text.insert(domain_name.clone(), &texts);
            self.env().emit_event(ContentTextInfo {
                domain_name,
                content_text: texts.clone(),
            });
            Ok(())
        }

        #[ink(message)]
        pub fn set_content_hash(
            &mut self,
            domain_name: String,
            content_hash: String,
        ) -> Result<()> {
            self.only_domain_owner(domain_name.clone());

            let mut texts = self
                .domain_content_text
                .get(domain_name.clone())
                .unwrap_or_default();
            texts.ipfs = content_hash;
            self.domain_content_text.insert(domain_name.clone(), &texts);

            self.env().emit_event(ContentTextInfo {
                domain_name,
                content_text: texts.clone(),
            });
            Ok(())
        }

        #[ink(message)]
        pub fn change_domain_owner(
            &mut self,
            domain_name: String,
            new_domain_owner: AccountId,
            records_availability: bool,
        ) -> Result<()> {
            self.only_domain_owner(domain_name.clone());

            let mut record_info: Records = self.records.get(domain_name.clone()).unwrap();
            record_info.domain_owner = new_domain_owner;
            self.records.insert(domain_name.clone(), &record_info);
            if !records_availability {
                self.domain_content_text.remove(domain_name.clone());
            }
            self.env().emit_event(DomainOwnerInfo {
                domain_name,
                domain_owner: new_domain_owner,
                records_availability,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn unregister_domain(&mut self, domain_name: String) -> Result<()> {
            self.only_manager();

            if self.env().block_timestamp()
                <= self
                    .read_domain_expiry_time(domain_name.clone())
                    .add(self.grace_period)
            {
                Err(Error::DomainNotExpired)
            } else {
                self.records.remove(domain_name.clone());
                self.domain_content_text.remove(domain_name.clone());

                Ok(())
            }
        }

        #[ink(message)]
        pub fn renew_domain(&mut self, domain_name: String, new_duration: Timestamp) -> Result<()> {
            let domain_expiry_time = self.read_domain_expiry_time(domain_name.clone());
            self.only_domain_owner(domain_name.clone());

            if self.env().block_timestamp() < domain_expiry_time
                && self.env().block_timestamp() > domain_expiry_time.add(self.grace_period)
            {
                Err(Error::RenewTimeExpired)
            } else {
                let mut record_info = self.records.get(domain_name.clone()).unwrap();
                record_info.duration = record_info.duration.add(new_duration);
                record_info.domain_expiry_time = domain_expiry_time.add(new_duration);
                self.records.insert(domain_name.clone(), &record_info);

                self.env().emit_event(RenewDomainInfo {
                    domain_name,
                    domain_expiry_time: record_info.domain_expiry_time,
                    domain_duration: record_info.duration,
                });
                Ok(())
            }
        }

        #[ink(message)]
        pub fn register_subdomain(&mut self, parent_domain: String, sub_domain: String) -> bool {
            self.only_domain_owner(parent_domain.clone());
            let mut parent_domain_records = self.records.get(parent_domain.clone()).unwrap();
            parent_domain_records.sub_domain = sub_domain.clone();
            self.records.insert(parent_domain, &parent_domain_records);
            self.sub_domain_manager
                .insert(sub_domain, &Self::env().caller());

            true
        }

        #[ink(message)]
        pub fn unregister_subdomain(&mut self, parent_domain: String) -> Result<()> {
            self.only_domain_owner(parent_domain.clone());
            let mut parent_records = self.records.get(parent_domain.clone()).unwrap();
            parent_records.sub_domain = String::from("");
            self.records.insert(parent_domain.clone(), &parent_records);
            Ok(())
        }

        #[ink(message)]
        pub fn set_sub_domain_content_text(
            &mut self,
            sub_domain_name: String,
            content_key: String,
            sub_domain_content_text: String,
        ) -> Result<()> {
            self.only_sub_domain_manager(sub_domain_name.clone());

            let mut texts = self
                .sub_domain_content_text
                .get(sub_domain_name.clone())
                .unwrap_or_default();
            match content_key.as_str() {
                "social" => texts.social.push(sub_domain_content_text),
                "general" => texts.general.push(sub_domain_content_text),
                "address" => texts.address.push(sub_domain_content_text),
                "avatar" => texts.avatar = sub_domain_content_text,
                "abi" => texts.abi = sub_domain_content_text,
                _ => return Err(Error::InvalidContentKey),
            }
            self.sub_domain_content_text
                .insert(sub_domain_name.clone(), &texts);

            self.env().emit_event(SubDomainContentTextInfo {
                sub_domain_name,
                sub_domain_content_text: texts.clone(),
            });

            Ok(())
        }

        #[ink(message)]
        pub fn set_grace_period(&mut self, new_grace_period: Timestamp) {
            self.only_admin();
            self.grace_period = new_grace_period;
            self.env().emit_event(GracePeriod {
                grace_period: new_grace_period,
            });
        }

        #[ink(message)]
        pub fn change_manager(&mut self, manager: AccountId) {
            self.only_admin();
            self.manager = manager;
        }

        #[ink(message)]
        pub fn change_sub_domain_manager(&mut self, parent_domain: String, manager: AccountId) {
            self.only_domain_owner(parent_domain.clone());
            let sub_domain = self.records.get(parent_domain.clone()).unwrap().sub_domain;

            self.sub_domain_manager.insert(sub_domain, &manager);
        }

        #[ink(message)]
        pub fn read_grace_period(&self) -> Timestamp {
            self.grace_period
        }

        #[ink(message)]
        pub fn read_domain_content_text(&self, domain_name: String) -> ContentText {
            self.domain_content_text.get(domain_name).unwrap()
        }

        #[ink(message)]
        pub fn read_subdomain_content_text(&self, sub_domian_name: String) -> SubDomainContentText {
            self.sub_domain_content_text.get(sub_domian_name).unwrap()
        }

        #[ink(message)]
        pub fn read_content_hash(&self, domain_name: String) -> String {
            self.domain_content_text.get(domain_name).unwrap().ipfs
        }

        #[ink(message)]
        pub fn read_record(&self, domain_name: String) -> Records {
            self.records.get(domain_name).unwrap()
        }

        #[ink(message)]
        pub fn read_domain_owner(&self, domain_name: String) -> AccountId {
            let owner_record = self.records.get(domain_name).unwrap();
            owner_record.domain_owner
        }

        #[ink(message)]
        pub fn read_sub_domain_owner(&self, parent_domain: String) -> AccountId {
            let owner_record = self.records.get(parent_domain).unwrap();
            owner_record.domain_owner
        }

        #[ink(message)]
        pub fn read_sub_domain_manager(&self, sub_domain: String) -> AccountId {
            self.sub_domain_manager.get(sub_domain.clone()).unwrap()
        }

        #[ink(message)]
        pub fn read_domain_expiry_time(&self, domain_name: String) -> Timestamp {
            let owner_record = self.records.get(domain_name).unwrap();
            owner_record.domain_expiry_time
        }

        #[ink(message)]
        pub fn check_domain_availablility(&self, domain_name: String) -> bool {
            let availability = self.records.contains(domain_name);
            !availability
        }

        #[ink(message)]
        pub fn read_admin(&self) -> AccountId {
            self.admin
        }

        #[ink(message)]
        pub fn read_manager(&self) -> AccountId {
            self.manager
        }

        fn create_record_info(
            &self,
            domain_name: String,
            domain_owner: AccountId,
            duration: Timestamp,
            secret: [u8; 32],
            resolver: AccountId,
            domain_expiry_time: Timestamp,
        ) -> Records {
            Records {
                domain_name,
                domain_owner,
                duration,
                secret,
                resolver,
                domain_expiry_time,
                sub_domain: String::from(""),
            }
        }

        fn create_domain_info(
            &self,
            domain_name: String,
            domain_owner: AccountId,
            domain_expiry_time: Timestamp,
        ) -> Domain {
            Domain {
                domain_name,
                domain_owner,
                domain_expiry_time,
            }
        }

        fn only_admin(&self) {
            let caller = Self::env().caller();
            assert_eq!(caller, self.admin, "must be contract owner");
        }

        fn only_manager(&self) {
            let caller = Self::env().caller();
            assert_eq!(caller, self.manager, " must be contract manager");
        }

        fn only_domain_owner(&self, domain_name: String) {
            let caller = Self::env().caller();
            assert_eq!(
                caller,
                self.read_domain_owner(domain_name),
                "must be domain owner!"
            );
        }
        fn only_sub_domain_manager(&self, sub_domain: String) {
            let caller = Self::env().caller();
            assert_eq!(
                caller,
                self.read_sub_domain_manager(sub_domain),
                "must be sub domain manager!"
            );
        }
    }
}
