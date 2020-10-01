# Megapool

## Abstract

What if we are able to put all assets into a single pool, and find out a mechanism how to balance the pool with multiple assets. One approach would be to use a [Balancer](https://balancer.finance/) approach.


## Rationale


2 asset pool has few advantages: it's simple, stable and easy to model and reason about. In that model however, trading between assets will always involve some base currency. Moreover, using a single asset as a base currency for all other assets is not economically stable. It will create a big demand, which may not be justified by other means. This research follows the ideas developed by Uniswap v2 (2 asset pools without single base currency) and Balancer (multi asset pools with weighted power).

Observation: the world is a graph of inter-connected resources, not represented as pairs.


### One megapool

The __megapool_ is a contract with one multi-token pool - one pool for all assets. The idea was to use the Balancer model. In that pool, all assets are coupled and contribute to the value of the pool. If one asset gets more expensive, then naturally the value of the pools grows, and the rate of that asset to other assets increases as well.

The drawback is that each move in that pool will create imbalances which are difficult to reason about. So, for example, if someone will buy from the megapool GOLD using both NEAR and USD, it will impact also all other assets in the pool. We call it `transitional` imbalance (looking for a better name proposal).
Based on the observation above, this is a logical behavior. For example, ff in the real world, someone buys a lot of crude oil, it will impact all the supply chains, even USD (proportionally to the whole USD market). So to make a global wide noticeable imbalance, the trade have to be significant.

There are few major difficulties in this model:
+ it requires deep liquidity in the pool to reduce the  transitional imbalances.
+ adjusting correct weights based on the combination of all assets in the pool is not trivial.



### Pools with 2 asset pool and odd weights

Keep 2 assets pool with same base currency (NEAR), but introduce weights as Balancer is doing. In this model we will increase the weight of the base currency based on the total value of the CLP contract.
