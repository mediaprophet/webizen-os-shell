
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeStage {
    None,
    Talk,
    Mail,
    Directory,
    Wallet,
    Wellfair,
    Library,
    Projects,
    Poet,
    Settings,
}

#[derive(Clone, Debug)]
pub struct ShellState {
    pub current_stage: VolumeStage,
    pub p2p_peers_count: usize,
    pub nym_mixnet_active: bool,
    pub spatial_mode: bool,
    pub did_fingerprint: String,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            current_stage: VolumeStage::None,
            p2p_peers_count: 3,
            nym_mixnet_active: true,
            spatial_mode: true,
            did_fingerprint: "did:q42:webizen:human-sovereign-01".into(),
        }
    }
}
