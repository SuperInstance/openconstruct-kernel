# openconstruct-kernel

The actual kernel that ties ForgeFlux metabolism to hardware. **Same kernel, every scale.**

From ESP32 to cloud — one kernel, hardware-adaptive rooms, tick-based processing, and T-minus predictive coordination.

## Concepts

- **Rooms**: Named spaces with sensors, agents, and tick counters
- **Metabolism**: Tracks tiles processed, conservation ratio (CR), decomposer/transform counts
- **T-Minus Coordinator**: Predictive event system — predict, confirm, adapt
- **Transport**: Local, HTTP, MQTT, A2A, or Git — same kernel, different transport
- **Hardware Detection**: Automatically profiles the host and caps room count accordingly

## Hardware Profiles

| Class | Cores | RAM | Rooms |
|-------|-------|-----|-------|
| ESP32 | ≤2 | ≤1GB | 1 |
| Jetson | ≤6 | ≤16GB | 6 |
| Desktop | ≤16 | — | 100 |
| Cloud | 16+ | — | 1000+ |

## Usage

```rust
use openconstruct_kernel::{Kernel, Sensor, SensorKind, Transport};

let mut kernel = Kernel::with_rooms(4);
kernel.add_room("bridge", vec![Sensor {
    kind: SensorKind::Temperature,
    reading: 22.0,
    unit: "°C".into(),
    last_reading_ms: 0,
    deadband: 0.5,
}]);

let ticks = kernel.process_tick(Sensor {
    kind: SensorKind::Temperature,
    reading: 25.0,
    unit: "°C".into(),
    last_reading_ms: 1000,
    deadband: 0.5,
});

let predictions = kernel.predict_events();
let status = kernel.status();
```

## Ecosystem

openconstruct-kernel is the **hardware layer (L0)** of the PLATO Nervous System.

**Where this sits:** Layer 0 (hardware). Detects sensors, processes raw ticks, and feeds structured data into [plato-nervous](https://github.com/SuperInstance/plato-nervous) for deadband filtering and the signal chain.

**Signal chain:**
```
Physical hardware → openconstruct-kernel (ticks) → plato-nervous (deadband L0) → ...
```

| Repo | Role |
|------|------|
| [plato-nervous](https://github.com/SuperInstance/plato-nervous) | Core signal chain — consumes raw tick data from this crate |
| [plato-vision-jepa](https://github.com/SuperInstance/plato-vision-jepa) | Vision perception — may receive camera hardware detection |
| [plato-audio-jepa](https://github.com/SuperInstance/plato-audio-jepa) | Audio perception — may receive microphone hardware detection |
| [concrete-token-demo](https://github.com/SuperInstance/concrete-token-demo) | CLI demo of the full pipeline |
| [plato-browser](https://github.com/SuperInstance/plato-browser) | Browser-native demo |
| [luciddreamer-ai](https://github.com/SuperInstance/luciddreamer-ai) | Cloud-layer reactive podcast engine |
| [hermit-crab](https://github.com/SuperInstance/hermit-crab) | Agent migration — hardware context informs migration decisions |

See [DEPENDENCIES.md](./DEPENDENCIES.md) for detailed dependency and data flow information.

## License

MIT
