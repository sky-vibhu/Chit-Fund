#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod my_contract {
    use ink::{prelude::vec::Vec};
    use ink_env::emit_event;
    use ink::prelude::*;

    #[ink(storage)]
    pub struct ChitFund {
        pub admin: AccountId,
        pub max_participants: u32,
        pub monthly_contribution: Balance,
        pub current_round: u32,
        pub pot: Balance,
        pub total_amount: Balance,
        pub participants: Vec<AccountId>,
        pub used_indexes: Vec<AccountId>,
        pub finished: bool,
    } 

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ParticipantsAlreadyFull,
        ChitFundHasFinished,
        AlreadyJoined,
        CannotJoinMidCycle,
        OnlyOwnerCanBeginCycle,
        OnlyOwnerCanEndCycle,
        ChitFundNotFinished,
        NotParticipant,
        OnlyAdminCanDraw,
        ChitFundAlreadyFinished,
        FailedToGetWinner,
    }
    // pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct JoinedChitFund {
        #[ink(topic)]
        account: Option<AccountId>
    }

    #[ink(event)]
    pub struct FundDeposited {
        #[ink(topic)]
        account: Option<AccountId>,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct NewCycleBegan {
        #[ink(topic)]
        admin: Option<AccountId>,
    }

    #[ink(event)]
    pub struct CycleEnded {
        #[ink(topic)]
        admin: Option<AccountId>,
    }

    #[ink(event)]
    pub struct DrawWinner {
        #[ink(topic)]
        victor: Option<AccountId>,
        #[ink(topic)]
        amount_won: Balance,
    }

    impl ChitFund {
        #[ink(constructor)]
        pub fn new( admin : AccountId, max_participants: u32, monthly_contribution: Balance)
           
            -> Self {
            Self {
                admin : admin,
                max_participants,
                monthly_contribution,
                current_round: 1,
                pot: 0,
                total_amount: Default::default(),
                participants: Default::default(),
                used_indexes: Default::default(),
                finished: false,
            }
        }

        // The join function allows participants to join the chit fund.
        #[ink(message)]
        pub fn join(&mut self) -> Result<(), Error> {
            let participant = self.env().caller();
  
            if self.participants.len() >= self.max_participants.try_into().unwrap() {
            return Err(Error::ParticipantsAlreadyFull);
            }
            if self.finished { 
                return Err(Error::ChitFundHasFinished);
            }
            if self.participants.contains(&participant) { 
                return Err(Error::AlreadyJoined);
            }

            self.participants.push(participant);
            self.env().emit_event(JoinedChitFund {
                account: Some(participant),
            });
            Ok(())
        }

        #[ink(message)] 
        pub fn begin_cycle(&mut self) -> Result<(), Error> {
            let sender = self.env().caller();
            if sender != self.admin { 
            return Err(Error::OnlyOwnerCanBeginCycle);
            }     
            if !self.finished {
            return Err(Error::ChitFundNotFinished);
            }
            self.total_amount = self.pot;
            self.pot = 0;
            self.finished = false;
            self.env().emit_event(NewCycleBegan {
                admin: Some(sender), 
            });
            Ok(())
        }

        // The deposit function allows participants to deposit their 
        // monthly contribution into the chit fund's pot.
        #[ink(message, payable)]
        pub fn deposit(&mut self) -> Result<(), Error> {
            let sender = self.env().caller();
            if !self.participants.contains(&sender) { 
            return Err(Error::NotParticipant);
            }
            if self.finished { 
            return Err(Error::ChitFundHasFinished);
            }
            let transferred_balance = self.env().transferred_value();
            self.pot += transferred_balance;

            self.env().emit_event(FundDeposited {
                account: Some(sender),
                amount: transferred_balance,
            });
            Ok(())
        }
        // The draw function allows the admin to get a winner after the cycle is ended.
        #[ink(message, payable)]
            pub fn draw(&mut self) -> Result<(), Error> {
            if self.used_indexes.len() == self.participants.len() {
                self.used_indexes.clear();
            }
            let sender = self.env().caller();
            if sender != self.admin { 
                return Err(Error::OnlyAdminCanDraw);
            }
            if !self.finished {
                return Err(Error::ChitFundNotFinished);
            }
            let block_number = Self::env().block_number(); 
            if let Some(winner) = ChitFund::get_random_account(&mut self.participants, &mut self.used_indexes, block_number) {
                let amount = self.total_amount - self.pot;
               
                Self::env().transfer(winner, amount);
                self.env().emit_event(DrawWinner {
                    victor: Some(winner),
                    amount_won: amount,
                    
                });
                return Ok(())
            }
            return Err(Error::FailedToGetWinner);
        }
        
        //  To get a random account number for the winner
        fn get_random_account(participants: &mut Vec<AccountId>, used_indexes: &mut Vec<AccountId> ,block_number: u32) -> Option<AccountId> {
            if participants.is_empty() {
                return None;
            }
        
            let idx = (block_number as usize) % participants.len();
            let account_id = participants[idx];
            if used_indexes.contains(&account_id) {
                return None;
            }
            else {
                used_indexes.push(account_id);
                Some(account_id)
            }
        }

        // End a particular round after its completion
        #[ink(message)] 
        pub fn end_cycle(&mut self) -> Result<(), Error> {
            let sender = self.env().caller();
            if sender != self.admin { 
            return Err(Error::OnlyOwnerCanEndCycle);
            }     
            if self.finished {  
            return Err(Error::ChitFundAlreadyFinished);
            }
            self.total_amount = self.pot;
            self.pot = 0;
            self.current_round += 1;
                self.finished = true;
                self.env().emit_event(CycleEnded {
                    admin: Some(sender), 
                });
                Ok(())
        }
    }
}
    

