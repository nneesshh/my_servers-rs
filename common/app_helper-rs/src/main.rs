use commlib::{PacketType, TcpConn, ConnId};
use net_packet::{take_small_packet, CmdId};

use app_helper::net_packet_encdec::encode_packet_server;
use app_helper::write_prost_message;
use app_helper::{proto, Cluster, NetProxy};
use prost::Message;

fn main() {
    //
    let mut net_proxy = NetProxy::new(PacketType::Server);
    let hd = ConnId::from(123);
    let conn = TcpConn::new(hd);
    let _cluster = Cluster::new(&mut net_proxy);

    // call packet handler
    {
        let cmd = proto::InnerReservedCmd::IrcNodeInfoNtf as CmdId;
        let info = proto::InnerNodeInfo {
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
