
struct CPU {
}

impl CPU {

    pub fn new() -> CPU {
        CPU{}
    }

    pub fn powerup(&mut self) {
    }

    fn run(&mut self) {
        loop {
            self.step();
        }
    }

    fn step(&mut self) {
    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
