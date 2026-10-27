# v0 supports omp only; no provider abstraction layer

Decision: papago v0 drives omp alone and uses its distinctive capabilities fully (rpc-ui Ask bridging, subagent subscriptions, slash-command discovery, host tools). No multi-provider interface is defined. The "RPC events → Timeline" mapping lives in a dedicated package, which is the physical seam where a future Pi/ACP provider would slot in. Rationale: first-class omp depth beats a generic any-CLI wrapper (the wedge identified in the ADE landscape); an abstraction layer is pure liability until a second provider is real.
