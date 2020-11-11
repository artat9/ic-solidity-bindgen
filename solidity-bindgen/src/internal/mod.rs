use super::Context;
use std::marker::Unpin;
use web3::contract::tokens::{Detokenize, Tokenize};
use web3::contract::Contract;
use web3::transports::Http;
use web3::types::{Address, TransactionReceipt};

mod types;
pub use types::*;

/// Mostly exists to map to the new futures.
/// This is the "untyped" API which the generated types will use.
pub struct ContractWrapper {
    contract: Contract<Http>,
    context: Context,
}

impl ContractWrapper {
    pub async fn call<T: Detokenize + Unpin>(
        &self,
        name: &'static str,
        params: impl Tokenize,
    ) -> Result<T, web3::Error> {
        match self
            .contract
            .query(
                name,
                params,
                Some(self.context.from()),
                Default::default(),
                None,
            )
            .await
        {
            Ok(v) => Ok(v),
            Err(e) => match e {
                web3::contract::Error::Api(e) => Err(e),
                // The other variants InvalidOutputType and Abi should be
                // prevented by the code gen. It is useful to convert the error
                // type to be restricted to the web3::Error type for a few
                // reasons. First, the web3::Error type (unlike the
                // web3::contract::Error type) implements Send. This makes it
                // usable in async methods. Also for consistency it's easier to
                // mix methods using both call and send to use the ? operator if
                // they have the same error type. It is the opinion of this
                // library that ABI sorts of errors are irrecoverable and should
                // panic anyway.
                e => panic!("The ABI is out of date. Name: {}. Inner: {}", name, e),
            },
        }
    }

    pub async fn send(
        &self,
        func: &'static str,
        params: impl Tokenize,
    ) -> Result<TransactionReceipt, web3::Error> {
        self.contract
            .signed_call_with_confirmations(
                func,
                params,
                Default::default(),
                // Num confirmations. From a library standpoint, this should be
                // a parameter of the function. Choosing a correct value is very
                // difficult, even for a consumer of the library as it would
                // require assessing the value of the transaction, security
                // margins, and a number of other factors for which data may not
                // be available. So just picking a pretty high security margin
                // for now.
                24,
                self.context.secret_key(),
            )
            .await
    }

    pub fn new(
        contract_address: Address,
        context: &Context,
        json_abi: &[u8],
    ) -> Result<Self, web3::error::Error> {
        let context = context.clone();

        // All of the ABIs are verified at compile time, so we can just unwrap here.
        // See also 4cd1038f-56f2-4cf2-8dbe-672da9006083
        let contract = Contract::from_json(context.eth(), contract_address, json_abi).unwrap();

        Ok(Self { contract, context })
    }
}
