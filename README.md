# Octos Simulator Daemon

Octos is a bare-metal, AI-first operating system framework. This repository contains the Phase 1 simulator: a user-space daemon that acts as an OS overlay layer to test core architectural primitives in Rust.

## Workspace Crates
- **`octos-core`** (Binary): The central daemon and asynchronous message routing orchestrator.
- **`octos-iac`** (Library): The Inter-Arm Communication protocol definitions and serialization structures.
- **`octos-storage`** (Library): The non-hierarchical vector filesystem simulator utilizing cosine similarity search.

## Running the Simulator

Ensure you have a recent version of the Rust toolchain installed. To build and run the simulation, execute:

```bash
cargo run
```

This will:
1. Spin up a multi-threaded Tokio runtime.
2. Initialize the `VectorStore` with mock knowledge nodes representing documentation, code, and user data.
3. Register mock Arms: Storage Arm, Logic Arm, and UI Arm.
4. Trigger a simulated user semantic query goal.
5. Asynchronously route IAC packets through the channel-based message bus.
6. Perform real-time cosine similarity search on the Vector File System.
7. Print detailed execution tracing.
