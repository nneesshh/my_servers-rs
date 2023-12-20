/// 对等节点
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Peer {
    /// peer name
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// addr such as "ip:port"
    #[prost(string, tag = "2")]
    pub raddr: ::prost::alloc::string::String,
}
/// 上行：注册对等节点
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct P2sRegisterPeer {
    /// 对等节点
    #[prost(message, optional, tag = "1")]
    pub peer: ::core::option::Option<Peer>,
}
/// 上行：注销对等节点
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct P2sUnregisterPeer {
    /// 对等节点名称
    #[prost(string, tag = "1")]
    pub peer_name: ::prost::alloc::string::String,
}
/// 下行：对等节点列表
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2pPeerList {
    /// 对等节点列表
    #[prost(message, repeated, tag = "1")]
    pub peer_list: ::prost::alloc::vec::Vec<Peer>,
}
/// 下行：添加对等节点通知
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2pPeerNotificationAdded {
    /// 对等节点
    #[prost(message, optional, tag = "1")]
    pub peer: ::core::option::Option<Peer>,
}
/// 下行：移除对等节点通知
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2pPeerNotificationRemoved {
    /// 对等节点名称
    #[prost(string, tag = "1")]
    pub peer_name: ::prost::alloc::string::String,
}
/// 平行：Greetings
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct P2pGreetings {
    /// 对等节点名称
    #[prost(string, tag = "1")]
    pub peer_name: ::prost::alloc::string::String,
    /// 问候语
    #[prost(string, tag = "2")]
    pub greeting: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EnumMsgType {
    None = 0,
    /// 加密 token
    EncryptToken = 1102,
}
impl EnumMsgType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            EnumMsgType::None => "None",
            EnumMsgType::EncryptToken => "EncryptToken",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "None" => Some(Self::None),
            "EncryptToken" => Some(Self::EncryptToken),
            _ => None,
        }
    }
}
/// 加密 token
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2cEncryptToken {
    /// 64字节加密 token
    #[prost(bytes = "vec", optional, tag = "1")]
    pub token: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
