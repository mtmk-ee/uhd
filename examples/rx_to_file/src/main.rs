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

use uhd_usrp::{ArrayBuffer, RxMetadata, StreamArgs, Usrp};

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
    let mut rx_stream = usrp.rx_stream(StreamArgs::<Sample>::new().channels(&[0]))?;
    println!("max: {}", rx_stream.max_samples_per_buffer());
    let mut buff = ArrayBuffer::<Sample>::new(1, rx_stream.max_samples_per_buffer());

    let mut reader = rx_stream
        .reader()
        .with_timeout(Duration::from_secs_f32(0.1))
        .open()?;
    let mut md = RxMetadata::new()?;

    let start = Instant::now();
    while start.elapsed() < dur {
        // let samples = reader.recv(&mut buff, &mut md)?;
        // if let Err(_) = send.send(buff[0][..samples].to_vec())
        // {
        //     eprintln!("channel dropped before time elapsed");
        //     break;
        // }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut usrp = Usrp::open_with_str(&args.args)?;
    usrp.set_rx_config(args.channel)
        .set_antenna(&args.ant)?
        .set_center_freq(args.freq)?
        .set_bandwidth(args.bw)?
        .set_gain(None, args.gain)?
        .set_sample_rate(args.rate)?;

    let (send, recv): (Sender<Vec<Complex32>>, Receiver<Vec<Complex32>>) = channel();
    let thr = std::thread::spawn(move || write_to_file(recv, args.file).unwrap());

    run_recv(usrp, send, Duration::from_secs(args.duration))?;
    thr.join().unwrap();
    Ok(())
}
