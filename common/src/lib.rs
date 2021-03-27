use std::mem::size_of;

pub const PORT: u32 = 5321;
pub const DEVEL_IP: &str = "localhost:5321";
pub const PROD_IP: &str = "143.198.74.108:5321";

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PongInputState {
    // Warning: For network security, there should be no byte padding inserted. This would cause
    // unitialized memory to be transmitted over the network: BAD IDEA - Ben Aubin
    pub frame: u32,
    pub input: f32,
}

impl PongInputState {
    pub fn new() -> Self {
        PongInputState {
            frame: 0,
            input: 0.0,
        }
    }

    // for unit tests
    pub fn from_input(input: f32) -> Self {
        PongInputState {
            frame: 0,
            input: input,
        }
    }

    // To ensure byte alignment, you should probably
    // call this like PongInputState::new().into_u8()
    pub fn into_u8(self) -> [u8; size_of::<Self>()] {
        unsafe { std::mem::transmute(self) }
    }

    // caution: this method may only be called on u8 slices that are slices of memory that is PongInputState
    // or at least PongInputState byte aligned.
    pub unsafe fn from_u8(b: [u8; size_of::<Self>()]) -> Self {
        std::mem::transmute(b)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
