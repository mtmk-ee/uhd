use std::error::Error;

use clap::Parser;

use uhd_usrp::Usrp;

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
    let sensors = match dir {
        Direction::Rx => usrp.mboard(mb_idx).rx_subdev_spec().unwrap(),
        Direction::Tx => usrp.mboard(mb_idx).tx_subdev_spec().unwrap(),
    };
    let n_chans = sensors.len();

    for chan_idx in 0..n_chans {
        match dir {
            Direction::Rx => println!("* Rx Channel {chan_idx}:"),
            Direction::Tx => println!("* Tx Channel {chan_idx}:"),
        }
        let sensors = match dir {
            Direction::Rx => usrp.rx_config(chan_idx).sensor_names().unwrap(),
            Direction::Tx => usrp.tx_config(chan_idx).sensor_names().unwrap(),
        };
        for sensor in sensors {
            let sensor_value = match dir {
                Direction::Rx => usrp.rx_config(chan_idx).sensor_value(&sensor).unwrap(),
                Direction::Tx => usrp.tx_config(chan_idx).sensor_value(&sensor).unwrap(),
            };
            println!("\t* {sensor_value}");
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
