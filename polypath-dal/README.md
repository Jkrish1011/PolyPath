# PolyPath - Data Acquisition Layer
The data backbone of the system — interfaces with external bridges/DEX APIs.

### Purpose

- Fetch and normalize real-time data (fees, liquidity, latency, uptime).
- Support parallel async queries.
- Cache normalized results for reuse.


### Setup

Your config file should be as below:

```
[global]
update_interval=60
cache_ttl=120
log_level="info"

[bridges.stargate]
base_url=""
chains= ["ethereum", "polygon", "arbitrum"]
fees=0.0006

[bridges.wormhole]
base_url=""
chains= ["ethereum", "polygon", "arbitrum"]
fees=0.0006
guardian_count=19

[bridges.routerprotocol]
base_url = "https://api.routerprotocol.com"
chains = ["ethereum", "polygon", "avalanche"]
quote_endpoint = "/v2/quote"


[bridges.symbiosis]
base_url = "https://api.symbiosis.finance"
chains = ["bsc", "polygon", "ethereum"]
```