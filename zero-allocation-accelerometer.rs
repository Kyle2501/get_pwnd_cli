//! Zero-allocation accelerometer telemetry parser and motion classifier.
//! Operates entirely on stack buffers and byte slices without heap allocation.

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Computes the Euclidean norm: ||v|| = sqrt(x^2 + y^2 + z^2)
    #[inline(always)]
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Computes magnitude squared to avoid expensive square root operations when thresholding.
    #[inline(always)]
    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Absolute delta between two vector states.
    #[inline(always)]
    pub fn delta(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: (self.x - other.x).abs(),
            y: (self.y - other.y).abs(),
            z: (self.z - other.z).abs(),
        }
    }
}

// --- Motion Classification Events ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionEvent {
    None,
    Freefall { g_force: f32 },
    SignificantMotion { delta_mag: f32 },
    HardImpact { g_force: f32 },
}

// --- Motion Threshold Configuration ---

pub struct MotionThresholds {
    pub freefall_limit_sq: f32,   // < ~2.0 m/s^2 (~0.2G)
    pub impact_limit_sq: f32,     // > ~25.0 m/s^2 (~2.5G)
    pub motion_delta_sq: f32,     // Change across consecutive frames
}

impl Default for MotionThresholds {
    fn default() -> Self {
        Self {
            freefall_limit_sq: 4.0,   // (2.0)^2
            impact_limit_sq: 625.0,   // (25.0)^2
            motion_delta_sq: 16.0,    // (4.0)^2
        }
    }
}

// --- Zero-Heap Byte Parser ---

pub struct AccelerometerParser;

impl AccelerometerParser {
    /// In-place byte scanner that locates `"values": [` and parses 3 float values.
    /// Returns `None` if the buffer is incomplete or malformed.
    pub fn parse_values(buffer: &[u8]) -> Option<Vector3> {
        let needle = b"\"values\":";
        let pos = buffer.windows(needle.len()).position(|w| w == needle)?;
        let remainder = &buffer[pos + needle.len()..];

        // Seek opening bracket '['
        let start_bracket = remainder.iter().position(|&b| b == b'[')?;
        let array_slice = &remainder[start_bracket + 1..];

        // Seek closing bracket ']'
        let end_bracket = array_slice.iter().position(|&b| b == b']')?;
        let values_slice = &array_slice[..end_bracket];

        // Extract 3 comma-delimited numeric segments
        let mut parts = [0f32; 3];
        let mut part_idx = 0;
        let mut token_start = 0;

        for i in 0..=values_slice.len() {
            let is_delimiter = i == values_slice.len() || values_slice[i] == b',';
            if is_delimiter {
                if part_idx >= 3 {
                    break;
                }
                let token = &values_slice[token_start..i];
                parts[part_idx] = Self::fast_parse_f32(token)?;
                part_idx += 1;
                token_start = i + 1;
            }
        }

        if part_idx == 3 {
            Some(Vector3::new(parts[0], parts[1], parts[2]))
        } else {
            None
        }
    }

    /// Zero-allocation ASCII to f32 parser.
    #[inline]
    fn fast_parse_f32(slice: &[u8]) -> Option<f32> {
        // Strip ASCII whitespace
        let mut start = 0;
        while start < slice.len() && (slice[start] == b' ' || slice[start] == b'\n' || slice[start] == b'\r' || slice[start] == b'\t') {
            start += 1;
        }
        let mut end = slice.len();
        while end > start && (slice[end - 1] == b' ' || slice[end - 1] == b'\n' || slice[end - 1] == b'\r' || slice[end - 1] == b'\t') {
            end -= 1;
        }

        if start >= end {
            return None;
        }

        let s = core::str::from_utf8(&slice[start..end]).ok()?;
        s.parse::<f32>().ok()
    }
}

// --- Stateful Motion Detector ---

pub struct MotionEngine {
    thresholds: MotionThresholds,
    last_reading: Option<Vector3>,
}

impl MotionEngine {
    pub fn new(thresholds: MotionThresholds) -> Self {
        Self {
            thresholds,
            last_reading: None,
        }
    }

    /// Evaluates reading against thresholds in $O(1)$ time without sqrt operations.
    pub fn process_sample(&mut self, current: Vector3) -> MotionEvent {
        let mag_sq = current.magnitude_squared();

        // 1. Check Freefall (near 0G: gravity dropped out)
        if mag_sq < self.thresholds.freefall_limit_sq {
            self.last_reading = Some(current);
            return MotionEvent::Freefall {
                g_force: mag_sq.sqrt() / 9.80665,
            };
        }

        // 2. Check Impact (spikes exceeding threshold)
        if mag_sq > self.thresholds.impact_limit_sq {
            self.last_reading = Some(current);
            return MotionEvent::HardImpact {
                g_force: mag_sq.sqrt() / 9.80665,
            };
        }

        // 3. Check Differential Motion against previous frame
        let mut event = MotionEvent::None;
        if let Some(prev) = self.last_reading {
            let delta = current.delta(&prev);
            let delta_sq = delta.magnitude_squared();

            if delta_sq > self.thresholds.motion_delta_sq {
                event = MotionEvent::SignificantMotion {
                    delta_mag: delta_sq.sqrt(),
                };
            }
        }

        self.last_reading = Some(current);
        event
    }
}
