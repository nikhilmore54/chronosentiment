# Release Artifact Quickstart

This operational bundle is entirely self-contained. 

To verify and bootstrap the release, run exactly this sequence:

```bash
shasum -a 256 -c SHA256SUMS
./chrono bootstrap
./chrono smoke
```

No further orchestration or environment setup is required.
