# NEARswap

NEARswap is the first from the NEAR-CLP protocols family. It provides the following features:

* Automated Market Maker. Arbitragers and exchanges can use the NEARswap Liquidity Pools.
* Continuous Liquidity Pools. Users have a constant access to a pool resources.
* Liquidity Pool Shares market. Users own shares of a pool and can trade it.
  We created a multi-token standard for LP Shares tokens.

* Price discovery mechanism. AMM is an efficient market to balance the token ratios. We design a time wrapped contracts to provide solid data to the market, with a limited risk for price manipulators.

**TBD**: we are working on few more features directly linked to the CLP which will be disclosed later.


## Usage

Go to the [NEAR-CLP usage](/docs/nearclp-usage.md) page.


## Design Principles

1. NEARswap is a common good. Open and accessible for everyone. Hence we want to make it as predictable and as safe as possible.
1. We start with Uniswap v2 market model as a solid and proven base. Each trade will incur fees which will be shared with liquidity providers.
   1. Each pool is based only on two different tokens, one of the tokens must be NEAR.
   1. NEAR is the base currency for each pool, which increases the utility function of the NEAR tokens.
      NEAR is the most liquid asset in the pool. It's possible to easily move NEAR between pools, which opens few interesting features which we will discuss later.

1. We keep all pools under the same smart contract, which brings many advantages:
   * simplifies implementation
   * reduces attack vectors
   * Logically, NEARswap is a one big NEAR holder, which, again, opens few interesting features to research.

   Note: This design will limit the a capabilities of the contract (all pools will stay in the same shard). However we don't see it as a problem. Each token contract will already be evenly distributed. And our implementation design provides better efficiency for the protocols we are designing.

1. We are aware about the [Impermanent Loss](https://medium.com/@pintail/uniswap-a-good-deal-for-liquidity-providers-104c0b6816f2) problems and we are working on new trading models for AMM. In a volatile market, the basic fee model (as the one defined by Uniswap) is not enough to cover the Impermanent Losses. Through investigation of many approaches drafted the evolution of the protocol and will probably implement the Thorchain CLP trading model.

We will iterate on the above model, and we are already designing new mechanism for pool management and balancing which will not depend on a fixed fee and Uniswap v2.


#### Future design goals

+ We admit the current limitation of the NEP-21 standard and want to work with anyone who will like to promote a better standard. As a first step we are creating a multi-token standard.




## On a Common Goods goals

Common goods are defined in economics as goods that are rivalrous and non-excludable. [Wikipedia](https://en.wikipedia.org/wiki/Common_good_(economics)).
Common Goods must be:
+ accessible to everyone
+ be inclusive
+ don't create or be based on a ponzi scheme.
+ be limited and controlled to not be used to increase financial classes differences.

As discussed below, we don't see any functioning governance model which is __open_ and __thriving_  and efficiently working in the current DeFi ecosystem.

That being said we are working on alternative models to improve the reward system and fight the Impermanent Losses.

## Notes about incentives utility tokens


Liquidity Providers (LPs) are essential to the CLP system to function. However, in the open finance world, there is lot of competition for asset management services. Why LPs should park their funds in CLP? They could use assets for staking, lending and other protocols. To make the system thriving we need to define correct and sustainable ecosystem, where LPs will feel secure to park their funds.

The dominant mechanisms to keep the incentives for LPs are: fee share scheme (discussed above), governance tokens, yield farming.


### Governance


Creating a reliable and sustainable governance in a form of DAO is yet to be proven. Many big projects failed to show a governance model which is both: **open** and **thriving**. Most of the current solutions deliver either one or none:

+ Uniswap v1: no governance.
+ Uniswap v3: introduced governance tokens, we yet to see how it will work.
+ Compound: on chain governance and on chain execution. Users vote on the contract change, not an idea, and on the logic swap, users have 24h to move the funds if they don't agree. Very good approach, however it's not thriving.
+ Sushiswap: governance with off-chain execution. Users votes on an idea, and don't have a control on the execution and logical changes.
+ Tezos: on chain governance and on chain execution. The process is very slow and so far limited to the core team or related organizations.

Recently we've seen many protocols with governance yield farming tokens. So far they are didn't show a grounded value. Instead we have seen a speculators rush on money:

+ Uniswap offering USD 1k worth of gov tokens to [Unisock](https://unisocks.exchange/) holders (NFT minted for users who bought Uniswap socks)
+ Sushiswap created
+ Food named coins with a ponzi gov token distribution scheme: each one is offering a migration bounty in a form of 10x based on the parent protocol bounty. It's easy to see that this is a house of cards.

Many of this tokens don't have a solid product-market fit nor proven unique value proposition.

There is a very nice [article](https://www.tokendaily.co/newsletter/21371e5354) summarizing the investors landscape of DeFi market related to recently popular governance tokens:
*"the major investors and backers of popular DeFi protocols are essentially the same small group of investors. Compare that to the list of top 10 shareholders of JPMorgan and BoA above: you’ll quickly see that the game is not that different. Governance tokens can open a door of re-centralization that many of us believed were shut. Ironically, initiatives that aim to distribute control can often lead to even more centralization."*

Let's be aware that a governance token posses a power to adapt the protocol towards more trading profits. Gov tokens can become a playground for gamification and market manipulation. These activities, when done with a right precision, could be extremely hard to observe.

In summary: today DeFi projects didn't deliver a solid governance mechanism, instead they created an **augmented speculation scheme** in a form of _limited_ governance tokens. Instead of shared responsibility and shareholders principals, the gov tokens are mainly being used in trading venues, making the traders, rather than the users, to controlling the market

Maybe in the future the space can mature. But we don't see that future yet.


### How about Yield Farming?

We believe that Yield Farming can create additional incentives for powerful entities who are able to manipulate the market. Yield Farming, when not done correctly:

- disturb the balance in the economy: it inevitably provides new ways how people can use their power to get even more power and move the market to their side.
- it makes the "trustless" aspect of the blockchain less trustless
- it's a gravity force for opportunistic users
- creates more instability in the market.

Here is a great overview of strategies and values of [liquidity mining](https://medium.com/bollinger-investment-group/liquidity-mining-a-user-centric-token-distribution-strategy-1d05c5174641).

We want to assure economy-safe conditions in NEARswap protocol, hence we will put lot of effort to make a solid incentives scheme before shipping any yield farming mechanism.

### Summary

Most of DeFi is currently money out of the air, with arguable utility provided. IMHO, a good, AMM protocol doesn't need any governance. Market should decide about strategies. If you have better strategy, you modify a smart-contract code, deploy it and let users decide to move the funds. No need for governance tokens. And in fact I prefer an AMM without governance because I can trust it.
