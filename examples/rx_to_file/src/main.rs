use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, Instant},
};

use bytemuck::cast_slice;
use clap::Parser;
use num_complex::Complex32;

use uhd_usrp::{timespec, RxMetadata, Usrp};

type Sample = Complex32;

#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    file: PathBuf,
    #[arg(long)]
    freq: f64,
    #[arg(long)]
    rate: f64,
    #[arg(long)]
    bw: f64,
    #[arg(long)]
    gain: f64,
    #[arg(long)]
    ant: String,
    #[arg(long)]
    args: String,
    #[arg(long)]
    channel: usize,
    #[arg(long)]
    duration: u64,
}

fn write_to_file(recv: Receiver<Vec<Sample>>, file: PathBuf) -> Result<(), Box<dyn Error>> {
    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)?;
    let mut f = BufWriter::with_capacity(1_000_000, f);
    while let Ok(data) = recv.recv() {
        f.write_all(cast_slice(&data[..]))?;
    }
    Ok(())
}

fn run_recv(usrp: Usrp, send: Sender<Vec<Sample>>, dur: Duration) -> Result<(), Box<dyn Error>> {
    let mut rx_stream = usrp.rx_stream::<Sample>().with_channels(&[0]).open()?;
    let mut buff = vec![Complex32::new(0.0, 0.0); rx_stream.max_samples_per_channel()];
    let mut md = RxMetadata::new();

    rx_stream
        .start_command()
        .with_time(timespec!(500 ms))
        .send()?;

    let start = Instant::now();
    while start.elapsed() < dur {
        let samples = rx_stream
            .reader()
            .with_metadata_output(&mut md)
            .with_timeout(Duration::from_millis(100))
            .recv(&mut buff)?;

        if let Err(_) = send.send(buff[..samples].to_vec()) {
            eprintln!("channel dropped before time elapsed");
            break;
        }
    }

    rx_stream.stop_now()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut usrp = Usrp::open_with_args(&args.args)?;
    usrp.set_rx_config(args.channel)
        .set_antenna(&args.ant)?
        .set_center_freq(args.freq)?
        .set_bandwidth(args.bw)?
        .set_gain(None, args.gain)?
        .set_sample_rate(args.rate)?;

    let config = usrp.rx_config(args.channel);
    println!("Antenna: {}", config.antenna()?);
    println!("Freq: {}", config.center_freq()?);
    println!("Bandwidth: {}", config.bandwidth()?);
    println!("Gain: {}", config.gain(None)?);
    println!("Rate: {}", config.sample_rate()?);

    let (send, recv): (Sender<Vec<Complex32>>, Receiver<Vec<Complex32>>) = channel();
    let thr = std::thread::spawn(move || write_to_file(recv, args.file).unwrap());

    run_recv(usrp, send, Duration::from_secs(args.duration))?;
    thr.join().unwrap();
    Ok(())
}
