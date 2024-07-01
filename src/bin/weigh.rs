use std::{thread::sleep, time::Duration};

use uom::si::{
    f64::Force,
    force::{kilogram_force, pound_force},
};

use weighty;

fn pretty_print_force(f: Force) {
    let kg = f.get::<kilogram_force>();
    let lb = f.get::<pound_force>();

    println!("{}kg ({}lb)", kg, lb);
}

fn main() {
    let all_scales = weighty::get_all_scales();

    loop {
        if all_scales.len() > 0 {
            for scale in all_scales {
                pretty_print_force(scale.read().unwrap());
            }
        } else {
            println!(
                "No scales found.  Are you sure it's plugged in, on, and something you have access to?"
            );
        }
        sleep(Duration::from_millis(1000.0));
    }
}
