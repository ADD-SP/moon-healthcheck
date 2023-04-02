use std::time;

#[derive(PartialEq, Debug)]
enum Status {
    Init,
    Healthy,
    Unhealthy,
}

#[derive(PartialEq, Debug)]
pub enum Protocol {
    Tcp,
    Udp,
    Http,
}

#[derive(Debug)]
pub struct State {
    protocol: Protocol,
    status: Status,
    last_check: Option<time::Instant>,
    last_error: Option<String>,
    consecutive_failures: usize,
    max_consecutive_failures: usize,
}

impl State {
    pub fn new(protocol: Protocol, max_consecutive_failures: usize) -> State {
        State {
            protocol,
            status: Status::Init,
            last_check: None,
            last_error: None,
            consecutive_failures: 0,
            max_consecutive_failures,
        }
    }

    pub fn get_protocol(&self) -> &Protocol {
        &self.protocol
    }

    pub fn get_last_check(&self) -> Option<time::Instant> {
        self.last_check
    }

    pub fn get_last_error(&self) -> Option<String> {
        self.last_error.clone()
    }

    pub fn is_healthy(&self) -> bool {
        self.status == Status::Healthy || self.status == Status::Init
    }

    pub fn report_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_check = Some(time::Instant::now());
        self.update();
    }

    pub fn report_failure(&mut self, err: String)
    {
        // usize is big enough to hold the failures
        self.consecutive_failures += 1;
        self.last_check = Some(time::Instant::now());
        self.last_error = Some(err.clone());
        self.update();
    }

    fn update(&mut self) {
        if self.consecutive_failures > self.max_consecutive_failures {
            self.status = Status::Unhealthy;

        } else {
            self.status = Status::Healthy;
        }
    }

    pub fn set_health(&mut self, health: bool) {
        match health {
            true => self.consecutive_failures = 0,
            false => self.consecutive_failures = self.max_consecutive_failures + 1,
        }
        self.update();
    }

}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let state = State::new(Protocol::Tcp, 3);
        assert_eq!(state.get_protocol(), &Protocol::Tcp);
        assert_eq!(state.is_healthy(), true);
        assert_eq!(state.get_last_check(), None);
        assert_eq!(state.get_last_error(), None);
    }

    #[test]
    fn report_success() {
        let mut state = State::new(Protocol::Tcp, 3);
        state.set_health(false);
        assert_eq!(state.is_healthy(), false);
        state.report_success();
        assert_eq!(state.is_healthy(), true);
    }

    #[test]
    fn report_failure() {
        let mut state = State::new(Protocol::Tcp, 1);
        state.report_failure(String::from("test"));
        state.report_failure(String::from("test"));
        assert_eq!(state.is_healthy(), false);
        assert_eq!(state.get_last_error(), Some("test".to_string()));
    }

    #[test]
    fn get_protocol() {
        let state = State::new(Protocol::Tcp, 3);
        assert_eq!(state.get_protocol(), &Protocol::Tcp);
    }

    #[test]
    fn set_health() {
        let mut state = State::new(Protocol::Tcp, 3);
        state.set_health(false);
        assert_eq!(state.is_healthy(), false);
        state.set_health(true);
        assert_eq!(state.is_healthy(), true);
    }
}
