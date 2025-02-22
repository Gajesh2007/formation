use std::{collections::{btree_map::{Iter, IterMut}, BTreeMap}, fmt::Display, net::IpAddr};
use crdts::{map::Op, merkle_reg::Sha3Hash, BFTReg, CmRDT, Map, bft_reg::Update};
use form_dns::store::FormDnsRecord;
use form_types::state::{Response, Success};
use k256::ecdsa::SigningKey;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use tiny_keccak::Hasher;
use crate::Actor;

pub type InstanceOp = Op<String, BFTReg<Instance, Actor>, Actor>; 

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstanceStatus {
    Building,
    Built,
    Created,
    Started,
    Stopped,
    Killed,
    CriticalError,
}

impl Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceStatus::Building => writeln!(f, "{}", "Building"),
            InstanceStatus::Built => writeln!(f, "{}", "Built"),
            InstanceStatus::Created => writeln!(f, "{}", "Created"),
            InstanceStatus::Started => writeln!(f, "{}", "Started"),
            InstanceStatus::Stopped => writeln!(f, "{}", "Stopped"),
            InstanceStatus::Killed => writeln!(f, "{}", "Killed"),
            InstanceStatus::CriticalError => writeln!(f, "{}", "Critical Error"),

        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instance {
    pub instance_id: String,
    pub node_id: String,
    pub build_id: String,
    pub instance_owner: String,
    pub formnet_ip: Option<IpAddr>,
    pub dns_record: Option<FormDnsRecord>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_snapshot: i64,
    pub status: InstanceStatus,
    pub host_region: String,
    pub resources: InstanceResources,
    pub cluster: InstanceCluster,
    /// Base64 encoded formfile
    pub formfile: String, 
    pub snapshots: Option<Snapshots>,
    pub metadata: InstanceMetadata,
}

impl Default for Instance {
    fn default() -> Self {
        let null_hash = [0u8; 32];
        let null_hex = hex::encode(null_hash);
        Self {
            instance_id: null_hex.clone(), 
            node_id: null_hex.clone(), 
            build_id: null_hex.clone(), 
            instance_owner: null_hex.clone(),
            formnet_ip: None,
            dns_record: None,
            created_at: 0,
            updated_at: 0,
            last_snapshot: 0,
            status: InstanceStatus::Building,
            host_region: String::new(),
            resources: Default::default(),
            cluster: Default::default(),
            formfile: String::new(),
            snapshots: None,
            metadata: Default::default()

        }
    }
}

impl Sha3Hash for Instance {
    fn hash(&self, hasher: &mut tiny_keccak::Sha3) {
        hasher.update(&bincode::serialize(self).unwrap());
    }
}