#[cfg(test)]
mod tests {
    use crate::my_contract::{ChitFund, Error};
    // use ink_prelude::*;
    use super::*;
    use ink::primitives::AccountId;
    // use scale::{Encode, Decode};
    pub type Result<T> = core::result::Result<T, Error>;

    // Helper function to create a random account ID for testing purposes.
    fn random_account_id() -> AccountId {
        AccountId::from([0x42; 32])
    }

    #[test]
    fn test_new() {
        let admin = random_account_id();
        let max_participants = 5;
        let monthly_contribution = 100;
        let chit_fund = ChitFund::new(admin, max_participants, monthly_contribution);

        assert_eq!(chit_fund.admin, admin);
        assert_eq!(chit_fund.max_participants, max_participants);
        assert_eq!(chit_fund.monthly_contribution, monthly_contribution);
        assert_eq!(chit_fund.current_round, 1);
        assert_eq!(chit_fund.pot, 0);
        assert_eq!(chit_fund.total_amount, 0);
        assert_eq!(chit_fund.participants.len(), 0);
        assert_eq!(chit_fund.used_indexes.len(), 0);
        assert_eq!(chit_fund.finished, false);
    }
}

//     #[test]
//     fn test_join() {
//         let admin = random_account_id();
//         let max_participants = 5;
//         let monthly_contribution = 100;
//         let mut chit_fund = ChitFund::new(admin, max_participants, monthly_contribution);

//         // Test successful join.
//         let participant = random_account_id();
//         chit_fund.join();
//         assert_eq!(chit_fund.participants, vec![participant]);

//         // Test join after participants are full.
//         for i in 0..max_participants {
//             let participant = random_account_id();
//             assert_eq!(chit_fund.participants.len() as u32, i + 1);
//         }
//         let participant = random_account_id();
//         // ink::env::debug_println!("got a call from {:?}", chit_fund.join());

//         assert_eq!(chit_fund.join(), Err(Error::ParticipantsAlreadyFull));
//         // ink::env::debug_println!("value: {}", chit_fund.join().try_into().unwrap());

//         // Test join after chit fund is finished.
//         chit_fund.finished = true;
//         let participant = random_account_id();
//         assert_eq!(chit_fund.join(), Err(Error::ChitFundHasFinished));

//         // Test join when already joined.
//         let participant = chit_fund.participants[0];
//         assert_eq!(chit_fund.join(), Err(Error::AlreadyJoined));
//     }

//     #[test]
//     fn test_begin_cycle() {
//         let admin = random_account_id();
//         let max_participants = 5;
//         let monthly_contribution = 100;
//         let mut chit_fund = ChitFund::new(admin, max_participants, monthly_contribution);

//         // Test successful begin cycle.
//         chit_fund.finished = true;
//         chit_fund.total_amount = 500;
//         chit_fund.begin_cycle();
//         assert_eq!(chit_fund.pot, 0);
//         assert_eq!(chit_fund.finished, false);

//         // Test begin cycle with non-admin caller.
//         let non_admin = random_account_id();
//         assert_eq!(chit_fund.begin_cycle(), Err(Error::OnlyOwnerCanBeginCycle));

//         // Test begin cycle with chit fund not finished.
//         chit_fund.finished = false;
//         assert_eq!(chit_fund.begin_cycle(), Err(Error::ChitFundNotFinished));
//     }
// }


