use std::collections::LinkedList;
use std::rc::Rc;
use std::sync::Arc;

use net_packet::take_packet;
use net_packet::{CmdId, NetPacketGuard};

use commlib::get_leading_field_size;
use commlib::utils::Base64;
use commlib::{ConnId, PacketType, TcpConn};

use super::net_packet_encdec::EncryptData;
use super::net_packet_encdec::{decode_packet, encode_packet};

///
pub type EncryptTokenHander = Box<dyn Fn(&mut NetProxy, &TcpConn)>;
pub type PacketHander = Box<dyn Fn(&mut NetProxy, &TcpConn, CmdId, &[u8])>;

///
pub struct NetProxy {
    packet_type: PacketType,                                   // 通信 packet 类型
    leading_field_size: u8,                                    // 包体前导长度字段占用字节数
    conn_table: hashbrown::HashMap<ConnId, Arc<TcpConn>>,      // hd -> connection info
    hd_encrypt_table: hashbrown::HashMap<ConnId, EncryptData>, // 包序号和密钥

    encrypt_token_handler: Rc<EncryptTokenHander>,
    default_handler: Rc<PacketHander>,
    handlers: hashbrown::HashMap<CmdId, Rc<PacketHander>>,
}

impl NetProxy {
    ///
    pub fn new(packet_type: PacketType) -> Self {
        let leading_field_size = get_leading_field_size(packet_type);

        Self {
            packet_type,
            leading_field_size,
            conn_table: hashbrown::HashMap::with_capacity(4096),
            hd_encrypt_table: hashbrown::HashMap::with_capacity(4096),

            encrypt_token_handler: Rc::new(Box::new(|_1, _2| {})),
            default_handler: Rc::new(Box::new(|_1, _2, _3, _4| {})),
            handlers: hashbrown::HashMap::new(),
        }
    }

    ///
    pub fn on_incomming_conn(&mut self, conn: &Arc<TcpConn>, push_encrypt_token: bool) {
        //
        let hd = conn.hd;
        self.add_conn(hd, conn);

        //
        log::info!(
            "[hd={}] on_incomming_conn  packet_type={:?}",
            hd,
            self.packet_type
        );
        conn.set_packet_type(self.packet_type);

        //
        if push_encrypt_token {
            // 发送 EncryptToken
            let encrypt_token_handler = self.encrypt_token_handler.clone();
            (*encrypt_token_handler)(self, conn);
        }
    }

    ///
    pub fn on_net_packet(&mut self, conn: &TcpConn, mut pkt: NetPacketGuard) {
        let hd = conn.hd;
        {
            let peek = pkt.peek();
            log::info!("[hd={}] on_net_packet: 1) ({}){:?}", hd, peek.len(), peek);
        }

        if decode_packet(self.packet_type, hd, &mut pkt, &mut self.hd_encrypt_table) {
            let cmd = pkt.cmd();
            let slice = pkt.consume();
            log::info!("[hd={}] on_net_packet: 2) cmd({})", hd, cmd);

            if let Some(handler) = self.handlers.get(&cmd) {
                let h = handler.clone();
                (h)(self, conn, cmd, slice);
            } else {
                // no-handler(trans), use default handler
                let default_handler = self.default_handler.clone();
                (*default_handler)(self, conn, cmd, slice);
            }
        } else {
            //
            let peek = pkt.peek();
            log::error!(
                "[hd={}] on_net_packet failed!!! ({}){:?}!!!",
                hd,
                peek.len(),
                peek
            );
        }
    }

    ///
    pub fn on_hd_lost(&mut self, hd: ConnId) -> Option<Arc<TcpConn>> {
        self.remove_conn(hd)
    }

