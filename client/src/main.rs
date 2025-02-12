use std::collections::HashMap;

use serde::Deserialize;
use serde_json::json;

struct Node<'a> {
    root: HashMap<&'a str, Node<'a>>,
}
impl<'a> Node<'a> {
    fn pop(&mut self, s: &str) -> Option<Node> {
        self.root.remove(s)
    }
    fn one(self) -> Option<&'a str> {
        self.root.into_keys().next()
    }
    fn new() -> Self {
        Self {
            root: HashMap::new(),
        }
    }
    fn populate(&mut self, parts: impl Iterator<Item = &'a str>) {
        let mut cur = self;
        for part in parts {
            cur = cur.root.entry(part).or_insert(Self::new());
        }
    }
}

struct Room {
    room_name: String,
    self_name: String,
    server_handle: String,
    self_message_lines: Vec<String>,
    last_message_id: Option<u64>,
    ends_with_newline: bool,
}
fn read_room(file_path: &str) -> Room {
    let room = std::fs::read_to_string(file_path).expect("room file name should be present");
    let room = room.replace("\r\n", "\n");
    let mut config = Node::new();
    let mut lines = room.split('\n');
    loop {
        let Some(mut line) = lines.next() else {
            panic!("Room config should end with a dot line");
        };
        line = line.trim_end();
        if line == "." {
            break;
        }
        if line != "" {
            config.populate(line.split('\\'));
        }
    }
    let mut self_message_lines = Vec::new();
    let mut ends_with_newline = false;
    let mut last_message_id = None;
    for l in lines.rev() {
        if l.is_empty() && self_message_lines.is_empty() {
            ends_with_newline = true;
            continue;
        }
        if let Some(data) = l.strip_prefix('\\') {
            let mut parts = data.split('\\');
            let _username = parts.next().unwrap();
            let _datetime = parts.next().unwrap();
            let _message_id = parts.next().unwrap();
            last_message_id = Some(parts.next().unwrap().to_owned().parse().unwrap());
            break;
        }
        self_message_lines.push(l);
    }
    let self_message_lines = self_message_lines
        .into_iter()
        .rev()
        .map(|s| s.to_owned())
        .collect();
    Room {
        self_message_lines,
        ends_with_newline,
        last_message_id,
        room_name: config
            .pop("room name")
            .and_then(|n| n.one())
            .expect("room name should be specified")
            .to_owned(),
        self_name: config
            .pop("self name")
            .and_then(|n| n.one())
            .expect("self name should be specified")
            .to_owned(),
        server_handle: config
            .pop("server handle")
            .and_then(|n| n.one())
            .expect("server handle should be specified")
            .to_owned(),
    }
}

#[derive(Deserialize)]
struct NewMessage {
    lines: Vec<String>,
    id: u64,
    utc_unix_timestamp: i64,
    sender_name: String,
}

#[derive(Deserialize)]
struct MessageSuccess {
    utc_unix_timestamp: i64,
    id: u64,
}

#[derive(Deserialize)]
struct ServerResponse {
    self_message_success: Option<MessageSuccess>,
    new_messages: Vec<NewMessage>,
}

fn write_msg<'a>(
    f: &mut std::fs::File,
    lines: impl Iterator<Item = &'a String>,
    msg_id: u64,
    utc_unix_timestamp: i64,
    sender_name: &str,
    last_msg_id: u64,
) {
    use std::io::Write;
    for line in lines {
        writeln!(f, "{}", line).unwrap();
    }
    let utc = chrono::DateTime::<chrono::Utc>::from_timestamp(utc_unix_timestamp, 0).unwrap();
    let local = utc.with_timezone(&chrono::Local);
    writeln!(
        f,
        "\\{}\\{}\\{}\\{}",
        sender_name,
        local.to_rfc2822(),
        msg_id,
        last_msg_id
    )
    .unwrap();
}

fn main() {
    let room_file_path = &std::env::args()
        .nth(1)
        .expect("room file name should be provided");
    let room = read_room(room_file_path);
    let mut room_file = std::fs::OpenOptions::new()
        .append(true)
        .open(room_file_path)
        .unwrap();
    let resp = reqwest::blocking::Client::new()
        .post(room.server_handle)
        .json(&json!({
            "room_name": room.room_name,
            "last_message_id": room.last_message_id,
            "self_message_lines": room.self_message_lines,
            "self_name": room.self_name,
        }))
        .send()
        .expect("connection to the server should work");
    let resp = resp.text().unwrap();
    let resp: ServerResponse = serde_json::from_str(&resp)
        .unwrap_or_else(|_| panic!("Unexpected server response: {}", resp));
    if resp.new_messages.is_empty() && resp.self_message_success.is_none() {
        return;
    }
    if !room.ends_with_newline {
        use std::io::Write;
        write!(room_file, "\n").unwrap();
    }
    let self_message_id = if let Some(self_message_success) = resp.self_message_success {
        println!("Sent your own message");
        write_msg(
            &mut room_file,
            std::iter::empty(),
            self_message_success.id,
            self_message_success.utc_unix_timestamp,
            &room.self_name,
            self_message_success.id,
        );
        Some(self_message_success.id)
    } else {
        None
    };
    println!("Received {} new messages", resp.new_messages.len());
    for new_message in resp.new_messages {
        write_msg(
            &mut room_file,
            new_message.lines.iter(),
            new_message.id,
            new_message.utc_unix_timestamp,
            &new_message.sender_name,
            self_message_id.unwrap_or(new_message.id),
        );
    }
}
