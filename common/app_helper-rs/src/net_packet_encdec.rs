use std::collections::LinkedList;

use net_packet::NetPacketGuard;

use commlib::utils::rand_between_exclusive_i8;
use srv_helper::{ConnId, PacketType};
use srv_helper::{FROM_CLIENT_PKT_LEADING_FIELD_SIZE, PKT_LEADING_FIELD_SIZE_DEFAULT};

/// 4字节包体前导长度字段 + 2字节协议号 :
///     leading(full_len)(4) + cmd(2)
#[allow(dead_code)]
const SERVER_INNER_HEADER_SIZE: usize = PKT_LEADING_FIELD_SIZE_DEFAULT + 2;

/// 4字节包体前导长度字段 + 2字节协议号(发往客户端) :
///     leading( full_len)(4) + cmd(2)
#[allow(dead_code)]
const TO_CLIENT_HEADER_SIZE: usize = PKT_LEADING_FIELD_SIZE_DEFAULT + 2;

/// 2字节包体前导长度字段 + 1字节序号 + 2字节协议号(来自客户端) :
///     leading(full_len)(2) + client_no(1) + cmd(2)
#[allow(dead_code)]
const FROM_CLIENT_HEADER_SIZE: usize = FROM_CLIENT_PKT_LEADING_FIELD_SIZE + 1 + 2;

/// 2字节协议号(WS) :
///     cmd(2)
#[allow(dead_code)]
const TO_CLIENT_HEADER_SIZE_WS: usize = 2;

/// 1字节序号 + 2字节协议号: WS
///     client_no(1) + cmd(2)
#[allow(dead_code)]
const FROM_CLIENT_HEADER_SIZE_WS: usize = 3;

/// 协议号类型，2字节
#[allow(dead_code)]
pub const PKT_CMD_LEN: usize = 2; /* 2字节协议号 */

/// 消息体加密字节数
const ENCRYPT_MAX_BODY_LEN: usize = 4; /* 4字节消息体 */
pub const ENCRYPT_MAX_LEN: usize = PKT_CMD_LEN + ENCRYPT_MAX_BODY_LEN;
pub const ENCRYPT_KEY_LEN: usize = 64; /* 密钥总长度，根据 client no 进行偏移 */

const SAVED_NO_COUNT: usize = 1;

///
pub struct EncryptData {
    pub no_list: LinkedList<i8>, // 缓存的包序号列表
    pub encrypt_key: String,
}

/// Decode header from packet slice
pub fn decode_packet(
    packet_type: PacketType,
    hd: ConnId,
    pkt: &mut NetPacketGuard,
    encrypt_table: &mut hashbrown::HashMap<ConnId, EncryptData>,
) -> bool {
    match packet_type {
        PacketType::Server => decode_packet_server(pkt),
        PacketType::Client => decode_packet_client(hd, pkt, encrypt_table),
        PacketType::Robot => decode_packet_robot(pkt),
        PacketType::ClientWs => decode_packet_client_ws(hd, pkt, encrypt_table),
        PacketType::RobotWs => decode_packet_robot_ws(pkt),
        _ => {
            std::unreachable!()
        }
    }
}

///
#[inline(always)]
pub fn decode_packet_server(pkt: &mut NetPacketGuard) -> bool {
    read_server_packet(pkt);
    true
}

///
#[inline(always)]
pub fn decode_packet_client(
    hd: ConnId,
    pkt: &mut NetPacketGuard,
    encrypt_table: &mut hashbrown::HashMap<ConnId, EncryptData>,
) -> bool {
    // 解密
    let encrypt_opt = encrypt_table.get_mut(&hd);
    if let Some(encrypt) = encrypt_opt {
        if !pkt.check_packet() {
            // TODO: 是不是直接 close 这个连接？？？
            log::error!(
                "[decode_packet_client][hd={}] error: check packet failed!!!",
                hd
            );

            //
            false
        } else {
            //
            read_client_packet(pkt, encrypt.encrypt_key.as_str());

            // TODO: 包序号检查
            let client_no = pkt.client_no();
            if !add_packet_no(encrypt, client_no) {
                log::error!(
                    "[decode_packet_client][hd={}] error: packet no {} already exist!!!",
                    hd,
                    client_no
                );

                //
                false
            } else {
                //
                true
            }
        }
    } else {
        log::error!(
            "[decode_packet_client][hd={}] error: encrypt data not exist!!!",
            hd
        );

        //
        false
    }
}

