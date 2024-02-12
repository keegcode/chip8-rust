pub trait Emulator {
    fn load_fonts(&mut self, fonts: &[u8]) -> Result<&mut Self, String>;
    fn load_rom(&mut self, path: &str) -> Result<&mut Self, String>;
    fn run(&mut self) -> Result<(), String>;
}