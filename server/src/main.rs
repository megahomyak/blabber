use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
struct Request {
    room_name: String,
    last_message_id: Option<u64>,
    self_message_lines: Vec<String>,
    self_name: String,
}

fn main() {
    let ass = assystem::ASS::open(
        std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open("messages.ass")
            .unwrap(),
    )
    .unwrap();
    let ass = std::sync::Mutex::new(ass);
    let server = rouille::Server::new("0.0.0.0:80", move |request| {
        ass.clear_poison();
        let mut ass = ass.lock().unwrap();
        let mut data = String::new();
        request.data().unwrap().read_to_string(&mut data).unwrap();
        let request: Request = serde_json::from_str(&data).unwrap();

        let mut new_messages = Vec::new();
        let mut current_message_id: u64 = request.last_message_id.map(|id| id + 1).unwrap_or(0);
        while let Some(new_message) =
            ass.get(format!("{}/messages/{}", request.room_name, current_message_id).as_bytes())
        {
            let new_message: serde_json::Value =
                serde_json::from_str(&String::from_utf8(new_message).unwrap()).unwrap();
            current_message_id += 1;
            new_messages.push(new_message);
        }

        let mut self_message_success = None;
        if !request.self_message_lines.is_empty() {
            let new_message_id =
                ass.get(format!("{}/last message id", request.room_name).as_bytes());
            let new_message_id: u64 = new_message_id.map_or_else(
                || 0,
                |id| String::from_utf8(id).unwrap().parse::<u64>().unwrap() + 1,
            );
            let new_message_id_string = new_message_id.to_string();
            ass.set(
                format!("{}/last message id", request.room_name).as_bytes(),
                new_message_id_string.as_bytes(),
            );
            let utc_unix_timestamp = chrono::Utc::now().timestamp();
            let new_message = serde_json::json!({
                "lines": request.self_message_lines,
                "id": new_message_id,
                "utc_unix_timestamp": utc_unix_timestamp,
                "sender_name": request.self_name,
            });
            ass.set(
                format!("{}/messages/{}", request.room_name, new_message_id).as_bytes(),
                new_message.to_string().as_bytes(),
            );
            self_message_success = Some(serde_json::json!({
                "utc_unix_timestamp": utc_unix_timestamp,
                "id": new_message_id,
            }));
        }
        rouille::Response::text(
            serde_json::json!({
                "self_message_success": self_message_success,
                "new_messages": new_messages,
            })
            .to_string(),
        )
    })
    .unwrap();
    let (join_handle, stopper) = server.stoppable();
    std::thread::spawn(move || {
        for _sig in signal_hook::iterator::Signals::new([
            signal_hook::consts::SIGTERM,
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGQUIT,
        ])
        .unwrap()
        .forever()
        {
            stopper.send(()).unwrap();
        }
    });
    join_handle.join().unwrap();
}
