# Official WSLC fixtures

These fixtures originate from Microsoft's `wslc.exe` installed with WSL on
2026-09-04. Only insignificant JSON whitespace and the neutral fixture resource
name were normalized. `wslc version` reported `wslc 2.9.4.0`.

| Fixture | Command |
| --- | --- |
| `inspect-image-hello-world.json` | `wslc inspect --type image hello-world:latest` |
| `inspect-container.json` | `wslc inspect --type container wslc-schema-ut-20260904` |
| `inspect-network.json` | `wslc network inspect wslc-schema-ut-20260904` |
| `inspect-volume.json` | `wslc volume inspect wslc-schema-ut-20260904` |

The inspected image is the public `hello-world:latest` image. Keeping the
captured output in the repository makes the tests deterministic and allows
them to run without WSL, a container session, or network access.

The container, network, and volume use the neutral fixture name
`wslc-schema-ut-20260904`. The container configuration includes a published
port, an environment variable, a label, CPU and memory limits, a ulimit, and a
network so its inspection exercises nested schema definitions.
