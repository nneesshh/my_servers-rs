use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use atomic::{Atomic, Ordering};
use hashbrown::HashMap;
use net_packet::CmdId;
use prost::Message as ProstMessage;

use commlib::NodeId;
use srv_helper::{ConnId, TcpConn};

use super::NetProxy;

static WAITER_NEXT_ID: Atomic<u64> = Atomic::new(1_u64);

///
#[allow(dead_code)]
pub enum ServerType {
    Unknown = 0,
    ClientSrv = 1000,
    GateWaySrv = 1001,
    CrossSrv = 1002,
    GameSrv = 1003,
    DBSrv = 1004,
    BattleSrv = 1005,
    WorldSrv = 1006,
    GMSrv = 1007,
    CommonSrv = 1008,
    LogSrv = 1009,
    ChatSrv = 1010,
    SocialSrv = 1011,
    GuildSrv = 1012,
}

///
pub type ReturnHander = Box<dyn Fn(&mut NetProxy, &TcpConn, CmdId, &[u8])>;

///节点数据
pub struct NodeData {
    pub nid: NodeId,                //节点ID
    conn_opt: Option<Arc<TcpConn>>, //tcp 连接信息
}
struct Waiter {
    nodes: Vec<NodeId>,
    cb: Box<dyn Fn()>,
}

///
pub struct Cluster {
    // my NodeData
    pub my_node: NodeData,

    // nid,NodeData
    pub nodes: HashMap<NodeId, Rc<NodeData>>,

    // node ready callback
    node_ready_cb: Box<dyn Fn(&mut NetProxy, &NodeData)>,

    // handshake callback
    handshake_cb: Box<dyn Fn(&mut NetProxy, &proto_inner::InnerNodeInfo)>,

    // wait node list
    wait_nodes: Vec<Waiter>,

    // waiting packet
    waiting_handlers: HashMap<NodeId, ReturnHander>,
}

impl Cluster {
    ///
    pub fn new(net_proxy: &mut NetProxy) -> Rc<RefCell<Self>> {
        let cluster = Self {
            my_node: NodeData {
                nid: 0,
                conn_opt: None,
            },
            nodes: HashMap::default(),
            node_ready_cb: Box::new(|_1, _2| {}),
            handshake_cb: Box::new(|_1, _2| {}),
            wait_nodes: Vec::default(),
            waiting_handlers: HashMap::default(),
        };
        let cluster = Rc::new(RefCell::new(cluster));
        regitser_packet_handler(net_proxy, &cluster);
        cluster
    }

    /// 连接成功后，发送节点信息
    pub fn on_connect(&self, net_proxy: &mut NetProxy, conn: &Arc<TcpConn>) {
        let mut req = proto_inner::InnerNodeInfo {
            nid: 0,
            r#type: 0,
            sids: Vec::default(),
            kv: Vec::default(),
            maxnum: 0i32,
        };

        req.nid = self.my_node.nid;
        req.r#type = self.my_node.nid as i32;

        //
        net_proxy.send_proto(
            &*conn,
            proto_inner::InnerReservedCmd::IrcNodeHandshake as CmdId,
            &req,
        );
    }

    /// 连接断开
    pub fn on_close(&mut self, hd: ConnId) {
        self.nodes.retain(|_, node| {
            if let Some(conn) = &node.conn_opt {
                if conn.hd == hd {
                    return false; // remove
                }
            }

            true
        });
    }

    /// 握手
    pub fn handle_node_handshake(
        &mut self,
        net_proxy: &mut NetProxy,
        conn: &TcpConn,
        cmd: CmdId,
        data: &[u8],
    ) {
        let result = proto_inner::InnerNodeInfo::decode(data);
        match result {
            Ok(msg) => {
                self.update_node_info(net_proxy, conn, &msg);

                let my = &self.my_node;
                let ntf = proto_inner::InnerNodeInfo {
                    nid: my.nid,
                    r#type: my.nid as i32,
                    sids: Vec::default(),
                    kv: Vec::default(),
                    maxnum: 0,
                };

                (self.handshake_cb)(net_proxy, &ntf);

                net_proxy.send_proto(
                    conn,
                    proto_inner::InnerReservedCmd::IrcNodeInfoNtf as CmdId,
                    &ntf,
                );
            }
            Err(err) => {
                log::error!(
                    "[Cluster::handle_node_handshake()] InnerNodeInfo decode failed err:{}, cmd={cmd:?}",
                    err
                );
            }
        }
    }

