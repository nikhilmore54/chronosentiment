# Frequently Asked Questions

**Q: Can I connect ChronoSentiment directly to Binance or a live broker websocket?**
A: No. ChronoSentiment is not a live trading engine. You must use a separate live capture tool to save the stream to a JSONL substrate, and then replay that substrate through ChronoSentiment.

**Q: Why doesn't the CLI have progress bars or colorized success emojis?**
A: We optimize for mechanical honesty and CI/CD integration. Output is strictly `[OK]`, `[FAIL]`, and `[WARN]`.

**Q: How do I run this on a Kubernetes cluster with multiple nodes?**
A: ChronoSentiment Core is a single-host verification instrument. Distributed orchestration is explicitly out of scope.

**Q: What happens if a trace file exceeds my disk capacity?**
A: The engine protects the host by automatically truncating raw JSON traces at 500,000 events. The cryptographic metadata hash remains fully valid for the entire sequence.

**Q: Where did the documentation about market simulations and execution ecology go?**
A: Historical research, conceptual lineage, and exploration for future systems are strictly segregated in `docs/research/`. They are not part of the active operational core.
