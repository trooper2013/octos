#[cfg(target_os = "linux")]
use smithay::utils::{Point, Size, Rectangle, Logical};

#[cfg(not(target_os = "linux"))]
pub mod mock_smithay {
    #[derive(Debug, Clone)]
    pub struct Logical;

    #[derive(Debug, Clone)]
    pub struct Point<T> {
        pub x: T,
        pub y: T,
    }

    impl<T> From<(T, T)> for Point<T> {
        fn from(val: (T, T)) -> Self {
            Self { x: val.0, y: val.1 }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Size<T> {
        pub w: T,
        pub h: T,
    }

    impl<T> From<(T, T)> for Size<T> {
        fn from(val: (T, T)) -> Self {
            Self { w: val.0, h: val.1 }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Rectangle<T, U> {
        pub loc: Point<T>,
        pub size: Size<T>,
        _phantom: std::marker::PhantomData<U>,
    }

    impl<T, U> Rectangle<T, U> {
        pub fn from_loc_and_size(loc: Point<T>, size: Size<T>) -> Self {
            Self {
                loc,
                size,
                _phantom: std::marker::PhantomData,
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
use mock_smithay::{Point, Size, Rectangle, Logical};

pub struct OctosDisplayServer {
    pub outputs: Vec<Rectangle<i32, Logical>>,
    pub surfaces: Vec<String>,
    pub frame_buffers: Vec<Vec<u8>>,
}

impl OctosDisplayServer {
    pub fn new() -> Self {
        // Initialize screen outputs with 1920x1080 geometry using Smithay's Logical scale
        let default_output = Rectangle::from_loc_and_size(
            Point::from((0, 0)),
            Size::from((1920, 1080)),
        );
        Self {
            outputs: vec![default_output],
            surfaces: Vec::new(),
            frame_buffers: Vec::new(),
        }
    }

    /// Renders an agent card layout to RGBA buffer canvas mapping states.
    pub fn render_agent_card(&mut self, intent: &str, payload_json: &str) -> Vec<u8> {
        println!(
            "[SYSTEM LOG] [COMPOSITOR] render_agent_card: intent='{}', payload='{}'",
            intent, payload_json
        );

        let width = 800;
        let height = 600;
        let mut buffer = vec![0u8; width * height * 4];

        // Draw basic card backdrop
        for idx in 0..(width * height) {
            let offset = idx * 4;
            // Draw slate grey: RGBA (44, 44, 46, 255)
            buffer[offset] = 44;
            buffer[offset + 1] = 44;
            buffer[offset + 2] = 46;
            buffer[offset + 3] = 255;
        }

        // Track rendering state
        self.surfaces.push(format!("card:{}", intent));
        self.frame_buffers.push(buffer.clone());

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_agent_card_state_mapping() {
        let mut server = OctosDisplayServer::new();
        assert_eq!(server.outputs.len(), 1);
        assert_eq!(server.outputs[0].size.w, 1920);
        assert_eq!(server.outputs[0].size.h, 1080);

        let intent = "approve_payment";
        let payload = r#"{"amount": 5000}"#;
        let buffer = server.render_agent_card(intent, payload);

        // Verify buffer is generated, non-empty, and has correct size (800 * 600 * 4 = 1,920,000 bytes)
        assert_eq!(buffer.len(), 800 * 600 * 4);
        assert_eq!(buffer[3], 255); // Opaque alpha validation
        
        // Verify tracking state
        assert_eq!(server.surfaces.len(), 1);
        assert_eq!(server.surfaces[0], "card:approve_payment");
        assert_eq!(server.frame_buffers.len(), 1);
    }
}
