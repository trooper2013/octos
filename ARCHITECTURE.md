# Octos Phase 1 Architecture: User-Space Simulator

This document provides a technical deep-dive into the architectural primitives and execution mechanics of the Phase 1 simulator for **Octos**.

## System Topology

```mermaid
graph TD
    subgraph Core Orchestrator Bus
        BusRouter[start_router_loop]
        ActiveSenders[(arm_senders Map)]
        BusRouter <--> ActiveSenders
    end

    subgraph Persistent Subsystem Tasks (Arms)
        UI[start_ui_arm]
        Storage[start_storage_arm]
        Analysis[start_analysis_arm]
    end

    UI -->|IacPacket| BusRouter
    Storage -->|IacPacket| BusRouter
    Analysis -->|IacPacket| BusRouter

    BusRouter -->|Route via Channel| UI
    BusRouter -->|Route via Channel| Storage
    BusRouter -->|Route via Channel| Analysis
```

The simulator models a message-driven microkernel using:
1. **Tokio channels**: Serves as the point-to-point bus.
2. **Cooperative thread scheduling**: Run loop components execute concurrently.
3. **Rust thread safety**: State sharing via `Arc<RwLock<T>>`.

---

## 1. Point-to-Point Message Routing
The `OctosCore` orchestrator acts as a central hardware bus controller.
- When an Arm is registered via `register_arm(arm_info, arm_sender_channel)`, its `Uuid` and transmitter channel are added to an internal `HashMap` wrapped in a thread-safe `Arc<RwLock<...>>`.
- When an Arm invokes `route_packet(packet)`:
  - The packet is queued in the central orchestrator's receiver.
  - The orchestrator router loop (`start_router_loop`) dispatches the packet to the recipient Arm's local transmitter.

---

## 2. Persistent Arm Lifecycles

### A. Storage Arm
- Listens for `"SearchVectorFileSystem"` intents.
- Receives the semantic query vector.
- Queries `VectorStore` using **Cosine Similarity**:
  $$\text{Similarity}(\mathbf{A}, \mathbf{B}) = \frac{\sum A_i B_i}{\sqrt{\sum A_i^2} \sqrt{\sum B_i^2}}$$
- Returns a list of sorted matching `KnowledgeNode` records.

### B. Analysis Arm
- Simulates automated OS logic reasoning.
- When spreadsheet audit results are received, it executes rules to detect anomalously high transactions.
- Dispatches authorization requests to the User Interface Arm.
- Validates the confirmation token returned by the UI Arm.

### C. UI Arm & Dynamic Widgets
- Renders simulated layout configurations based on packet requests.
- Implements `render_dynamic_widget(intent, payload)`:
  - `"approve_payment"`: Triggers a mock terminal approval dialog.
  - `"select_photo"`: Triggers a mock photo picker dialog.
- Automatically generates secure, simulated biometric verification tokens (e.g. `TOKEN-VERIFY-9EFB0B8B`) to return to the caller.
