use std::sync::Arc;
use std::{str, thread};
use std::thread::JoinHandle;
use log::{debug, error, info, warn};
use crate::constants::fixed_sizes::UDP_STREAMING_BUFFER_SIZE;
use crate::dht::kademlia::KademliaDHT;
use crate::network::datagram::{Datagram, DatagramType};
use crate::network::rpc_socket::RpcSocket;

#[derive(Clone, Debug)] pub struct Server {
    pub app: Arc<KademliaDHT>,
}


impl Server{
    pub fn new(app : Arc<KademliaDHT>) -> Server {
        Self{
            app
        }
    }

    pub fn start_service(self) -> JoinHandle<()> { //todo: insure only one listiner a time for a node
        info!("Initializing node services at {}", self.app.node.get_address());

        let app = self.app.clone();
        thread::spawn(move || {
            ;
            let mut buffer =  [0u8; UDP_STREAMING_BUFFER_SIZE];

            loop {
                let (size , src_addr) = match app.service.socket
                    .recv_from(&mut buffer){
                    Ok((sz, src)) => (sz, src),
                    Err(e) => {
                        error!("Failed to receive data from {}",e.to_string());
                        continue;
                    }
                };

                let payload =
                    String::from(match str::from_utf8(&buffer[..size]){
                        Ok(utf) => utf,
                        Err(_) => {
                            error!("Unable to parse string from received bytes");
                            continue;
                        }
                    });

                debug!("Node sender {}", src_addr);

                let mut data : Datagram = match serde_json::from_str(&payload) {
                    Ok(d) =>   d,
                    Err(_) =>  {
                        error!("Unable to decode string payload ");
                        debug!("Payload unknown [{}]", payload);
                        continue;
                    }
                };

                if data.destination != app.service.node.get_address() {
                    warn!("Destination address doesn't match node address, ignoring");
                    continue;
                }

                if data.source != src_addr.to_string() {
                    warn!("Source address doesn't match socket address, ignoring");
                    continue;
                }

                debug!("[Payload] {:?}", data);

                data.source= src_addr.to_string();

                match data.data_type {
                    DatagramType::REQUEST => {
                        Server::request_handler(app.clone(), data)
                    }
                    DatagramType::RESPONSE => {
                        self.clone().response_handler(data)

                    }
                    DatagramType::KILL => {break;}
                }
            }
        })

    }

    fn reply(rpc : Arc<RpcSocket>, msg: &Datagram) {
        let encoded = serde_json::to_string(msg)
            .map_err(|_| error!("Unable to serialize message")).unwrap();

        rpc.socket
            .send_to(&encoded.as_bytes(), &msg.destination)
            .map_err(|_|" Error while sending message to specified address").unwrap();
    }

    fn request_handler( app: Arc<KademliaDHT>,  payload: Datagram, ){
        thread::spawn(move || {
            let response : Datagram = KademliaDHT::handle_request(app.clone(),payload);

            Server::reply(app.service.clone(),&Datagram {
                token_id : response.token_id,
                data_type: DatagramType::RESPONSE,
                source:response.destination,
                destination: response.source,
                data: response.data
            });

        });

    }

    fn response_handler(self, payload: Datagram) {
        thread::spawn(move || {
            let app = self.app.clone();
            let mut await_response = app.service.await_response
                .lock()
                .map_err(|_| error!("Failed to acquire lock on AwaitResponse")).unwrap();

            let token = payload.token_id.clone();

            let tmp = match await_response.get(&token) {
                Some(sender) => sender.send(Some(payload)),
                None => {
                    warn!("Unsolicited response received, ignoring...");
                    return;
                }
            };

            if let Ok(_) = tmp {
                await_response.remove(&token);
            }
        });

    }

}