use core::fmt;

#[derive(Debug, Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    ///Create a new Vector3
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    ///Calculate the distance between two vectors
    pub fn distance(a: Vector3, b: Vector3) -> f32 {
        ((b.x - a.x).powi(2) + (b.y - a.y).powi(2) + (b.z - a.z).powi(2)).sqrt()
    }

    //Add a vector to the existing vector
    pub fn add(&self, a: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + a.x,
            y: self.y + a.y,
            z: self.z + a.z,
        }
    }

    ///Calculate the displacement vector towards a target location and with given speed
    pub fn displacement_vector(&self, target: Vector3, speed: f32) -> Vector3 {
        let dx = target.x - self.x;
        let dy = target.y - self.y;
        let dz = target.z - self.z;

        let magnitude = (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt();

        let norm_dx = dx / magnitude;
        let norm_dy = dy / magnitude;
        let norm_dz = dz / magnitude;

        let vel_x = norm_dx * speed;
        let vel_y = norm_dy * speed;
        let vel_z = norm_dz * speed;

        Vector3::new(vel_x, vel_y, vel_z)
    }

    ///Convert Vector3 to a vector of u8.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.append(&mut self.x.to_be_bytes().to_vec());
        bytes.append(&mut self.y.to_be_bytes().to_vec());
        bytes.append(&mut self.z.to_be_bytes().to_vec());
        bytes
    }

    ///Create Vector3 from a slice of u8.
    pub fn from_bytes(bytes: &[u8]) -> Option<Vector3> {
        if bytes.len() < 12 {
            return None;
        }
        let x_bytes: [u8; 4] = bytes[0..4].try_into().ok()?;
        let y_bytes = bytes[4..8].try_into().ok()?;
        let z_bytes = bytes[8..12].try_into().ok()?;
        let x = f32::from_be_bytes(x_bytes);
        let y = f32::from_be_bytes(y_bytes);
        let z = f32::from_be_bytes(z_bytes);
        Some(Vector3::new(x, y, z))
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{},{}]", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = Vector3::new(1.0, 2.0, 3.0);

        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_distance() {
        let v1 = Vector3::new(2.0, 2.0, 3.0);
        let v2 = Vector3::new(2.0, 4.0, 0.0);

        let out = Vector3::distance(v1, v2);

        assert_eq!(out, 3.6055512);
    }

    #[test]
    fn test_distance_negative() {
        let v1 = Vector3::new(2.0, 2.0, 3.0);
        let v2 = Vector3::new(-2.0, -4.0, 0.0);

        let result = Vector3::distance(v1, v2);

        assert_eq!(result, 7.81025);
    }

    #[test]
    fn test_add() {
        let v1 = Vector3::new(1.0, 1.0, 1.0);
        let v2 = Vector3::new(1.0, 1.0, 1.0);

        let out = v1.add(v2);

        assert_eq!(out.x, 2.0);
        assert_eq!(out.y, 2.0);
        assert_eq!(out.z, 2.0);
    }
    #[test]
    fn test_add_negative() {
        let v1 = Vector3::new(1.0, 1.0, 1.0);
        let v2 = Vector3::new(-5.0, -5.0, -5.0);

        let out = v1.add(v2);

        assert_eq!(out.x, -4.0);
        assert_eq!(out.y, -4.0);
        assert_eq!(out.z, -4.0);
    }

    #[test]
    fn test_displacement_vector() {
        let v1 = Vector3::new(3.0, 4.0, 0.0);
        let v2 = Vector3::new(7.0, 8.0, 10.0);
        let speed: f32 = 5.0;

        let out = v1.displacement_vector(v2, speed);

        assert_eq!(out.x, 1.7407765);
        assert_eq!(out.y, 1.7407765);
        assert_eq!(out.z, 4.351941);
    }
}
