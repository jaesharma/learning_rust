use std::{env, thread, process};
use std::str::FromStr;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::mpsc::{Sender, channel};

const MAX: u16 = 65535;

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }
        if(MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        match args.len() {
            len if len < 2 => Err("Not enough arguments"),
            len if len > 4 => Err("Too many arguments"),
            2 => match IpAddr::from_str(&args[1]) {
                Ok(ipaddr) => Ok(Arguments { flag: String::from(""), ipaddr, threads: 4 }),
                Err(_) => {
                    let flag = args[1].clone();
                    if flag.contains("-h") || flag.contains("--help") {
                        println!("Usage: \n-j to select how many threads\n-h or --help to show this message.");
                        Err("help")
                    } else {
                        Err("Invalid Syntax")
                    }
                }
            },
            _ => {
                let flag = args[1].clone();
                if !(flag.contains("-h") || flag.contains("--help")) {
                    let ipaddr = match IpAddr::from_str(&args[3]) {
                        Ok(s) => s,
                        Err(_) => return Err("not a valid IP Address; must be IPv4 or IPv6")
                    };
                    let threads = match args[2].parse::<u16>() {
                        Ok(s) => s,
                        Err(_) => return Err("failed to parse thread number")
                    };
                    Ok(Arguments{threads, flag, ipaddr})
                } else {
                    Err("too many arguments")
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(
        |err| {
            if err.contains("help") {
                process::exit(0);
            } else {
                eprint!("{} problem parsing arguments: {}", program, err);
                process::exit(0);
            }
        }
    );

    let num_threads = arguments.threads;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        let tx = tx.clone();

        thread::spawn(move || {
            scan(tx, i, arguments.ipaddr, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}
