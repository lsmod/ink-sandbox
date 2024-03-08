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

    impl Hodl {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self { 
              balances: Default::default(),
              hold_until_block: Default::default()
            }
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn deposit(&mut self, number_of_block: u32) -> Result<u32, ()> {
            let caller_account = self.env().account_id();
            if self.balances.contains(caller_account) {
              return Err(());
            }

            let current_block = self.env().block_number();
            let amount_to_deposit = self.env().transferred_value();
            let locked_until_block = current_block + number_of_block;

            self.balances.insert(caller_account, &amount_to_deposit);
            self.hold_until_block.insert(caller_account, &locked_until_block);

            Ok(locked_until_block)
        }

        #[ink(message)]
        pub fn get_holders(&self) -> bool {
          true
        }

        #[ink(message)]
        pub fn withdraw(&mut self) -> Result<(), ()> {
          let caller_account = self.env().caller();

          if !self.balances.contains(caller_account) {
            return Err(())
          }
          if self.env().block_number() < self.hold_until_block.get(caller_account).unwrap() {
            return Err(())
          }
          let owner_balance = self.balances.get(caller_account).unwrap();


          let transfert_result = self.env().transfer(caller_account, owner_balance);
          match transfert_result {
            Ok(_) => {
              self.balances.remove(caller_account);
              self.hold_until_block.remove(caller_account);
              Ok(())
            },
            Err(_) => Err(())
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
            assert_eq!(result, Err(()), "should not be able to deposit (second time)");
        }
    }
  }