    /// 握手成功通知
    pub fn handle_node_info_notify(
        &mut self,
        net_proxy: &mut NetProxy,
        conn: &TcpConn,
        cmd: CmdId,
        data: &[u8],
    ) {
        let result = proto_inner::InnerNodeInfo::decode(data);
        match result {
            Ok(msg) => {
                self.update_node_info(net_proxy, conn, &msg);
            }
            Err(err) => {
                log::error!(
                    "[Cluster::handle_node_info_notify()] InnerNodeInfo decode failed err:{}, cmd={cmd:?}",
                    err
                );
            }
        }
    }

    /// 回包
    pub fn handle_rpc_return(
        &mut self,
        proxy: &mut NetProxy,
        conn: &TcpConn,
        cmd: CmdId,
        data: &[u8],
    ) {
        let result = proto_inner::InnerRpcReturn::decode(data);
        match result {
            Ok(msg) => {
                let wait_handler = &mut self.waiting_handlers;
                if let Some(cb) = wait_handler.get(&msg.rpc_id) {
                    cb(proxy, conn, cmd, data);
                    wait_handler.remove(&msg.rpc_id);
                } else {
                    log::error!(
                        "[Cluster::handle_rpc_return()] InnerRpcReturn call back err not register rpcid:{}",
                        msg.rpc_id
                    );
                }
            }
            Err(err) => {
                log::error!(
                    "[Cluster::handle_rpc_return()] InnerRpcReturn decode failed err:{}",
                    err
                );
            }
        }
    }

    /// 设置成功连接回调
    pub fn wait_connected<F>(&mut self, nodes: Vec<NodeId>, f: F)
    where
        F: Fn() + 'static,
    {
        let wait = Waiter {
            nodes,
            cb: Box::new(f),
        };

        self.wait_nodes.push(wait);
    }

    /// 更新等待的节点列表
    pub fn check_waiter(&mut self) {
        self.wait_nodes.retain(|waiter| {
            for nid in &waiter.nodes {
                let ret = self.nodes.get(nid);
                if ret.is_some() && self.my_node.nid != *nid {
                    (waiter.cb)();
                    return false; // remove
                }
            }

            true
        });
    }

    /// set node ready callback
    pub fn set_node_ready_cb<F>(&mut self, f: F)
    where
        F: Fn(&mut NetProxy, &NodeData) + 'static,
    {
        self.node_ready_cb = Box::new(f);
    }

    /// 设置自己节点信息
    pub fn set_my_node_info(&mut self, nid: NodeId) {
        self.my_node = NodeData {
            nid,
            conn_opt: None,
        };
    }

    /// 更新节点数据
    pub fn update_node_info(
        &mut self,
        net_proxy: &mut NetProxy,
        conn: &TcpConn,
        info: &proto_inner::InnerNodeInfo,
    ) {
        //
        let hd = conn.hd;
        let nid = info.nid;
        let arced_conn = net_proxy.get_conn(hd);

        //
        let node_opt = self.nodes.remove(&nid);
        match node_opt {
            Some(node) => {
                if let Some(node_conn) = &node.conn_opt {
                    if node_conn.hd != hd {
                        log::info!(
                            "[Cluster::update_node_info()][hd={}] info.hd={}",
                            hd,
                            node_conn.hd,
                        );
                    }
                } else {
                    log::info!(
                        "[Cluster::update_node_info() [hd={}] my_node: {}",
                        hd,
                        self.my_node.nid
                    );
                }
            }
            None => {
                log::info!(
                    "[Cluster::update_node_info()] insert node [hd={}] my_node: {}",
                    hd,
                    self.my_node.nid
                );
            }
        }

        //
        let node = NodeData {
            nid,
            conn_opt: Some(arced_conn.clone()),
        };
        (self.node_ready_cb)(net_proxy, &node);
        self.nodes.insert(
            nid,
            Rc::new(NodeData {
                nid,
                conn_opt: Some(arced_conn.clone()),
            }),
        );

        // 有新的节点数据上报过来，更新一下等待的节点列表
        self.check_waiter();
    }

    /// 消息发送
    #[inline(always)]
    pub fn send(
        &mut self,
        net_proxy: &mut NetProxy,
        conn: &TcpConn,
        cmd: CmdId,
        msg: &impl ProstMessage,
    ) {
        net_proxy.send_proto(conn, cmd, msg);
    }

    ///
    #[inline(always)]
    pub fn send_to_world(&mut self, net_proxy: &mut NetProxy, cmd: CmdId, msg: &impl ProstMessage) {
        self.send_to_server(net_proxy, ServerType::WorldSrv as NodeId, cmd, msg);
    }

