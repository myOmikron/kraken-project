pub mod rpc_attacks {

    pub mod shared {
        tonic::include_proto!("attacks.shared");
    }

    tonic::include_proto!("attacks");
}