impl Instance {
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn instance_owner(&self) -> &str {
        &self.instance_owner
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    pub fn updated_at(&self) -> i64 {
        self.updated_at
    }

    pub fn last_snapshot(&self) -> i64 {
        self.last_snapshot
    }

    pub fn host_region(&self) -> &str {
        &self.host_region
    }

    pub fn resources(&self) -> &InstanceResources {
        &self.resources
    }

    pub fn cluster(&self) -> &InstanceCluster {
        &self.cluster
    }

    pub fn formfile(&self) -> &str {
        &self.formfile
    }

    pub fn snapshots(&self) -> &Option<Snapshots> {
        &self.snapshots
    }

    pub fn metadata(&self) -> &InstanceMetadata {
        &self.metadata
    }

    pub fn vcpus(&self) -> u8 {
        self.resources.vcpus()
    }

    pub fn memory_mb(&self) -> u32 {
        self.resources.memory_mb() 
    }

    pub fn bandwidth_mbps(&self) -> u32 {
        self.resources.bandwidth_mbps()
    }

    pub fn gpu(&self) -> Option<InstanceGpu> {
        self.resources.gpu()
    }

    pub fn gpu_count(&self) -> Option<u8> {
        self.resources.gpu_count()
    }

    pub fn gpu_model(&self) -> Option<&str> {
        self.resources.gpu_model()
    }

    pub fn gpu_vram_mp(&self) -> Option<u32> {
        self.resources.gpu_vram_mb()
    }

    pub fn gpu_usage(&self) -> Option<u32> {
        self.resources.gpu_usage()
    }

    pub fn cluster_members(&self) -> &BTreeMap<String, ClusterMember> {
        self.cluster().members()
    }

    pub fn is_cluster_member(&self, id: &str) -> bool {
        self.cluster().contains_key(id)
    }

    pub fn get_cluster_member(&self, id: &str) -> Option<&ClusterMember> {
        self.cluster().get(id)
    }

    pub fn get_cluster_member_status(&self, id: &str) -> Option<&str> {
        self.cluster().get_member_status(id)
    }

    pub fn get_cluster_member_last_heartbeat(&self, id: &str) -> Option<i64> {
        self.cluster().get_member_last_heartbeat(id)
    }

    pub fn get_cluster_member_heartbeats_skipped(&self, id: &str) -> Option<u32> {
        self.cluster().get_member_heartbeats_skipped(id)
    }

    pub fn insert_cluster_member(&mut self, member: ClusterMember) {
        self.cluster.members_mut().insert(member.id().to_string(), member.clone());
    }

    pub fn remove_cluster_member(&mut self, id: &str) -> Option<ClusterMember> {
        self.cluster.remove(id)
    }

    pub fn cluster_member_iter(&self) -> Iter<String, ClusterMember> {
        self.cluster.iter()
    }

    pub fn cluster_member_iter_mut(&mut self) -> IterMut<String, ClusterMember> {
        self.cluster.iter_mut()
    }

    pub fn formfile_b64(&self) -> &str {
        &self.formfile
    }

    pub fn n_snapshots_ago(&self, mut n: u32) -> (Option<Snapshots>, u32) {
        let mut current: Option<Snapshots> = self.snapshots().clone();
        while n > 0 {
            if let Some(ref c) = &current {
                let next = *c.previous_snapshot.clone(); 
                if !next.is_none() {
                    current = next;
                    n -= 1;
                } else {
                    return (current.clone(), n);
                }
            } else {
                return (current, n)
            }
        }

        return (current, n);
    }

    pub fn tags(&self) -> Vec<String> {
        self.metadata().tags()
    }

    pub fn description(&self) -> &str {
        self.metadata().description()
    }

    pub fn annotations(&self) -> &InstanceAnnotations {
        self.metadata().annotations()
    }

    pub fn security(&self) -> &InstanceSecurity {
        self.metadata().security()
    }

    pub fn monitoring(&self) -> &InstanceMonitoring {
        self.metadata().monitoring()
    }

    pub fn encryption(&self) -> &InstanceEncryption {
        self.security().encryption()
    }

    pub fn tee(&self) -> bool {
        self.security().tee()
    }

    pub fn hsm(&self) -> bool {
        self.security().hsm()
    }

    pub fn is_encrypted(&self) -> bool {
        self.encryption().is_encrypted()
    }

    pub fn scheme(&self) -> Option<String> {
        self.encryption().scheme()
    }

    pub async fn get(id: &str) -> Option<Self> {
        let resp = Client::new()
            .get(format!("http://127.0.0.1:3004/instance/{}/get", id))
            .send().await.ok()?
            .json::<Response<Self>>().await.ok()?;

        match resp {
            Response::Success(Success::Some(instance)) => return Some(instance),
            _ => return None,
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceResources {
    pub vcpus: u8,
    pub memory_mb: u32,
    pub bandwidth_mbps: u32,
    pub gpu: Option<InstanceGpu>
}

impl InstanceResources {
    pub fn vcpus(&self) -> u8 {
        self.vcpus
    }

    pub fn memory_mb(&self) -> u32 {
        self.memory_mb
    }

    pub fn bandwidth_mbps(&self) -> u32 {
        self.bandwidth_mbps
    }

    pub fn gpu(&self) -> Option<InstanceGpu> {
        self.gpu.clone()
    }

    pub fn gpu_count(&self) -> Option<u8> {
        if let Some(gpu) = &self.gpu {
            return Some(gpu.count())
        }
        None
    }

    pub fn gpu_model(&self) -> Option<&str> {
        if let Some(gpu) = &self.gpu {
            return Some(gpu.model())
        }
        None
    }

    pub fn gpu_vram_mb(&self) -> Option<u32> {
        if let Some(gpu) = &self.gpu {
            return Some(gpu.vram_mb())
        }
        None
    }

    pub fn gpu_usage(&self) -> Option<u32> {
        if let Some(gpu) = &self.gpu {
            return Some(gpu.usage())
        }
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceGpu {
    pub count: u8,
    pub model: String,
    pub vram_mb: u32,
    pub usage: u32,
}

impl InstanceGpu {
    pub fn count(&self) -> u8 {
        self.count
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn vram_mb(&self) -> u32 {
        self.vram_mb
    } 

    pub fn usage(&self) -> u32 {
        self.usage
    }

}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceCluster {
    pub members: BTreeMap<String, ClusterMember>
}

impl InstanceCluster {
    pub fn members(&self) -> &BTreeMap<String, ClusterMember> {
        &self.members
    }

    pub fn members_mut(&mut self) -> &mut BTreeMap<String, ClusterMember> {
        &mut self.members
    }

    pub fn get(&self, id: &str) -> Option<&ClusterMember> {
        self.members.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ClusterMember> {
        self.members.get_mut(id)
    }

    pub fn insert(&mut self, member: ClusterMember) {
        let id = member.id();
        self.members.insert(id.to_string(), member);
    }

    pub fn iter(&self) -> Iter<String, ClusterMember> {
        self.members.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<String, ClusterMember> {
        self.members.iter_mut()
    }

    pub fn remove(&mut self, id: &str) -> Option<ClusterMember> {
        self.members.remove(id)
    }

    pub fn contains_key(&self, id: &str) -> bool {
        self.members.contains_key(id)
    }

    pub fn get_member_status(&self, id: &str) -> Option<&str> {
        if let Some(member) = self.get(id) {
            return Some(member.status())
        }

        None
    }

    pub fn get_member_last_heartbeat(&self, id: &str) -> Option<i64> {
        if let Some(member) = self.get(id) {
            return Some(member.last_heartbeat()) 
        }

        None
    }

    pub fn get_member_heartbeats_skipped(&self, id: &str) -> Option<u32> {
        if let Some(member) = self.get(id) {
            return Some(member.heartbeats_skipped())
        }

        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClusterMember {
    pub node_id: String,
    pub node_public_ip: IpAddr,
    pub node_formnet_ip: IpAddr,
    pub instance_id: String,
    pub instance_formnet_ip: IpAddr,
    pub status: String,
    pub last_heartbeat: i64,
    pub heartbeats_skipped: u32,
}

impl ClusterMember {
    pub fn id(&self) -> &str {
        &self.instance_id
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn last_heartbeat(&self) -> i64 {
        self.last_heartbeat
    }

    pub fn heartbeats_skipped(&self) -> u32 {
        self.heartbeats_skipped
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snapshots {
    pub snapshot_id: String,
    pub timestamp: i64,
    pub description: Option<String>,
    pub previous_snapshot: Box<Option<Snapshots>>
}

impl Snapshots {
    pub fn id(&self) -> &str {
        &self.snapshot_id
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn previous_snapshot(&self) -> Box<Option<Snapshots>> {
        self.previous_snapshot.clone()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceMetadata {
    pub tags: Vec<String>,
    pub description: String,
    pub annotations: InstanceAnnotations,
    pub security: InstanceSecurity,
    pub monitoring: InstanceMonitoring
}

impl InstanceMetadata {
    pub fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn annotations(&self) -> &InstanceAnnotations {
        &self.annotations
    }

    pub fn security(&self) -> &InstanceSecurity {
        &self.security
    }

    pub fn monitoring(&self) -> &InstanceMonitoring {
        &self.monitoring
    }

    pub fn deployed_by(&self) -> &str {
        &self.annotations.deployed_by()
    }

    pub fn network_id(&self) -> u16 {
        self.annotations.network_id()
    }

    pub fn build_commit(&self) -> Option<String> {
        self.annotations.build_commit.clone()
    }

    pub fn is_encrypted(&self) -> bool {
        self.security.encryption().is_encrypted()
    }

    pub fn encryption_scheme(&self) -> Option<String> {
        self.security.encryption().scheme()
    }

    pub fn is_tee_enabled(&self) -> bool {
        self.security.tee()
    }

    pub fn is_hsm_enabled(&self) -> bool {
        self.security.hsm()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceAnnotations {
    pub deployed_by: String,
    pub network_id: u16,
    pub build_commit: Option<String>
}

impl InstanceAnnotations {
    pub fn deployed_by(&self) -> &str {
        &self.deployed_by
    }

    pub fn network_id(&self) -> u16 {
        self.network_id
    }

    pub fn build_commit(&self) -> Option<String> {
        self.build_commit.clone()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceSecurity {
    pub encryption: InstanceEncryption,
    pub tee: bool,
    pub hsm: bool,
}

impl InstanceSecurity {
    pub fn encryption(&self) -> &InstanceEncryption {
        &self.encryption
    }

    pub fn tee(&self) -> bool {
        self.tee
    }

    pub fn hsm(&self) -> bool {
        self.hsm
    }

    pub fn is_encrypted(&self) -> bool {
        self.encryption.is_encrypted()
    }

    pub fn scheme(&self) -> Option<String> {
        self.encryption.scheme()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceMonitoring {
    pub logging_enabled: bool,
    pub metrics_endpoint: String,
}

impl InstanceMonitoring {
    pub fn logging_enabled(&self) -> bool {
        self.logging_enabled
    }

    pub fn metrics_endpoint(&self) -> &str {
        &self.metrics_endpoint
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceEncryption {
    pub is_encrypted: bool,
    pub  scheme: Option<String>,
}

impl InstanceEncryption {
    pub fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }

    pub fn scheme(&self) -> Option<String> {
        self.scheme.clone()
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceState {
    node_id: String,
    pk: String,
    pub map: Map<String, BFTReg<Instance, Actor>, Actor> 
}


impl InstanceState {

    pub fn new(node_id: String, pk: String) -> Self {
        Self {
            node_id,
            pk,
            map: Map::new()
        }
    }

    pub fn map(&self) -> Map<String, BFTReg<Instance, Actor>, Actor> {
        self.map.clone()
    }

    pub fn update_instance_local(&mut self, instance: Instance) -> InstanceOp {
        log::info!("Acquiring add ctx...");
        let add_ctx = self.map.read_ctx().derive_add_ctx(self.node_id.clone());
        log::info!("Decoding our private key...");
        let signing_key = SigningKey::from_slice(
            &hex::decode(self.pk.clone())
                .expect("PANIC: Invalid SigningKey Cannot Decode from Hex"))
                .expect("PANIC: Invalid SigningKey cannot recover ffrom Bytes");
        log::info!("Creating op...");
        let op = self.map.update(instance.instance_id().to_string(), add_ctx, |reg, _ctx| {
            let op = reg.update(instance.into(), self.node_id.clone(), signing_key).expect("PANIC: Unable to sign updates");
            op
        });
        log::info!("Op created, returning...");
        op
    }

    pub fn remove_instance_local(&mut self, id: String) -> InstanceOp {
        log::info!("Acquiring remove context...");
        let rm_ctx = self.map.read_ctx().derive_rm_ctx();
        log::info!("Building Rm Op...");
        self.map.rm(id, rm_ctx)
    }

    pub fn instance_op(&mut self, op: InstanceOp) -> Option<(String, String)> {
        log::info!("Applying peer op");
        self.map.apply(op.clone());
        match op {
            Op::Up { dot, key, op: _ } => Some((dot.actor, key)),
            Op::Rm { .. } => None
        }
    }

    pub fn instance_op_success(&self, key: String, update: Update<Instance, String>) -> (bool, Instance) {
        if let Some(reg) = self.map.get(&key).val {
            if let Some(v) = reg.val() {
                // If the in the updated register equals the value in the Op it
                // succeeded
                if v.value() == update.op().value {
                    return (true, v.value()) 
                // Otherwise, it could be that it's a concurrent update and was added
                // to the DAG as a head
                } else if reg.dag_contains(&update.hash()) && reg.is_head(&update.hash()) {
                    return (true, v.value()) 
                // Otherwise, we could be missing a child, and this particular update
                // is orphaned, if so we should requst the child we are missing from
                // the actor who shared this update
                } else if reg.is_orphaned(&update.hash()) {
                    return (true, v.value())
                // Otherwise it was a no-op for some reason
                } else {
                    return (false, v.value()) 
                }
            } else {
                return (false, update.op().value) 
            }
        } else {
            return (false, update.op().value);
        }
    }

    pub fn get_instance(&self, key: String) -> Option<Instance> {
        if let Some(reg) = self.map.get(&key).val {
            if let Some(v) = reg.val() {
                return Some(v.value())
            }
        }

        return None
    }

    pub fn get_instances_by_build_id(&self, build_id: String) -> Vec<Instance> {
        let mut instances = vec![];
        for ctx in self.map.iter() {
            let (_, reg) = ctx.val;
            if let Some(val) = reg.val() {
                let instance = val.value();
                if instance.build_id == build_id {
                    instances.push(instance)
                }
            }
        }

        instances
    }

    pub fn get_instance_by_ip(&self, ip: IpAddr) -> Result<Instance, Box<dyn std::error::Error>> {
        let mut instance_opt: Option<Instance> = None; 
        for ctx in self.map.iter() {
            let (_, reg) = ctx.val;
            if let Some(val) = reg.val() {
                let instance = val.value();
                if let Some(formnet_ip) = instance.formnet_ip {
                    if ip == formnet_ip { 
                        instance_opt = Some(instance);
                    }
                }
            }
        }

        Ok(instance_opt.ok_or(
            Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Unable to find instance with ip {ip}")
                )
            )
        )?)
    }
}
