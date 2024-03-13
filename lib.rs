#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod hodl {
    use ink::storage::Mapping;

    #[ink(storage)]
    #[derive(Default)]
    pub struct Hodl {
        balances: Mapping<AccountId, Balance>,
        hold_until_block: Mapping<AccountId, u32>
    }

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
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
              balances: Mapping::default(),
              hold_until_block: Mapping::default()
            }
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn deposit(&mut self, number_of_block: u32) -> Result<u32, Error> {
            let caller_account = self.env().account_id();
            if self.balances.contains(caller_account) {
              return Err(Error::AlReadyDeposited);
            }

            let current_block = self.env().block_number();
            let amount_to_deposit = self.env().transferred_value();
            // check for overflow when adding the block number
            if let Some(locked_until_block) = current_block.checked_add(number_of_block) {
                self.balances.insert(caller_account, &amount_to_deposit);
                self.hold_until_block.insert(caller_account, &locked_until_block);
                return Ok(locked_until_block)
            } 
            Err(Error::BlockNumberIsTooHigh)
        }

        #[ink(message)]
        pub fn withdraw(&mut self) -> Result<(), Error> {
          let caller_account = self.env().caller();

          if !self.balances.contains(caller_account) {
            return Err(Error::InsufficientBalance)
          }
          if self.env().block_number() < self.hold_until_block.get(caller_account).unwrap() {
            return Err(Error::FundsLocked)
          }
          let owner_balance = self.balances.get(caller_account).unwrap();


          let transfert_result = self.env().transfer(caller_account, owner_balance);
          match transfert_result {
            Ok(_) => {
              self.balances.remove(caller_account);
              self.hold_until_block.remove(caller_account);
              Ok(())
            },
            Err(_) => Err(Error::TransferFailed)
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

            assert_eq!(hold.balances.get(accounts.alice), Some(amout_to_transfer), "balance should be transfered value");
            assert_eq!(hold.hold_until_block.get(accounts.alice), Some(number_of_blocks), "should hold until {} block", number_of_blocks);           
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
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_e2e::ContractsBackend;
        /// A helper function used for calling contract messages.

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Given
            let mut constructor = HodlRef::new();

            let contract_account_id = client
            .instantiate("hodl", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed")
            .account_id;
              // When
            Ok(())
        }
        
    }
}