    /// 发送到指定节点
    #[inline(always)]
    pub fn send_to_server(
        &mut self,
        net_proxy: &mut NetProxy,
        nid: NodeId,
        cmd: CmdId,
        msg: &impl ProstMessage,
    ) {
        let node_opt = self.nodes.get(&nid);
        if let Some(node) = node_opt {
            if let Some(conn) = &node.conn_opt {
                net_proxy.send_proto(&conn, cmd, msg);
            } else {
                log::error!(
                    "[Cluster::send_to_server()] cmd:{} nid:{} send failed because conn is none!!!",
                    cmd,
                    nid
                );
            }
        } else {
            log::error!(
                "[Cluster::send_to_server()] cmd:{} nid:{} send failed because node not exists!!!",
                cmd,
                nid
            );
        }
    }

    /// 发送到所有节点
    #[inline(always)]
    pub fn send_to_all_nodes(
        &mut self,
        net_proxy: &mut NetProxy,
        cmd: CmdId,
        msg: &impl ProstMessage,
    ) {
        for (_, node) in &self.nodes {
            if let Some(conn) = &node.conn_opt {
                net_proxy.send_proto(&conn, cmd, msg);
            } else {
                log::error!(
                    "[Cluster::send_to_all_nodes()] cmd:{} nid:{} send failed because conn is none!!!",
                    cmd,
                    node.nid
                );
            }
        }
    }

    /// 发送到指定节点
    #[inline(always)]
    pub fn rpc_to_server(
        &mut self,
        net_proxy: &mut NetProxy,
        nid: NodeId,
        cmd: CmdId,
        msg: &impl ProstMessage,
    ) {
        self.send_to_server(net_proxy, nid, cmd, msg);
    }

    /// 发送到指定节点 等待回包
    #[inline(always)]
    pub fn call_rpc_to_server<F>(
        &mut self,
        net_proxy: &mut NetProxy,
        nid: NodeId,
        cmd: CmdId,
        msg: &impl ProstMessage,
        f: F,
    ) where
        F: Fn(&mut NetProxy, &TcpConn, CmdId, &[u8]) + 'static,
    {
        let id = WAITER_NEXT_ID.fetch_add(1, Ordering::Relaxed);
        self.waiting_handlers.insert(id, Box::new(f));
        self.send_to_server(net_proxy, nid, cmd, msg);
    }
}

/// use thread local unsafe cell -- mut
#[macro_export]
macro_rules! cluster_register_packet_handler {
    ($net_proxy:ident, $cmd:path, $member_fn:ident, $source:ident) => {{
        let clone1 = $source.clone();
        $net_proxy.set_packet_handler($cmd as CmdId, move |proxy, conn, cmd, data| {
            let ret = clone1.try_borrow_mut();
            match ret {
                Ok(mut s) => {
                    paste::paste! {
                        s.[< $member_fn >](proxy, conn, cmd, data);
                    }
                }
                Err(err) => {
                    log::error!("source try_borrow_mut error: {:?}!!! cmd={cmd:?}!!!", err);
                }
            }
        });
    }};
}

fn regitser_packet_handler(net_proxy: &mut NetProxy, cluster: &Rc<RefCell<Cluster>>) {
    //
    cluster_register_packet_handler!(
        net_proxy,
        proto_inner::InnerReservedCmd::IrcNodeHandshake,
        handle_node_handshake,
        cluster
    );
    cluster_register_packet_handler!(
        net_proxy,
        proto_inner::InnerReservedCmd::IrcNodeInfoNtf,
        handle_node_info_notify,
        cluster
    );
    cluster_register_packet_handler!(
        net_proxy,
        proto_inner::InnerReservedCmd::IrcRpcReturn,
        handle_rpc_return,
        cluster
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use net_packet::{take_small_packet, CmdId};
    use prost::Message as ProstMessage;
    use srv_helper::{ConnId, PacketType, TcpConn};

    use crate::net_packet_encdec::encode_packet_server;
    use crate::write_prost_message;
    use crate::{Cluster, NetProxy};

    #[test]
    fn cluster_packet_handler() {
        //
        let mut net_proxy = NetProxy::new(PacketType::Server);
        let hd = ConnId::from(1);
        let conn = Arc::new(TcpConn::new(hd));
        let _cluster = Cluster::new(&mut net_proxy);
        net_proxy.add_conn(hd, &conn);

        // call packet handler
        {
            let cmd = proto_inner::InnerReservedCmd::IrcNodeInfoNtf as CmdId;
            let info = proto_inner::InnerNodeInfo {
                nid: 1234,
                r#type: 1234,
                ..Default::default()
            };

            let len = info.encoded_len();
            let mut pkt = take_small_packet();
            pkt.set_cmd(cmd);
            let pb_slice: &mut [u8] = pkt.as_write_mut();
            write_prost_message(&info, pb_slice);
            pkt.end_write(len);

            encode_packet_server(&mut pkt);
            net_proxy.on_net_packet(&conn, pkt);
        }
    }
}
