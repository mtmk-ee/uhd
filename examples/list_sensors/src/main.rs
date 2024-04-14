use std::error::Error;

use clap::Parser;

use uhd_usrp::{Channel, Usrp};

#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    args: String,
}

enum Direction {
    Rx,
    Tx,
}

fn print_db_sensors(usrp: &Usrp, mb_idx: usize, dir: Direction) {
    let spec = match dir {
        Direction::Rx => usrp.mboard(mb_idx).rx_subdev_spec().unwrap(),
        Direction::Tx => usrp.mboard(mb_idx).tx_subdev_spec().unwrap(),
    };
    for chan_idx in 0..spec.len() {
        let channel = match dir {
            Direction::Rx => Channel::Rx(chan_idx),
            Direction::Tx => Channel::Tx(chan_idx),
        };
        println!("* {channel}:");
        let channel = usrp.channel(channel).unwrap();
        for sensor in channel.iter_sensor_values().unwrap() {
            println!("\t* {sensor}");
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let usrp = Usrp::open_with_args(&args.args)?;

    let n_mboards = usrp.n_mboards()?;
    println!("Device contains {n_mboards} motherboard(s).");
    for i in 0..n_mboards {
        println!("_____________________________________________");
        println!("Motherboard {i}:");

        let mboard_sensors = usrp.mboard(i).sensor_names().unwrap();
        for sensor in mboard_sensors {
            let value = usrp.mboard(i).sensor_value(&sensor).unwrap();
            println!("* {value}");
        }

        print_db_sensors(&usrp, i, Direction::Rx);
        print_db_sensors(&usrp, i, Direction::Tx);
    }

    Ok(())
}
