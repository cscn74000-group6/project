use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let client_id: String = args[1].clone();
    let start_pos = args[2].clone();
    let end_pos = args[3].clone();
    let plane_speed = args[4].clone().parse::<i32>().unwrap();


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