///
#[inline(always)]
pub fn decode_packet_robot(pkt: &mut NetPacketGuard) -> bool {
    read_robot_packet(pkt);
    true
}

///
#[inline(always)]
pub fn decode_packet_client_ws(
    hd: ConnId,
    pkt: &mut NetPacketGuard,
    encrypt_table: &mut hashbrown::HashMap<ConnId, EncryptData>,
) -> bool {
    // 解密
    let encrypt_opt = encrypt_table.get_mut(&hd);
    if let Some(encrypt) = encrypt_opt {
        if !pkt.check_packet() {
            // TODO: 是不是直接 close 这个连接？？？
            log::error!(
                "[decode_packet_client_ws][hd={}] error: check packet failed!!!",
                hd
            );

            //
            false
        } else {
            //
            read_client_ws_packet(pkt, encrypt.encrypt_key.as_str());

            // TODO: 包序号检查
            let client_no = pkt.client_no();
            if !add_packet_no(encrypt, client_no) {
                log::error!(
                    "[decode_packet_client_ws][hd={}] error: packet no {} already exist!!!",
                    hd,
                    client_no
                );

                //
                false
            } else {
                //
                true
            }
        }
    } else {
        log::error!(
            "[decode_packet_client_ws][hd={}] error: encrypt data not exist!!!",
            hd
        );

        //
        false
    }
}

///
#[inline(always)]
pub fn decode_packet_robot_ws(pkt: &mut NetPacketGuard) -> bool {
    read_robot_ws_packet(pkt);
    true
}

/// Encode header into packet slice
pub fn encode_packet(
    packet_type: PacketType,
    hd: ConnId,
    pkt: &mut NetPacketGuard,
    encrypt_table: &mut hashbrown::HashMap<ConnId, EncryptData>,
) -> bool {
    match packet_type {
        PacketType::Server => encode_packet_server(pkt),
        PacketType::Client => encode_packet_client(pkt),
        PacketType::Robot => encode_packet_robot(hd, pkt, encrypt_table),
        PacketType::ClientWs => encode_packet_client_ws(pkt),
        PacketType::RobotWs => encode_packet_robot_ws(hd, pkt, encrypt_table),
        _ => {
            std::unreachable!()
        }
    }
}

///
#[inline(always)]
pub fn encode_packet_server(pkt: &mut NetPacketGuard) -> bool {
    write_server_packet(pkt);
    true
}

///
#[inline(always)]
pub fn encode_packet_client(pkt: &mut NetPacketGuard) -> bool {
    write_client_packet(pkt);
    true
}

///
#[inline(always)]
pub fn encode_packet_robot(
    hd: ConnId,
    pkt: &mut NetPacketGuard,
    encrypt_table: &mut hashbrown::HashMap<ConnId, EncryptData>,
) -> bool {
    // 加密
    let encrypt_opt = encrypt_table.get_mut(&hd);
    if let Some(encrypt) = encrypt_opt {
        // 随机序号
        let no = rand_packet_no(encrypt, hd);
        pkt.set_client_no(no);
        write_robot_packet(pkt, encrypt.encrypt_key.as_str());

        log::info!("[encode_packet_robot][hd={}] rand_packet_no: {}", hd, no,);

        //
        true
    } else {
        log::error!("[encode_packet_robot][hd={}] encrypt data not exist!!!", hd);

        //
        false
    }
}

///
#[inline(always)]
pub fn encode_packet_client_ws(pkt: &mut NetPacketGuard) -> bool {
    write_client_ws_packet(pkt);
    true
}

///
#[inline(always)]
pub fn encode_packet_robot_ws(
    hd: ConnId,
    pkt: &mut NetPacketGuard,
    encrypt_table: &mut hashbrown::HashMap<ConnId, EncryptData>,
) -> bool {
    // 加密
    let encrypt_opt = encrypt_table.get_mut(&hd);
    if let Some(encrypt) = encrypt_opt {
        // 随机序号
        let no = rand_packet_no(encrypt, hd);
        pkt.set_client_no(no);
        write_robot_ws_packet(pkt, encrypt.encrypt_key.as_str());

        log::info!("[encode_packet_robot_ws][hd={}] rand_packet_no: {}", hd, no,);

        //
        true
    } else {
        log::error!(
            "[encode_packet_robot_ws][hd={}] encrypt data not exist!!!",
            hd
        );

        //
        false
    }
}

