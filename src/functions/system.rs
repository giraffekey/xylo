use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use rand_chacha::ChaCha8Rng;

builtin_function!(width data => {
    [] => |data: &Data| Ok(Value::Integer(data.dimensions.0 as i32)),
});

builtin_function!(height data => {
    [] => |data: &Data| Ok(Value::Integer(data.dimensions.1 as i32)),
});

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_width_height_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);

        // Test with standard dimensions
        let data = Data {
            dimensions: (800, 600),
            ..Default::default()
        };

        // Test width
        assert_eq!(width(&mut rng, &data, &[]).ok(), Some(Value::Integer(800)));

        // Test height
        assert_eq!(height(&mut rng, &data, &[]).ok(), Some(Value::Integer(600)));

        // Test with minimum dimensions
        let min_data = Data {
            dimensions: (1, 1),
            ..Default::default()
        };

        assert_eq!(
            width(&mut rng, &min_data, &[]).ok(),
            Some(Value::Integer(1))
        );
        assert_eq!(
            height(&mut rng, &min_data, &[]).ok(),
            Some(Value::Integer(1))
        );

        // Test with maximum i32 dimensions
        let max_data = Data {
            dimensions: (i32::MAX as u32, i32::MAX as u32),
            ..Default::default()
        };

        assert_eq!(
            width(&mut rng, &max_data, &[]).ok(),
            Some(Value::Integer(i32::MAX))
        );
        assert_eq!(
            height(&mut rng, &max_data, &[]).ok(),
            Some(Value::Integer(i32::MAX))
        );
    }
}
