#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod hodl {
    use ink::storage::Mapping;

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Hodler {
        pub balance: Balance,
        pub hold_until_block: u32
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Hodl {
        hodlers: Mapping<AccountId, Hodler>
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        FundsLocked,
        BlockNumberIsTooHigh,
        AlReadyDeposited,
        InsufficientBalance,
        TransferFailed,
    }

    impl Hodl {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
              hodlers: Mapping::default()
            }
        }

        #[ink(message)]
        pub fn get_balance(&self) -> Option<Balance> {
            let caller = self.env().caller();
            self.hodlers.get(caller).map(|hodler| hodler.balance)
        }

        #[ink(message)]
        pub fn get_funds_locked_until_block(&self) -> Option<u32> {
            let caller = self.env().caller();
            self.hodlers.get(caller).map(|hodler| hodler.hold_until_block)
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn deposit(&mut self, number_of_block: u32) -> Result<u32, Error> {
            let caller_account = self.env().caller();

            if self.hodlers.get(caller_account).is_some() {
                return Err(Error::AlReadyDeposited);
            }

            let current_block = self.env().block_number();
            let amount_to_deposit = self.env().transferred_value();
            // check for overflow when adding the block number
            if let Some(locked_until_block) = current_block.checked_add(number_of_block) {
                self.hodlers.insert(caller_account, &Hodler {
                    balance: amount_to_deposit,
                    hold_until_block: locked_until_block
                });

                return Ok(locked_until_block);
            } 
            Err(Error::BlockNumberIsTooHigh)
        }

        #[ink(message)]
        pub fn withdraw(&mut self) -> Result<(), Error> {
          let caller_account = self.env().caller();
          let hodler = self.hodlers.get(caller_account);
          match hodler {
            None => return Err(Error::InsufficientBalance),
            Some(hodler) => {
              if self.env().block_number() < hodler.hold_until_block {
                return Err(Error::FundsLocked)
              }
              let owner_balance = hodler.balance;

              let transfert_result = self.env().transfer(caller_account, owner_balance);
              match transfert_result {
                Ok(_) => {
                  self.hodlers.remove(caller_account);
                  Ok(())
                },
                Err(_) => Err(Error::TransferFailed)
              }
            }
          }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let _hold = Hodl::default();
        }

        #[ink::test]
        fn can_transfer() {
            let number_of_blocks = 10;
            let amout_to_transfer = Balance::from(100 as u32);
            let mut hold = Hodl::default();
            let accounts =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(amout_to_transfer);
            let result = hold.deposit(number_of_blocks);
            assert_eq!(result, Ok(number_of_blocks));

            assert_eq!(hold.hodlers.get(accounts.alice).unwrap().balance, amout_to_transfer, "balance should be transfered value");
            assert_eq!(hold.hodlers.get(accounts.alice).unwrap().hold_until_block, number_of_blocks, "should hold until {} block", number_of_blocks);           
        }

        #[ink::test]
        fn can_only_deposit_once() {
            let number_of_blocks = 10;
            let amout_to_transfer = Balance::from(100 as u32);
            let mut hold = Hodl::default();
            let accounts =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(amout_to_transfer);
            let result = hold.deposit(number_of_blocks);
            assert_eq!(result, Ok(number_of_blocks), "should be able to deposit (first time)");

            let result = hold.deposit(number_of_blocks);
            assert_eq!(result, Err(Error::AlReadyDeposited), "should not be able to deposit (second time)");
        }
    }
    

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = HodlRef::new();

            // When
            let contract_account_id = client
                .instantiate("hodl", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
            Ok(())
        }
        
    }
}