#[inline(always)]
fn rand_packet_no(encrypt: &mut EncryptData, _hd: ConnId) -> i8 {
    let no_list = &mut encrypt.no_list;
    let no = rand_between_exclusive_i8(0, (ENCRYPT_KEY_LEN - 1) as i8, no_list);

    no_list.push_back(no);

    if no_list.len() > SAVED_NO_COUNT {
        no_list.pop_front();
    }
    no
}

#[inline(always)]
fn add_packet_no(encrypt: &mut EncryptData, no: i8) -> bool {
    if no >= ENCRYPT_KEY_LEN as i8 {
        return false;
    }

    let no_list = &mut encrypt.no_list;
    for it in &*no_list {
        if *it == no {
            return false;
        }
    }

    no_list.push_back(no);

    if no_list.len() > SAVED_NO_COUNT {
        no_list.pop_front();
    }
    true
}

/* //////////////////////////////////////////////////////////////// */

/// Server inner 写入包头
//#[inline(always)]
fn write_server_packet(pkt: &mut NetPacketGuard) {
    //
    let wrote_body_len = pkt.wrote_body_len();
    assert!(wrote_body_len > 0);

    // 组合最终包 (Notice: Prepend 是反向添加)
    // 2 字节 cmd
    let cmd = pkt.cmd as u16;
    pkt.prepend_u16(cmd);

    // 4 字节包体长度 + 2 字节协议号 = HEADER_SIZE
    let full_len = SERVER_INNER_HEADER_SIZE + wrote_body_len;
    pkt.prepend_u32(full_len as u32);
}

/// Server inner 读取包头
//#[inline(always)]
fn read_server_packet(pkt: &mut NetPacketGuard) {
    // MUST only one packet in buffer

    // 4 字节包体长度
    let full_len = pkt.read_u32() as usize;
    let body_size = full_len - SERVER_INNER_HEADER_SIZE;
    assert!(body_size > 0);

    // 2 字节 cmd
    pkt.cmd = pkt.read_u16();
}

/// Server 写入包头 ( 方向 server -> client )
#[inline(always)]
fn write_client_packet(pkt: &mut NetPacketGuard) {
    //
    let wrote_body_len = pkt.wrote_body_len();
    assert!(wrote_body_len > 0);

    // 组合最终包 (Notice: Prepend 是反向添加)
    // 2 字节 cmd
    let cmd = pkt.cmd as u16;
    pkt.prepend_u16(cmd);

    // 4 字节包体长度 + 2 字节协议号 = HEADER_SIZE
    let full_len = TO_CLIENT_HEADER_SIZE + wrote_body_len;
    pkt.prepend_u32(full_len as u32);
}

/// Server 读取包头 ( 方向 client -> server )
#[inline(always)]
fn read_client_packet(pkt: &mut NetPacketGuard, key: &str) {
    // MUST only one packet in buffer

    // 2 字节包体长度
    let full_len = pkt.read_u16() as usize;
    let body_size = full_len - FROM_CLIENT_HEADER_SIZE;
    assert!(body_size > 0);

    // 1 字节序号
    let no = pkt.read_u8() as i8;
    pkt.set_client_no(no);

    // 解密 ( 从 cmd 位置开始 )
    let read_mut = pkt.as_read_mut();
    assert_eq!(read_mut.len() - std::mem::size_of::<u16>(), body_size);
    decrypt_packet(read_mut, key, no);

    // 2 字节 cmd
    pkt.cmd = pkt.read_u16();
}

/// robot 写入包头
#[inline(always)]
fn write_robot_packet(pkt: &mut NetPacketGuard, key: &str) {
    //
    let wrote_body_len = pkt.wrote_body_len();
    assert!(wrote_body_len > 0);

    // 组合最终包 (Notice: Prepend 是反向添加)
    // 2 字节 cmd
    let cmd = pkt.cmd as u16;
    pkt.prepend_u16(cmd);

    // 加密 ( 从 cmd 位置开始 )
    let no = pkt.client_no();
    let read_mut = pkt.as_read_mut();
    assert_eq!(read_mut.len() - std::mem::size_of::<u16>(), wrote_body_len);
    encrypt_packet(read_mut, key, no);

    // 1 字节序号
    pkt.prepend_u8(no as u8);

    // 2 字节包体长度 + 1 字节序号 + 2 字节协议号 = HEADER_SIZE
    let full_len = FROM_CLIENT_HEADER_SIZE + wrote_body_len;
    pkt.prepend_u16(full_len as u16);
}

