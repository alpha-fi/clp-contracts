# OTC trading

Bonding curve AMM are inherently limited. They are "bounded" to the underlying, deterministic function. This means that the price function can't adapt easily to the market situation:

* Slippage is written in the code, and depends only on the given trade volume and pool size at that time. It doesn't reflect the market situation (stability, investor actions).
* LP position is always long on the depreciating asset which causes the impermanent loses.
* Shifting revenues from arbitrageurs to LP is tricky and usually requires additional incentives.

Over time, competitive traders will get most of the benefits from bonding curve AMMs, because they will have an in-depth market knowledge, whereas the AMMs itself will either continue to be dumb or will have a slow reaction speed through a questionable governance (questionable because we don't know what interest the governance will represent: whales, dapps or "common users").

In the image below, on the left we see a model assumed by a bonding curve AMM. It can't handle any phenomena. On the right we see a model representing a real world traders.

![bonding curve vs OTC trading model](https://miro.medium.com/max/700/0*m45BpNQaKpVC_jRY)

## Order books

The alternative to bondig curve AMMs is **Order Books** (with big enough volume and good customer driven UX, supported by analytics and easy mechanics), is known to be the most efficient trading venue. Order books are not limited to any price function, they allow to naturally react to different market situations. Moreover, when done properly, they can **drive traders** to make more secure or trades using more sophisticated trade conditions.

Why order book exchanges are not popular in blockchain?

1. Doing it fully on chain is expensive (storage and order book ordering costs).
1. Since layer-1 is usually not enough (see point 1. and the note below), many existing solutions will require additional layers, complexifying user experience.
1. Layer-2 / Off-chain solutions don't compose well with the DeFi lego blocks.

NOTE: some blockchains have design a high performance layer 1 (trading decentralization for performance) while keeping good enough security. One of them, Solana,



## Hybrid Order Books

To overcome a circumstances mentioned above, DeFi has to evolve and adapt an intelligent mechanisms. AMMs must be smarter to approximate humans and compete with centralized market makers.
Order Books give a room to capture this intelligence. But how can we combine the off-chain knowledge with on-chain execution?

Ox is working on similar models for a long time, and unfortunately there is no much adoption. But if the evolution would come directly from the exchanges then it would probably drive the usage and adoption.


### AMM with signature-based pricing function

We can use a verified oracles (which provide a signed data on chain) from exchanges, to represent an order-book off chain, but trades on-chain. We can call it a signature-based pricing function.
Basically AMM can bridge prices and liquidity between centralized exchanges and DeFi.

This is how it will work on the DeFi side:

* AMM has some liquidity
* AMM admin provides a price based on a whole market knowledge
* User can see a price and decide if he likes it or not. He can make a trade request which will be only executed if some conditions are met (price, volume...)

This can work perfectly for applying a sophisticated exchange models. Moreover the AMM is trustless (user doesn't need to trust anyone to be sure that his request is correctly processes) but exposes few risks:

+ frontrunning
+ price manipulation - in case the AMM is used as a price discovery with other smart contracts.
+ AMM may have a losses if they don't react fast enough on price changes.

Mitigations:

+ Add TTL for the currently submitted price
+ TTL can be combined with increased fee (eg, if you execute a trade in 1s after price update, you will have `1.1 * base_fee`, if you execute after 2s you will have `1.5*base_fee`...)


### Dynamic fee structure

Most of the AMM follow the Uniswap static (or semi-static => controlled by a governance) fee model. The big downside of this is that it yields high rewards for arbitrageurs during a high volatility time, while not shifting any of that premiums to Liquidity Providers.

Moreover, as noted above, the fee doesn't depend on any pattern of asset prices. More specialized market makers can be smarter in how they price assets, which gives them room to lower their fees. Fees should react on market volumes and shift as much of the profits as possible to LP.

Dynamic fees is an idea of allowing an algorithm to adjust the fees based on the current activity and, possible, a market knowledge (through some oracles).


## References

+ [CoFiX](https://medium.com/dragonfly-research/introducing-cofix-a-next-generation-amm-199aea686b6b)
