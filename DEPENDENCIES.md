# DEPENDENCIES — openconstruct-kernel

## Signal Chain Layer

**L0 (Hardware) — Sensor Detection & Tick Processing**

The hardware layer. Detects sensors, processes raw ticks, and feeds structured data into the plato-nervous signal chain.

## Ecosystem Dependencies

| Repo | Relationship | Description |
|------|-------------|-------------|
| [plato-nervous](https://github.com/SuperInstance/plato-nervous) | **Depended on by** | Consumes raw tick data and hardware metadata for deadband/L0 processing |
| [plato-vision-jepa](https://github.com/SuperInstance/plato-vision-jepa) | **Related** | May receive camera hardware detection from openconstruct-kernel |
| [plato-audio-jepa](https://github.com/SuperInstance/plato-audio-jepa) | **Related** | May receive microphone hardware detection from openconstruct-kernel |
| [hermit-crab](https://github.com/SuperInstance/hermit-crab) | **Related** | Hardware context informs agent migration decisions |

## Data Flow

```
IN:
  - Physical hardware (cameras, microphones, GPIO sensors)
  - System bus enumeration

OUT:
  - Raw sensor ticks with timestamps
  - Hardware capability metadata
  - Device availability/change notifications
```
