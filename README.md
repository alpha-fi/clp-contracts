![logo](/assets/logo-clp.png)


NEAR-CLP is a set of protocols for decentralized finance. With a ground research and solid experience in smart-contract development we define our mission as:

> To develop a sustainable foundation of DeFi ecosystem with NEAR blockchain

We are creating building blocks for the DeFi which are secure, reliable and sustainable.

We use **Ethereum-NEAR Rainbow Bridge** to provide better service for Ethereum assets. Check the note [below](#why-bridging-with-ethereum) for more details.


## Our protocols


#### [NEARswap](/docs/nearswap.md)

* Automated Market Maker
* Continious Liquidity Provider
* [OTC Trading](/docs/otc-trading.md)

Status: Alpha. Basic functionality is implemented and tested. However there are few known exploits - we don't handle all edge cases yet. Please use it for testing and playing.

Demo:

* [Video 1](https://www.dropbox.com/s/s1oiasb11qz8trv/NEAR-hack-the-rainbow-demo.ogv?dl=0): Rainbow bridge integration, moving ERC20 to NEARswap smart-contracts, creating NEARswap pool, adding liquidity to the pool, and trading tokens with the pool using the WEB UI.
* [Video 2](https://www.youtube.com/watch?v=DXV5Fa-r2UE)  - Command Line Interface.


#### [Flash Loans](/docs/clp-flash-loans.md)

TL;DR: this is not feasible in current design of NEAR blockchain.

#### BCO: Bonding-curve Coin Offering

* Did you hear about ICO? Or Uniswap Offering? BCO is moving this to a next level.
* An automated token release mechanism.

Status: research & development.


#### TBD...

We have few garage products which we will share later.

### Research

We analyze different approaches (slowly releasing a content there) in the [research](/docs/research.md) document.


## Usage

Please refer to the specific protocol documentation page to see the available tools and webapp functionality. Here are the principal tools:

* our webapp (provides the integration for basic features)
* [near-cli](https://github.com/near/near-cli) - a tool to make generic transactions and operations with NEAR blockchain runtime.
* [near-clp-cli](https://github.com/luciotato/near-clp-beta-cli/) - a CLI crafted for NEARswap API.


## Why and How?

### DeFi Background

The structure of global finance has remained largely the same since the start of the Industrial era, with a heavy reliance on financial intermediaries. Despite major advancements in software technology, huge parts of existing financial infrastructure remain archaic and burdensome. While many fintech products have sprouted into existence over the last decade to fill the gap, they all need to be plugged into the existing financial system to function, leaving a control of OUR funds and charging intermediation fees at the incumbents.

The benefits of DeFi will be major across both developed and emerging markets. In developed markets, DeFi will provide greater choice to consumers, reduce the costs of the legacy financial system, and bring greater liquidity and product innovation to financial markets. For emerging markets, DeFi will provide a more secure store of value with the rise of stablecoins and offer financial services to billions of adults who lack access to the existing financial system.

By transferring the trust layer from financial intermediaries to software and code on the blockchain, DeFi can provide universal access to financial services. This represents a paradigm shift where trust-minimized user experience is possible. In an era where control of users’ data and digital activity might be a liability, DeFi will be able to provide much better products and services at scale than traditional finance.

(Source: The Defiant)


### Macro Economy

Economy heavily depend on the following factors:

- production
- supply chains
- financial services

The latter one enable growth of our economy to new levels through all aspects of asset management and financial derivatives. At its heart, financial services arrange everything from savings accounts to synthetic collateralized debt obligations. With that, access to assets and obligations is a key to scale assets management.

With blockchain we can move financial services to a new level - Decentralized Finance. As noted above, access to assets obligation is a key to scale the economy. Principal solutions to that is:

- Liqidity services
- Automated Market Making.
- Protocol level investments: own your funds while letting them working for you through DeFi protocols.
- Direct payments: remove the market dependency from the huge tech unicorns (Google, Amazon, FB...) having an access to your wallet and getting intermediation fees, when in most cases it's not necessary.

### How

We build `NEAR-CLP` to fulfill this goals in a very open and decentralized manner. Here are our **GOALS**:

- Focus on liquidity pools and AMM
- Eliminate side markets incentives. Side market is an entity which could change a behavior of the CLP / AMM protocol and shift the benefits or potentially manipulate the whole market.
- Highly predictable behavior designed with the main blockchain principles: trustless smart-contracts without susceptible governance.



## Why NEAR

Thriving DeFi ecosystem needs:

1. Robust layer-1 to enable composability between different contracts (→ DeFi Lego).
2. Secure platform with adjustable throughput and storage capabilities. With DeFi we can't take a risk of unreliable or corruptible layer-2.

NEAR is providing a solution for above requirements today. With it's unique set of features and scaling potential aspires to be a significant player in the new Decentralize Finance world.


### Why Bridging with Ethereum?

Today, most of the valuable assets are hosted on Ethereum. In fact, today, DeFi is an Ethereum domain and it has the biggest ecosystem (developers, tools, dapps). It's also very secure. However Ethereum has few very important pain points:

- it doesn't scale
- layer-2 solutions are complex and not composable
- EVM execution model is unsafe

We want to enable people to use Ethereum based assets and do asset management on NEAR using NEAR growing ecosystem. Through the inter-blockchain **bridge**, NEAR will provide a scale-ability to the current Ethereum DeFi ecosystem and more robust smart-contract runtime.



## License

We create foundations for DeFi, hence an open access is the key.

We build the Open Source software and knowledge base. All our work is licensed with a Mozilla Public License (MPL) version 2.0. Refer to [LICENSE](LICENSE) for terms.

The MPL is a simple copyleft license. The MPL's "file-level" copyleft is designed to encourage contributors to share modifications they make to your code, while still allowing them to combine your code with code under other licenses (open or proprietary) with minimal restrictions.
Q2: Why yet another open source license?

The MPL fills a useful space in the spectrum of free and open source software licenses, sitting between the Apache license, which does not require modifications to be shared, and the GNU family of licenses, which requires modifications to be shared under a much broader set of circumstances than the MPL. [FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).



## Contact

+ Leave a note using github issues.
+ [Contributors](https://github.com/robert-zaremba/near-clp/graphs/contributors)


#### Hack the Rainbow team

[Hack the Rainbow ](https://near.org/blog/hack-the-rainbow-%F0%9F%8C%88-nears-first-massive-open-online-hackathon-aka-mooh/) is the NEAR’s first Massive Open Online Hackathon.

+ [Robert Zaremba](https://zaremba.ch/contact.html)
+ [Lucio Tato](https://github.com/luciotato)
+ [Amit Yadav](https://github.com/amityadav0)
+ [Jameson Hodge](https://github.com/jamesondh)
