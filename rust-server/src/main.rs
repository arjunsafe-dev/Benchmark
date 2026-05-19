use std::{  collections::HashMap, 
            sync::{Arc, RwLock},
            thread,
            time::Duration,
            net::TcpListener,
            io::{Read, Write}
        };

use prometheus::{register, register_counter, Counter, TextEncoder, Encoder, process_collector::ProcessCollector};
use lazy_static::lazy_static;

lazy_static! {
    static ref PACKETS: Counter = register_counter!("rust_packets_total", "total packets").unwrap();
}


#[cfg(target_os = "linux")]
fn main() {

    //initialize packets for prometheus
    lazy_static::initialize(&PACKETS);

    //register standard OS process metrics (CPU, Memory, Threads) 
        {
            if let Err(e) = register(Box::new(ProcessCollector::for_self())) {
                println!("Warning: Could not register process collector: {}", e);
            }
        }

    let state = Arc::new(RwLock::new(HashMap::<u32, Vec<f32>>::new()));

    thread::spawn(|| {
        let listener = TcpListener::bind("0.0.0.0:9898").unwrap();
        println!("Running metrics on http://localhost:9898");
        
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0; 1024];
                
                if let Ok(bytes_read) = stream.read(&mut buf) {
                    let request = String::from_utf8_lossy(&buf[..bytes_read]);
                    if request.contains("/metrics") {
                        let mut buffer = Vec::new();
                        let encoder = TextEncoder::new();
                        encoder.encode(&prometheus::gather(), &mut buffer).unwrap();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n{}",
                            buffer.len(),
                            String::from_utf8_lossy(&buffer)
                        );
                        let _ = stream.write_all(response.as_bytes());
                    } else {
                        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                    }
                }
            }
        }
    });

    // CPU/Threads background work
    let cpu_state = state.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            if let Ok(map) = cpu_state.read() {
                for (_, v1) in map.iter() {
                    for (_, v2) in map.iter() {
                        if let (Some(a), Some(b)) = (v1.first(), v2.first()) {
                            let _ = (a - b).abs();
                        }
                    }
                }
            }
        }
    });

    // TCP server on port 8080
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Running rust TCP server on port 8080");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let state_clone = state.clone();
            thread::spawn(move || {
                let mut conn = stream;
                let mut buf = [0; 8];
                while let Ok(()) = conn.read_exact(&mut buf) {
                    let id = u32::from_le_bytes(buf[0..4].try_into().unwrap());
                    let val = f32::from_le_bytes(buf[4..8].try_into().unwrap());
                    let data = vec![val, val + 1.0, val + 2.0];
                    if let Ok(mut map) = state_clone.write() {
                        map.insert(id, data);
                    }
                    PACKETS.inc();
                }
            });
        }
    }
}
