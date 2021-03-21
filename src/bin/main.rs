use std::{fs, net::{TcpListener, TcpStream}, thread, time::Duration};
use std::io::prelude::*;
use regex::Regex;
use web_server::ThreadPool;
fn main() {
    let html_stub = fs::read_to_string("./src/html_stubs/index.html").unwrap();
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(10) {
        let stream = stream.unwrap();
        let html = String::from(&html_stub[..]);
        pool.execute(|| {
            handle_conection(stream, html);
        });
    }
}

fn handle_conection(mut stream: TcpStream, html_stub: String) {
    let mut buffer = [0; 512];
    let mut html = String::from(html_stub);
    let re_content = Regex::new(r"#CONTENT#").unwrap();
    let re_title = Regex::new(r"#TITLE#").unwrap();
    let re_status = Regex::new(r"#STATUS#").unwrap();
    stream.read(&mut buffer).unwrap();
    let mut status = String::from("HTTP/1.1 #STATUS#");
    status = format!("{}", re_status.replace_all(&mut status, "200 OK").trim());

    if buffer.starts_with(b"GET / HTTP/1.1") {
        html = format!("{}", re_content.replace_all(&mut html, "200 OK RESPONSE").trim());
        html = format!("{}", re_title.replace_all(&mut html, "HELLO FROM RUST").trim());
    } else if buffer.starts_with(b"GET /sleep HTTP/1.1") {
        thread::sleep(Duration::from_secs(5));
        html = format!("{}", re_content.replace_all(&mut html, "Sleep 5 seconds...").trim());
        html = format!("{}", re_title.replace_all(&mut html, "Sleep...").trim());
    } else {
        html = format!("{}", re_content.replace_all(&mut html, "404 | PAGE NOT FOUND").trim());
        html = format!("{}", re_title.replace_all(&mut html, "404 | PAGE NOT FOUND").trim());
        status = format!("{}", re_status.replace_all(&mut status, "404 NOT FOUND").trim());
    }

    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status, html.len(), html);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
