use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kernel {
    pub rooms: Vec<Room>,
    pub metabolism: Metabolism,
    pub coordinator: TMinusCoordinator,
    pub transport: Transport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub name: String,
    pub sensors: Vec<Sensor>,
    pub agents: Vec<String>,
    pub tick_count: u64,
    pub last_tick_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    pub kind: SensorKind,
    pub reading: f64,
    pub unit: String,
    pub last_reading_ms: u64,
    pub deadband: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorKind {
    Temperature,
    Pressure,
    Gps,
    Sonar,
    Compass,
    Rpm,
    Fuel,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metabolism {
    pub tiles_processed: u64,
    pub cr_avg: f64,
    pub decomposer_count: usize,
    pub transform_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TMinusCoordinator {
    pub predictions: Vec<Prediction>,
    pub confirmations: u64,
    pub adaptations: u64,
    pub message_savings: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub event: String,
    pub predicted_at_ms: u64,
    pub confidence: f64,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transport {
    Local,
    Http,
    Mqtt,
    A2a,
    Git,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub name: String,
    pub ram_mb: u64,
    pub cores: u32,
    pub gpu: bool,
    pub rooms_supported: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub room_name: String,
    pub sensor_kind: SensorKind,
    pub value: f64,
    pub timestamp_ms: u64,
    pub delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelStatus {
    pub room_count: usize,
    pub sensor_count: usize,
    pub total_ticks: u64,
    pub cr_avg: f64,
    pub predictions_pending: usize,
    pub adaptations: u64,
    pub hardware: HardwareProfile,
    pub transport: Transport,
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {
            rooms: Vec::new(),
            metabolism: Metabolism {
                tiles_processed: 0,
                cr_avg: 1.0,
                decomposer_count: 0,
                transform_count: 0,
            },
            coordinator: TMinusCoordinator {
                predictions: Vec::new(),
                confirmations: 0,
                adaptations: 0,
                message_savings: 0.0,
            },
            transport: Transport::Local,
        }
    }

    pub fn with_rooms(n: usize) -> Self {
        let mut kernel = Self::new();
        let hw = Self::detect_hardware();
        let count = n.min(hw.rooms_supported);
        for i in 0..count {
            kernel.rooms.push(Room {
                name: format!("room-{}", i),
                sensors: Vec::new(),
                agents: Vec::new(),
                tick_count: 0,
                last_tick_ms: 0,
            });
        }
        kernel
    }

    pub fn detect_hardware() -> HardwareProfile {
        let cores: u32 = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);

        // Heuristic: assume ~2GB per core for RAM estimate
        let ram_mb = cores as u64 * 2048;

        let (name, rooms_supported) = if cores <= 2 && ram_mb <= 1024 {
            ("ESP32-class".into(), 1)
        } else if cores <= 6 && ram_mb <= 16384 {
            ("Jetson-class".into(), 6)
        } else if cores <= 16 {
            ("Desktop-class".into(), 100)
        } else {
            ("Cloud-class".into(), 1000)
        };

        HardwareProfile {
            name,
            ram_mb,
            cores,
            gpu: false,
            rooms_supported,
        }
    }

    pub fn add_room(&mut self, name: &str, sensors: Vec<Sensor>) {
        self.rooms.push(Room {
            name: name.to_string(),
            sensors,
            agents: Vec::new(),
            tick_count: 0,
            last_tick_ms: 0,
        });
    }

    pub fn process_tick(&mut self, sensor_reading: Sensor) -> Vec<Tick> {
        let mut ticks = Vec::new();
        let now = sensor_reading.last_reading_ms;

        for room in &mut self.rooms {
            for sensor in &mut room.sensors {
                // Only process if the sensor kind matches
                let matches = match (&sensor.kind, &sensor_reading.kind) {
                    (SensorKind::Custom(a), SensorKind::Custom(b)) => a == b,
                    (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
                };
                if matches {
                    let delta = (sensor_reading.reading - sensor.reading).abs();
                    if delta > sensor.deadband {
                        ticks.push(Tick {
                            room_name: room.name.clone(),
                            sensor_kind: sensor.kind.clone(),
                            value: sensor_reading.reading,
                            timestamp_ms: now,
                            delta,
                        });
                    }
                    sensor.reading = sensor_reading.reading;
                    sensor.last_reading_ms = now;
                }
            }
            room.tick_count += 1;
            room.last_tick_ms = now;
        }

        self.metabolism.tiles_processed += ticks.len() as u64;
        self.metabolism.transform_count += 1;

        ticks
    }

    pub fn predict_events(&mut self) -> Vec<Prediction> {
        let mut new_predictions = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        for room in &self.rooms {
            if room.tick_count > 0 && room.tick_count % 10 == 0 {
                let pred = Prediction {
                    event: format!("{}: periodic checkpoint", room.name),
                    predicted_at_ms: now + 5000,
                    confidence: 0.85,
                    confirmed: false,
                };
                new_predictions.push(pred.clone());
                self.coordinator.predictions.push(pred);
            }
        }

        new_predictions
    }

    pub fn confirm(&mut self, event: &str) -> bool {
        for pred in &mut self.coordinator.predictions {
            if pred.event == event && !pred.confirmed {
                pred.confirmed = true;
                self.coordinator.confirmations += 1;
                return true;
            }
        }
        false
    }

    pub fn adapt_if_needed(&mut self, actual_cr: f64) -> bool {
        if actual_cr < 0.5 {
            self.coordinator.adaptations += 1;
            self.metabolism.cr_avg = actual_cr;
            true
        } else {
            self.metabolism.cr_avg = actual_cr;
            false
        }
    }

    pub fn status(&self) -> KernelStatus {
        let sensor_count: usize = self.rooms.iter().map(|r| r.sensors.len()).sum();
        let total_ticks: u64 = self.rooms.iter().map(|r| r.tick_count).sum();
        let predictions_pending = self
            .coordinator
            .predictions
            .iter()
            .filter(|p| !p.confirmed)
            .count();

        KernelStatus {
            room_count: self.rooms.len(),
            sensor_count,
            total_ticks,
            cr_avg: self.metabolism.cr_avg,
            predictions_pending,
            adaptations: self.coordinator.adaptations,
            hardware: Self::detect_hardware(),
            transport: self.transport.clone(),
        }
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_new() {
        let k = Kernel::new();
        assert!(k.rooms.is_empty());
        assert_eq!(k.metabolism.tiles_processed, 0);
        assert_eq!(k.metabolism.cr_avg, 1.0);
        assert!(k.coordinator.predictions.is_empty());
        assert!(matches!(k.transport, Transport::Local));
    }

    #[test]
    fn test_kernel_default() {
        let k = Kernel::default();
        assert!(k.rooms.is_empty());
    }

    #[test]
    fn test_detect_hardware() {
        let hw = Kernel::detect_hardware();
        assert!(!hw.name.is_empty());
        assert!(hw.cores >= 1);
        assert!(hw.rooms_supported >= 1);
    }

    #[test]
    fn test_with_rooms() {
        let hw = Kernel::detect_hardware();
        let k = Kernel::with_rooms(hw.rooms_supported + 10);
        assert_eq!(k.rooms.len(), hw.rooms_supported); // capped
    }

    #[test]
    fn test_with_rooms_small() {
        let k = Kernel::with_rooms(2);
        assert!(k.rooms.len() <= 2);
    }

    #[test]
    fn test_add_room() {
        let mut k = Kernel::new();
        k.add_room("bridge", vec![]);
        assert_eq!(k.rooms.len(), 1);
        assert_eq!(k.rooms[0].name, "bridge");
    }

    #[test]
    fn test_add_room_with_sensors() {
        let mut k = Kernel::new();
        let sensor = Sensor {
            kind: SensorKind::Temperature,
            reading: 22.0,
            unit: "°C".into(),
            last_reading_ms: 1000,
            deadband: 0.5,
        };
        k.add_room("engine", vec![sensor]);
        assert_eq!(k.rooms[0].sensors.len(), 1);
    }

    #[test]
    fn test_process_tick_no_delta() {
        let mut k = Kernel::new();
        let sensor = Sensor {
            kind: SensorKind::Temperature,
            reading: 22.0,
            unit: "°C".into(),
            last_reading_ms: 1000,
            deadband: 5.0,
        };
        k.add_room("lab", vec![sensor]);
        let reading = Sensor {
            kind: SensorKind::Temperature,
            reading: 22.1,
            unit: "°C".into(),
            last_reading_ms: 2000,
            deadband: 5.0,
        };
        let ticks = k.process_tick(reading);
        assert!(ticks.is_empty()); // delta 0.1 < deadband 5.0
        assert_eq!(k.metabolism.transform_count, 1);
    }

    #[test]
    fn test_process_tick_with_delta() {
        let mut k = Kernel::new();
        let sensor = Sensor {
            kind: SensorKind::Temperature,
            reading: 20.0,
            unit: "°C".into(),
            last_reading_ms: 1000,
            deadband: 0.5,
        };
        k.add_room("lab", vec![sensor]);
        let reading = Sensor {
            kind: SensorKind::Temperature,
            reading: 25.0,
            unit: "°C".into(),
            last_reading_ms: 2000,
            deadband: 0.5,
        };
        let ticks = k.process_tick(reading);
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].delta, 5.0);
        assert_eq!(k.metabolism.tiles_processed, 1);
    }

    #[test]
    fn test_predict_events() {
        let mut k = Kernel::new();
        k.add_room("bridge", vec![]);
        // tick_count starts at 0, so we need to bump it
        k.rooms[0].tick_count = 10;
        let preds = k.predict_events();
        assert_eq!(preds.len(), 1);
        assert!(preds[0].event.contains("bridge"));
        assert!(!preds[0].confirmed);
    }

    #[test]
    fn test_confirm() {
        let mut k = Kernel::new();
        k.coordinator.predictions.push(Prediction {
            event: "test-event".into(),
            predicted_at_ms: 1000,
            confidence: 0.9,
            confirmed: false,
        });
        assert!(k.confirm("test-event"));
        assert_eq!(k.coordinator.confirmations, 1);
        assert!(!k.confirm("test-event")); // already confirmed
    }

    #[test]
    fn test_adapt_triggered() {
        let mut k = Kernel::new();
        assert!(k.adapt_if_needed(0.3));
        assert_eq!(k.coordinator.adaptations, 1);
        assert!((k.metabolism.cr_avg - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adapt_not_triggered() {
        let mut k = Kernel::new();
        assert!(!k.adapt_if_needed(0.8));
        assert_eq!(k.coordinator.adaptations, 0);
    }

    #[test]
    fn test_status() {
        let mut k = Kernel::new();
        k.add_room("a", vec![Sensor {
            kind: SensorKind::Pressure,
            reading: 101.3,
            unit: "kPa".into(),
            last_reading_ms: 0,
            deadband: 1.0,
        }]);
        let status = k.status();
        assert_eq!(status.room_count, 1);
        assert_eq!(status.sensor_count, 1);
    }

    #[test]
    fn test_sensor_kinds() {
        let kinds = vec![
            SensorKind::Temperature,
            SensorKind::Pressure,
            SensorKind::Gps,
            SensorKind::Sonar,
            SensorKind::Compass,
            SensorKind::Rpm,
            SensorKind::Fuel,
            SensorKind::Custom("humidity".into()),
        ];
        assert_eq!(kinds.len(), 8);

        // Verify custom serializes
        let json = serde_json::to_string(&SensorKind::Custom("co2".into())).unwrap();
        assert!(json.contains("co2"));
    }

    #[test]
    fn test_transport_variants() {
        let transports = vec![
            Transport::Local,
            Transport::Http,
            Transport::Mqtt,
            Transport::A2a,
            Transport::Git,
        ];
        assert_eq!(transports.len(), 5);
    }
}
