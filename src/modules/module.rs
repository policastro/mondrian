pub trait Module {
    fn start(&mut self);
    fn stop(&mut self);
    fn restart(&mut self);
    fn enable(&mut self, enabled: bool);
}
