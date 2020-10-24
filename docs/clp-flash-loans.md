# Flash Loans

Flash loans allow to borrow any available amount of assets without collateral. To make it safe, the borrower has to assure that the loan will be paid back withing the transaction.

CLP is a perfect utility for a flash loans. Liquidity pools are a natural entities providing flash loans.

On blockchains with fully composable smart contracts and sequential execution, like Ethereum this is simple. A smart contract providing a flash loan sends tokens to a receipient and calls a user defined defined smart contract. That usually initiate a sequence of other cross contract calls. At the end the lending smart contract checks if the assets were sent back. If not it will revert - causing the invalidating the whole smart contract and rolling back the changes (essentially the initial state of the balances), and making user to pay the gas fees only.

In a blockchain like NEAR, where cross smart contract calls are asynchronous and independent: cross smart contract call starts a new transaction (let's call them sub-transactions) with a programmed callback scheduled at the end of the called contract execution. Reverting a _sub-transaction_ doesn't revert the initial transaction. This means that a blockchain state has been already updated and committed. The scheduled callback can revert it's own internal state, but doesn't have a power to revert all the _sub-transactions_. This would require either:
* a permissioned and a trusted network of tokens with a complex system of permissions and limitation which contracts can be called using the lended assets.
* or an overhaul redesign how the smart contracts work.

Not having a solid and easy guarantee that we can roll-back a transaction with all it's sub-transactions makes **Flash Loans** insecure or extremely complex (which also increases the security risks) and limited.
