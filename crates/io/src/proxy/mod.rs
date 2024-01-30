use boringtun::{
    noise::Tunn,
    x25519::{PublicKey, StaticSecret},
};

pub trait NetworkProxy {}

pub fn get_proxy() -> Result<Box<dyn NetworkProxy>, eyre::Report> {
    let static_private = StaticSecret::from([0; 32]);
    let peer_static_public = PublicKey::from([0; 32]);

    #[allow(unused_variables)]
    let a = Tunn::new(static_private, peer_static_public, None, None, 0, None);

    todo!()
}
