pub struct Memory{
    pub memory : [u8; 65535] // * implements working memory
}

impl Memory {
    pub fn new() -> Memory {
        Memory { memory : [0x0; 65535]}
    }
}