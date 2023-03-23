use std::sync::Arc;
use crate::dht::kademlia::KademliaDHT;
use crate::network::client::Client;
use crate::network::datagram::{Datagram, DatagramType};
use crate::network::node::Node;
use crate::network::server::Server;

#[test]
fn two_way_handshake() {
    let mut data = &Datagram {
        data_type: DatagramType::REQUEST,
        token_id: "test".to_string(),
        source: "127.0.0.1:1234".to_string(),
        destination: "127.0.0.1:8000".to_string(),
        data: "this is a test".to_string()
    };
    let kill = &Datagram {
        data_type: DatagramType::KILL,
        token_id: "test".to_string(),
        source: "127.0.0.1:1234".to_string(),
        destination: "127.0.0.1:8000".to_string(),
        data: "this is a test".to_string()
    };

    let kill2 = &Datagram {
        data_type: DatagramType::KILL,
        token_id: "test".to_string(),
        source: "127.0.0.1:8000".to_string(),
        destination: "127.0.0.1:1234".to_string(),
        data: "this is a test".to_string()
    };

    let current_node = Node::new("127.0.0.1".to_string(),1234);
    let remote_node = Node::new("127.0.0.1".to_string(),8000);

    let kad = Arc::new(KademliaDHT::new(current_node.clone(),None));
    let kad2 = Arc::new(KademliaDHT::new(remote_node.clone(),None));

    let threa1 = Server::new(kad.clone()).start_service();
    let threa2 = Server::new(kad2.clone()).start_service();

    let client = Client::new(kad.service.clone());
    let client2 = Client::new(kad2.service.clone());

    let rec :Datagram = client.clone().make_request(&data.clone()).recv().unwrap().unwrap();


    client2.make_request(kill2);
    client.make_request(kill);

    threa1.join().expect("thead 1 dead");
    threa2.join().expect("thread 2 dead");

    assert_eq!(rec.data, data.data);
    assert_eq!(rec.token_id, data.token_id);
    assert_eq!(rec.source, data.destination);
    assert_eq!(rec.destination, data.source);

}

#[test]
fn test_no_response(){
    let mut data = &Datagram {
        data_type: DatagramType::REQUEST,
        token_id: "test".to_string(),
        source: "127.0.0.1:8080".to_string(),
        destination: "127.0.0.1:12345".to_string(),
        data: "this is a test".to_string()
    };
    let kill = &Datagram {
        data_type: DatagramType::KILL,
        token_id: "test".to_string(),
        source: "127.0.0.1:8080".to_string(),
        destination: "127.0.0.1:8080".to_string(),
        data: "this is a test".to_string()
    };

    let current_node = Node::new("127.0.0.1".to_string(),8080);

    let kad = Arc::new(KademliaDHT::new(current_node.clone(),None));

    let threa1 = Server::new(kad.clone()).start_service();

    let client = Client::new(kad.service.clone());

    let rec : Option<Datagram> = client.clone().make_request(&data.clone()).recv().unwrap();


    client.make_request(kill);
    threa1.join().expect("thead 1 dead");

    assert_eq!(rec, None);
}