/// robot 读取包头
#[inline(always)]
fn read_robot_packet(pkt: &mut NetPacketGuard) {
    // MUST only one packet in buffer

    // 4 字节包体长度
    let full_len = pkt.read_u32() as usize;
    let body_size = full_len - TO_CLIENT_HEADER_SIZE;
    assert!(body_size > 0);

    // 2 字节 cmd
    pkt.cmd = pkt.read_u16();
}

/// WS Server 写入包头 ( 方向 ws_server -> ws_client )
#[inline(always)]
fn write_client_ws_packet(pkt: &mut NetPacketGuard) {
    //
    let wrote_body_len = pkt.wrote_body_len();
    assert!(wrote_body_len > 0);

    // 组合最终包 (Notice: Prepend 是反向添加)
    // 2 字节 cmd
    let cmd = pkt.cmd as u16;
    pkt.prepend_u16(cmd);
}

/// WS Server 读取包头 ( 方向 ws_client -> ws_server )
#[inline(always)]
fn read_client_ws_packet(pkt: &mut NetPacketGuard, key: &str) {
    // MUST only one packet in buffer

    //
    let body_size = pkt.buffer_raw_len() - FROM_CLIENT_HEADER_SIZE_WS;
    assert!(body_size > 0);

    // 1 字节序号
    let no = pkt.read_u8() as i8;
    pkt.set_client_no(no);

    // 解密 ( 从 cmd 位置开始 )
    let read_mut = pkt.as_read_mut();
    assert_eq!(read_mut.len() - std::mem::size_of::<u16>(), body_size);
    decrypt_packet(read_mut, key, no);

    // 2 字节 cmd
    pkt.cmd = pkt.read_u16();
}

/// WS Client 写入包头 ( 方向 ws_client -> ws_server )
#[inline(always)]
fn write_robot_ws_packet(pkt: &mut NetPacketGuard, key: &str) {
    //
    let wrote_body_len = pkt.wrote_body_len();
    assert!(wrote_body_len > 0);

    // 组合最终包 (Notice: Prepend 是反向添加)
    // 2 字节 cmd
    let cmd = pkt.cmd as u16;
    pkt.prepend_u16(cmd);

    // 加密 ( 从 cmd 位置开始 )
    let no = pkt.client_no();
    let read_mut = pkt.as_read_mut();
    assert_eq!(read_mut.len() - std::mem::size_of::<u16>(), wrote_body_len);
    encrypt_packet(read_mut, key, no);

    // 1 字节序号
    pkt.prepend_u8(no as u8);
}

/// WS Client 读取包头 ( 方向 ws_server -> ws_client ）
#[inline(always)]
fn read_robot_ws_packet(pkt: &mut NetPacketGuard) {
    // MUST only one packet in buffer

    //
    let body_size = pkt.buffer_raw_len() - TO_CLIENT_HEADER_SIZE_WS;
    assert!(body_size > 0);

    // 2 字节 cmd
    pkt.cmd = pkt.read_u16();
}

#[inline(always)]
fn encrypt_packet(body: &mut [u8], key: &str, no: i8) {
    let len = body.len();
    let slice_len = if len < ENCRYPT_MAX_BODY_LEN {
        PKT_CMD_LEN + len
    } else {
        ENCRYPT_MAX_LEN
    };

    let key_len = ENCRYPT_KEY_LEN - no as usize;

    let encrypt_len = std::cmp::min(slice_len, key_len);
    let ptr = body.as_mut_ptr();
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, encrypt_len) };
    for i in 0..encrypt_len {
        let from = no as usize + i;
        slice[i] ^= (key.as_bytes())[from];
    }
}

#[inline(always)]
fn decrypt_packet(body: &mut [u8], key: &str, no: i8) {
    // xor decrypt is just same as encrypt
    encrypt_packet(body, key, no);
}
