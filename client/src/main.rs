use std::env;
use vecmath::{self, Vector3};

fn main() {
    let args: Vec<String> = env::args().collect();
    let client_id: String = args[1].clone();
    let start_pos: Vector3<i32> = [
        args[2].clone().parse::<i32>().unwrap(),
        args[3].clone().parse::<i32>().unwrap(),
        args[4].clone().parse::<i32>().unwrap(),
    ];
    let end_pos: Vector3<i32> = [
        args[5].clone().parse::<i32>().unwrap(),
        args[6].clone().parse::<i32>().unwrap(),
        args[7].clone().parse::<i32>().unwrap(),
    ];
    let plane_speed = args[8].clone().parse::<i32>().unwrap();

    //enter flight loop
    loop {
        //move aircraft

        //if distance to destination is less than A VALUE (idk what)

        break;

        //send data

        //wait for 5 seconds
    }

    //send big data
}
