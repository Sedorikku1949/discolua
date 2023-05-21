use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use log::{info, trace};
use tungstenite::{client, connect, Message};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::stream::MaybeTlsStream;
use url::Url;
use crate::constants::{API_BASE_URL, GATEWAY_VERSION};

pub type Socket = tungstenite::WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct WebSocketWorker {
    /// The websocket socket
    socket: Arc<Mutex<Socket>>,
    /// The sender of the channel
    sender: flume::Sender<Message>,
    /// The receiver of the channel
    receiver: flume::Receiver<Message>,

    /// The thread handle for the send loop
    send_handle: Option<thread::JoinHandle<()>>,
    /// The thread handle for the receive loop
    recv_handle: Option<thread::JoinHandle<()>>,

    /// A boolean to tell the threads to stop
    close_client: Arc<Mutex<bool>>,
}

impl Clone for WebSocketWorker {
    fn clone(&self) -> Self {
        Self {
            socket: self.socket.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            send_handle: None,
            recv_handle: None,
            close_client: self.close_client.clone(),
        }
    }
}

/// Retrieves the URL for the gateway
/// using the [`GET /gateway`][`get-gateway`] endpoin
/// Reference:
/// https://discord.com/developers/docs/topics/gateway#get-gateway
pub fn get_ws_url() -> String {
    let url = format!("{}/gateway", API_BASE_URL);

    // create request with good agent
    let req = ureq::get(&url).set("User-Agent", crate::constants::USER_AGENT);

    let res = match req.call() {
        Ok(res) => res,
        Err(err) => panic!("Failed to get gateway URL: {}", err),
    };

    let jsoned: serde_json::Value = res.into_json().expect("Failed to parse gateway URL");

    jsoned["url"].as_str().expect("Failed to parse gateway URL").to_string()
}

impl WebSocketWorker {
    pub fn init() -> Result<Self, ()> {

        let url = Url::parse(format!("{}?v={:?}&encoding=json", get_ws_url(), GATEWAY_VERSION).as_str()).expect("Failed to parse gateway URL");

        dbg!(&url.as_str());

        trace!("Connecting to the server...");

        let (socket, response) = connect(url).expect("Can't connect");

        trace!("Connected to the server");

        let arc_socket = Arc::new(Mutex::new(socket));
        let close_client = Arc::new(Mutex::new(false));

        let (sender, receiver) = flume::unbounded();

        // send loop
        let receiver_cln = receiver.clone();
        let socket_clone = arc_socket.clone();
        let close_client_clone1 = close_client.clone();
        let send_handle = thread::spawn(move || {
            loop {
                if *close_client_clone1.lock().unwrap() { break; }

                let msg: Message = receiver_cln.recv().expect("Error reading message");
                let mut socket = socket_clone.lock().unwrap();

                // check if the socket if finished
                if !socket.can_write() { break; }

                while let Err(_) = socket.write_message(msg.clone()) {}
                drop(socket);
            }
        });

        // receive loop
        let sender_cln = sender.clone();
        let socket_clone = arc_socket.clone();
        let close_client_clone2 = close_client.clone();
        let recv_handle = thread::spawn(move || {
            loop {
                if *close_client_clone2.lock().unwrap() { break; }
                let mut socket = socket_clone.lock().unwrap();

                // check if the socket if finished
                if !socket.can_read() { break; }
                let msg = socket.read_message().expect("Error reading message");

                if msg.is_close() { break; }

                sender_cln.send(msg).unwrap();
                drop(socket);
            }
        });

        Ok(Self {
            socket: arc_socket,
            sender, receiver,
            send_handle: Some(send_handle),
            recv_handle: Some(recv_handle),
            close_client,
        })
    }

    /// Send a message to the client
    /// Return () if the client is closed
    pub fn send(&self, msg: Message) -> Result<(), ()> {
        if let Err(_) = self.sender.send(msg) {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Receive a message from the client
    /// Return () if the client is closed
    pub fn recv(&self) -> Result<Message, ()> {
        self.receiver.recv().map_err(|_| ())
    }

    /// Receive a message from the client
    /// Return () if the client is closed
    /// Auto parse the message as JSON
    pub fn recv_json(&self) -> serde_json::Result<serde_json::Value> {
        let msg = self.receiver.recv().unwrap();
        dbg!(&msg);
        serde_json::from_str(msg.to_text().unwrap_or(""))
    }

    /// Stop the client
    /// This will close the socket and stop the threads
    pub fn stop(&mut self) {
        trace!("Stopping websocket client");
        // Tell the threads to stop
        *self.close_client.lock().unwrap() = true;
        trace!("close_client set to true");

        // Wait for the threads to finish
        trace!("closing send thread");
        self.send_handle.take().unwrap().join().unwrap();
        trace!("closing recv thread");
        self.recv_handle.take().unwrap().thread().unpark();
        trace!("Threads finished");

        trace!("Stopping client");
        // Close the socket
        self.socket
            .lock()
            .unwrap()
            .close(Some(
                CloseFrame {
                    code: CloseCode::Normal,
                    reason: "Stopping client".into()
                }
            ))
            .unwrap();
        trace!("Socket closed");
    }
}