use std::env;
use std::{thread, time};
use utils::vector::Vector3;

fn main() {
    let args: Vec<String> = env::args().collect();
    let client_id: String = args[1].clone();
    let start_pos: Vector3 = Vector3::new(
        args[2].clone().parse::<f32>().unwrap(),
        args[3].clone().parse::<f32>().unwrap(),
        args[4].clone().parse::<f32>().unwrap(),
    );
    let end_pos = Vector3::new(
        args[5].clone().parse::<f32>().unwrap(),
        args[6].clone().parse::<f32>().unwrap(),
        args[7].clone().parse::<f32>().unwrap(),
    );
    let plane_speed = args[8].clone().parse::<f32>().unwrap();

    let mut plane_pos = start_pos;

    //enter flight loop
    loop {
        //move aircraft
        plane_pos = plane_pos.add(plane_pos.displacement_vector(end_pos, plane_speed));
        println!("{client_id} moved to {plane_pos}");

        //if distance to destination is less than A VALUE (idk what)
        if Vector3::distance(plane_pos, end_pos) < 10.0 {
            break;
        }

        //send data
        println!("sent!");

        //wait for 5 seconds
        let ten_millis = time::Duration::from_secs(5);
        thread::sleep(ten_millis);
    }

    //send big data
}
