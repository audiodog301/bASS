pub fn rms(samples: Vec<f32>) -> f32 {
    let mut square_sum = 0.0;
    for sample in &samples {
        square_sum += sample.powi(2);
    }
    let mean_square = square_sum / samples.len() as f32;

    mean_square.powf(0.5)
}

enum GateState {
    Trigger,
    Passthrough,
    Stop,
}

pub struct ClipGate {
    mult: f32,
    gate_rms: f32,
    gate_mult: f32,
    target: f32,
    gate_state: GateState,
}

impl ClipGate {
    pub fn default() -> Self {
        Self {
            mult: 1.0,
            gate_rms: 0.0,
            gate_mult: 1.0,
            target: 0.0,
            gate_state: GateState::Stop,
        }
    }

    pub fn set_mult(&mut self, mult: f32) {
        self.mult = mult;
    }

    pub fn set_gate(&mut self, gate: f32) {
        self.gate_rms = gate;
    }

    pub fn process_sample(&mut self, mut input: f32, rms: f32) -> f32 {
        //basic hard-clip distortion
        input *= self.mult;
        if input > 1.0 {
            input = 1.0;
        }
        if input < -1.0 {
            input = -1.0;
        }
    
        //REALLY HACKY GATE IMPLEMENTATION
        if rms < self.gate_rms {
            self.target = 0.0
        }
        if rms > self.gate_rms {
            self.target = 1.0
        }

        self.gate_mult += (self.target - self.gate_mult) / 2.0;

        input * self.gate_mult
    }
}