    ///
    pub fn set_encrypt_key(&mut self, hd: ConnId, key: &[u8]) {
        let key = unsafe { String::from_utf8_unchecked(key.to_vec()) };

        let encrypt_opt = self.hd_encrypt_table.get(&hd);
        if let Some(encrypt) = encrypt_opt {
            let old_encrypt = encrypt;
            log::error!(
                "set [hd={}] encrypt key error!!! already exists {}!!!",
                hd,
                Base64::encode(&old_encrypt.encrypt_key)
            );
            self.hd_encrypt_table.remove(&hd);
        }

        //
        log::info!(
            "set [hd={}] encrypt key {} len {}",
            hd,
            Base64::encode(&key),
            key.len()
        );
        self.hd_encrypt_table.insert(
            hd,
            EncryptData {
                no_list: LinkedList::new(),
                encrypt_key: key,
            },
        );
    }

    ///
    #[inline(always)]
    pub fn packet_type(&self) -> PacketType {
        self.packet_type
    }

    ///
    pub fn set_encrypt_token_handler<F>(&mut self, f: F)
    where
        F: Fn(&mut NetProxy, &TcpConn) + 'static,
    {
        self.encrypt_token_handler = Rc::new(Box::new(f));
    }

    /// cmd handler
    pub fn set_packet_handler<F>(&mut self, cmd: CmdId, f: F)
    where
        F: Fn(&mut NetProxy, &TcpConn, CmdId, &[u8]) + 'static,
    {
        self.handlers.insert(cmd, Rc::new(Box::new(f)));
    }

    /// 发送接口线程安全
    #[inline(always)]
    pub fn send_raw(&mut self, conn: &TcpConn, cmd: CmdId, slice: &[u8]) {
        let mut pkt = take_packet(slice.len());
        pkt.set_leading_field_size(self.leading_field_size);
        pkt.set_cmd(cmd);
        assert_eq!(pkt.body_size, 0);
        pkt.append_slice(slice);

        //
        self.send_packet(conn, pkt);
    }

    ///
    #[inline(always)]
    pub fn send_proto<M>(&mut self, conn: &TcpConn, cmd: CmdId, msg: &M)
    where
        M: prost::Message,
    {
        //
        let len = msg.encoded_len();
        let mut pkt = take_packet(len);
        pkt.set_leading_field_size(self.leading_field_size);
        pkt.set_cmd(cmd);

        // set msg
        let pb_slice = pkt.as_write_mut();
        write_prost_message(msg, pb_slice);
        pkt.end_write(len);

        //
        self.send_packet(conn, pkt);
    }

    ///
    #[inline(always)]
    pub fn send_packet(&mut self, conn: &TcpConn, mut pkt: NetPacketGuard) {
        let hd = conn.hd;
        let cmd = pkt.cmd();

        //
        if encode_packet(conn.packet_type(), hd, &mut pkt, &mut self.hd_encrypt_table) {
            let slice = pkt.peek();
            log::info!(
                "[hd={}] send packet cmd({}) -- ({}){:?}",
                hd,
                cmd,
                slice.len(),
                slice
            );
            conn.send_buffer(pkt);
        } else {
            //
            let peek = pkt.peek();
            log::error!(
                "[hd={}] send packet failed!!! cmd({})!!! ({}){:?}!!!",
                hd,
                cmd,
                peek.len(),
                peek
            );
        }
    }

    ///
    #[inline(always)]
    pub fn add_conn(&mut self, hd: ConnId, conn: &Arc<TcpConn>) {
        self.conn_table.insert(hd, conn.clone());
    }

    ///
    #[inline(always)]
    pub fn remove_conn(&mut self, hd: ConnId) -> Option<Arc<TcpConn>> {
        self.conn_table.remove(&hd)
    }

    ///
    #[inline(always)]
    pub fn get_conn(&self, hd: ConnId) -> Arc<TcpConn> {
        match self.conn_table.get(&hd) {
            Some(conn) => conn.clone(),
            None => std::unreachable!(),
        }
    }
}

///
//#[inline(always)]
pub fn write_prost_message<M>(msg: &M, mut buf: &mut [u8]) -> bool
where
    M: prost::Message,
{
    match msg.encode(&mut buf) {
        Ok(()) => true,
        Err(err) => {
            log::error!("encode msg error: {}!!! {:?},", err, msg);
            false
        }
    }
}
