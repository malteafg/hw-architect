use std::time::Instant;

pub struct Timer {
    start_time: Instant,
    last_time: Instant,
}

impl Timer {
    pub fn new() -> Self {
        let start_time = Instant::now();
        Self {
            start_time,
            last_time: start_time,
        }
    }

    pub fn emit(&mut self, m: &str) {
        let curr_time = Instant::now();
        let print_time = curr_time - self.last_time;
        let output = format!("{}: {} ms", m, print_time.as_millis());
        println!("{}", output);
        self.last_time = curr_time;
    }

    pub fn elapsed(&self) {
        let output = format!(
            "Total elapsed time: {} ms",
            self.start_time.elapsed().as_millis()
        );
        println!("{}", output);
    }
}
