#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = MyEnvironment)]
pub mod erc721 {
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
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    /// A token ID.
    pub type TokenId = u64;

    #[ink(storage)]
    pub struct Erc721 {
        /// Mapping from token to owner.
        token_owner: Mapping<TokenId, AccountId>,
        /// Mapping from token to approvals users.
        token_approvals: Mapping<TokenId, AccountId>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: Mapping<AccountId, u32>,
        /// Mapping from owner to operator approvals.
        operator_approvals: Mapping<(AccountId, AccountId), ()>,
        /// Mapping from token to token uri.
        token_uri: Mapping<TokenId, String>,
        /// Importing Resolver contract address
        resolver_contract_address: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CannotInsert,
        CannotFetchValue,
        NotAllowed,
        DomainNotRegistered,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        id: TokenId,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        id: TokenId,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    impl Erc721 {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(resolver_contract_address: AccountId) -> Self {
            Self {
                token_owner: Mapping::default(),
                token_approvals: Mapping::default(),
                owned_tokens_count: Mapping::default(),
                operator_approvals: Mapping::default(),
                resolver_contract_address,
                token_uri: Mapping::default(),
            }
        }

        /// Returns the balance of the owner.
        /// This represents the amount of unique tokens the owner has.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u32 {
            self.balance_of_or_zero(&owner)
        }

        /// Returns the owner of the token.
        #[ink(message)]
        pub fn owner_of(&self, id: TokenId) -> Option<AccountId> {
            self.token_owner.get(id)
        }

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        pub fn get_approved(&self, id: TokenId) -> Option<AccountId> {
            self.token_approvals.get(id)
        }

        /// Returns `true` if the operator is approved by the owner.
        #[ink(message)]
        pub fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.approved_for_all(owner, operator)
        }

        /// Approves or disapproves the operator for all tokens of the caller.
        #[ink(message)]
        pub fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> Result<(), Error> {
            self.approve_for_all(to, approved)?;
            Ok(())
        }

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, id: TokenId) -> Result<(), Error> {
            self.approve_for(&to, id)?;
            Ok(())
        }

        /// Transfers the token from the caller to the given destination.
        #[ink(message)]
        pub fn transfer(&mut self, destination: AccountId, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &destination, id)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            self.transfer_token_from(&from, &to, id)?;
            Ok(())
        }

        /// Creates a new token.
        #[ink(message)]
        pub fn mint(
            &mut self,
            id: TokenId,
            domain_name: String,
            caller: AccountId,
            token_uri: String,
        ) -> Result<(), Error> {
            let domain_availaibiltiy = build_call::<MyEnvironment>()
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
                .invoke();

            assert!(!domain_availaibiltiy, "domain is not registered");

            self.token_uri.insert(id, &token_uri);
            self.add_token_to(&caller, id)?;
            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0x0; 20])),
                to: Some(caller),
                id,
            });
            Ok(())
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        pub fn burn(&mut self, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            let owner = token_owner.get(id).ok_or(Error::TokenNotFound)?;
            if owner != caller {
                return Err(Error::NotOwner);
            };

            let count = owned_tokens_count
                .get(caller)
                .map(|c| c.checked_sub(1).unwrap())
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(caller, &count);
            token_owner.remove(id);
            self.token_uri.remove(id);

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(AccountId::from([0x0; 20])),
                id,
            });

            Ok(())
        }

        /// Transfers token `id` `from` the sender to the `to` `AccountId`.
        fn transfer_token_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let owner = self.owner_of(id).ok_or(Error::TokenNotFound)?;
            if !self.approved_or_owner(caller, id, owner) {
                return Err(Error::NotApproved);
            };
            if owner != *from {
                return Err(Error::NotOwner);
            };
            self.clear_approval(id);
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id)?;
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                id,
            });
            Ok(())
        }

        /// Removes token `id` from the owner.
        fn remove_token_from(&mut self, from: &AccountId, id: TokenId) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if !token_owner.contains(id) {
                return Err(Error::TokenNotFound);
            }

            let count = owned_tokens_count
                .get(from)
                .map(|c| c.checked_sub(1).unwrap())
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(from, &count);
            token_owner.remove(id);

            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &AccountId, id: TokenId) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if token_owner.contains(id) {
                return Err(Error::TokenExists);
            }

            if *to == AccountId::from([0x0; 20]) {
                return Err(Error::NotAllowed);
            };

            let count = owned_tokens_count
                .get(to)
                .map(|c| c.checked_add(1).unwrap())
                .unwrap_or(1);

            owned_tokens_count.insert(to, &count);
            token_owner.insert(id, to);

            Ok(())
        }

        /// Approves or disapproves the operator to transfer all tokens of the caller.
        fn approve_for_all(&mut self, to: AccountId, approved: bool) -> Result<(), Error> {
            let caller = self.env().caller();
            if to == caller {
                return Err(Error::NotAllowed);
            }
            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator: to,
                approved,
            });

            if approved {
                self.operator_approvals.insert((&caller, &to), &());
            } else {
                self.operator_approvals.remove((&caller, &to));
            }

            Ok(())
        }

        /// Approve the passed `AccountId` to transfer the specified token on behalf of
        /// the message's sender.
        fn approve_for(&mut self, to: &AccountId, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            let owner = self.owner_of(id).ok_or(Error::TokenNotFound)?;
            if !(owner == caller || self.approved_for_all(owner, caller)) {
                return Err(Error::NotAllowed);
            };

            if *to == AccountId::from([0x0; 20]) {
                return Err(Error::NotAllowed);
            };

            if self.token_approvals.contains(id) {
                return Err(Error::CannotInsert);
            } else {
                self.token_approvals.insert(id, to);
            }

            self.env().emit_event(Approval {
                from: caller,
                to: *to,
                id,
            });

            Ok(())
        }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: TokenId) {
            self.token_approvals.remove(id);
        }

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &AccountId) -> u32 {
            self.owned_tokens_count.get(of).unwrap_or(0)
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.operator_approvals.contains((&owner, &operator))
        }

        /// Returns true if the `AccountId` `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: AccountId, id: TokenId, owner: AccountId) -> bool {
            from != AccountId::from([0x0; 20])
                && (from == owner
                    || self.token_approvals.get(id) == Some(from)
                    || self.approved_for_all(owner, from))
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        #[ink::test]
        fn mint_works() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x1; 20]));
            // Token 1 does not exists.
            assert_eq!(erc721.owner_of(1), None);
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 0);
            // Create token Id 1.
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
        }

        #[ink::test]
        fn mint_existing_should_fail() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1.
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // The first Transfer event takes place
            assert_eq!(1, ink::env::test::recorded_events().count());
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
            // Alice owns token Id 1.
            assert_eq!(erc721.owner_of(1), Some(AccountId::from([0x1; 20])));
            // Cannot create  token Id if it exists.
            // Bob cannot own token Id 1.
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Err(Error::TokenExists)
            );
        }

        #[ink::test]
        fn transfer_works() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1 for Alice
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Alice owns token 1
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
            // Bob does not owns any token
            assert_eq!(erc721.balance_of(AccountId::from([0x2; 20])), 0);
            // The first Transfer event takes place
            assert_eq!(1, ink::env::test::recorded_events().count());
            // Alice transfers token 1 to Bob
            assert_eq!(erc721.transfer(AccountId::from([0x2; 20]), 1), Ok(()));
            // The second Transfer event takes place
            assert_eq!(2, ink::env::test::recorded_events().count());
            // Bob owns token 1
            assert_eq!(erc721.balance_of(AccountId::from([0x2; 20])), 1);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Transfer token fails if it does not exists.
            assert_eq!(
                erc721.transfer(AccountId::from([0x2; 20]), 1),
                Err(Error::TokenNotFound)
            );
            // Token Id 2 does not exists.
            assert_eq!(erc721.owner_of(2), None);
            // Create token Id 2.
            assert_eq!(
                erc721.mint(
                    2,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
            // Token Id 2 is owned by Alice.
            assert_eq!(erc721.owner_of(2), Some(AccountId::from([0x1; 20])));
            // Set Bob as caller
            // Bob cannot transfer not owned tokens.
            assert_eq!(
                erc721.transfer(AccountId::from([0x3; 20]), 2),
                Err(Error::NotApproved)
            );
        }

        #[ink::test]
        fn approved_transfer_works() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1.
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Token Id 1 is owned by Alice.
            assert_eq!(erc721.owner_of(1), Some(AccountId::from([0x1; 20])));
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc721.approve(AccountId::from([0x2; 20]), 1), Ok(()));
            // Set Bob as caller
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(
                erc721.transfer_from(AccountId::from([0x1; 20]), AccountId::from([0x3; 20]), 1),
                Ok(())
            );
            // TokenId 3 is owned by Eve.
            assert_eq!(erc721.owner_of(1), Some(AccountId::from([0x3; 20])));
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 0);
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x2; 20])), 0);
            // Eve owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x3; 20])), 1);
        }

        #[ink::test]
        fn approved_for_all_works() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1.
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Create token Id 2.
            assert_eq!(
                erc721.mint(
                    2,
                    "arpitk.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Alice owns 2 tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 2);
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(
                erc721.set_approval_for_all(AccountId::from([0x2; 20]), true),
                Ok(())
            );
            // Bob is an approved operator for Alice
            assert!(
                erc721.is_approved_for_all(AccountId::from([0x1; 20]), AccountId::from([0x2; 20]))
            );
            // Set Bob as caller
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(
                erc721.transfer_from(AccountId::from([0x1; 20]), AccountId::from([0x3; 20]), 1),
                Ok(())
            );
            // TokenId 1 is owned by Eve.
            assert_eq!(erc721.owner_of(1), Some(AccountId::from([0x3; 20])));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
            // Bob transfers token Id 2 from Alice to Eve.
            assert_eq!(
                erc721.transfer_from(AccountId::from([0x1; 20]), AccountId::from([0x3; 20]), 2),
                Ok(())
            );
            // Bob does not own tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x2; 20])), 0);
            // Eve owns 2 tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x3; 20])), 2);
            // Remove operator approval for Bob on behalf of Alice.
            assert_eq!(
                erc721.set_approval_for_all(AccountId::from([0x2; 20]), false),
                Ok(())
            );
            // Bob is not an approved operator for Alice.
            assert!(
                !erc721.is_approved_for_all(AccountId::from([0x1; 20]), AccountId::from([0x2; 20]))
            );
        }

        #[ink::test]
        fn approve_nonexistent_token_should_fail() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Approve transfer of nonexistent token id 1
            assert_eq!(
                erc721.approve(AccountId::from([0x2; 20]), 1),
                Err(Error::TokenNotFound)
            );
        }

        #[ink::test]
        fn not_approved_transfer_should_fail() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1.
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x2; 20])), 0);
            // Eve does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x3; 20])), 0);
            // Set Eve as caller
            // Eve is not an approved operator by Alice.
            assert_eq!(
                erc721.transfer_from(AccountId::from([0x1; 20]), AccountId::from([0x4; 20]), 1),
                Err(Error::NotApproved)
            );
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x2; 20])), 0);
            // Eve does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x3; 20])), 0);
        }

        #[ink::test]
        fn burn_works() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1 for Alice
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 1);
            // Alice owns token Id 1.
            assert_eq!(erc721.owner_of(1), Some(AccountId::from([0x1; 20])));
            // Destroy token Id 1.
            assert_eq!(erc721.burn(1), Ok(()));
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(AccountId::from([0x1; 20])), 0);
            // Token Id 1 does not exists
            assert_eq!(erc721.owner_of(1), None);
        }

        #[ink::test]
        fn burn_fails_token_not_found() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Try burning a non existent token
            assert_eq!(erc721.burn(1), Err(Error::TokenNotFound));
        }

        #[ink::test]
        fn burn_fails_not_owner() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1 for Alice
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Try burning this token with a different account
            assert_eq!(erc721.burn(1), Err(Error::NotOwner));
        }

        #[ink::test]    
        fn transfer_from_fails_not_owner() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1 for Alice
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Bob can transfer alice's tokens
            assert_eq!(
                erc721.set_approval_for_all(AccountId::from([0x2; 20]), true),
                Ok(())
            );
            // Set caller to Frank
            // Create token Id 2 for Frank
            assert_eq!(
                erc721.mint(
                    2,
                    "arpitk.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Set caller to Bob
            // Bob makes invalid call to transfer_from (Alice is token owner, not Frank)
            assert_eq!(
                erc721.transfer_from(AccountId::from([0x4; 20]), AccountId::from([0x2; 20]), 1),
                Err(Error::NotOwner)
            );
        }

        #[ink::test]
        fn transfer_fails_not_owner() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new(AccountId::from([0x0; 20]));
            // Create token Id 1 for Alice
            assert_eq!(
                erc721.mint(
                    1,
                    "arpit.vne".to_string(),
                    AccountId::from([0x1; 20]),
                    "arpit".to_string()
                ),
                Ok(())
            );
            // Bob can transfer alice's tokens
            assert_eq!(
                erc721.set_approval_for_all(AccountId::from([0x2; 20]), true),
                Ok(())
            );
            // Set caller to bob

            // Bob makes invalid call to transfer (he is not token owner, Alice is)
            assert_eq!(
                erc721.transfer(AccountId::from([0x2; 20]), 1),
                Err(Error::NotOwner)
            );
        }

    }
}
