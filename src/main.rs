use cita_tool::{
    client::{
        basic::{Client, ClientExt},
        remove_0x, TransactionOptions,
    },
    crypto::{Encryption, PrivateKey},
    decode, decode_params, encode, encode_params,
    protos::blockchain::Transaction,
    protos::blockchain::UnverifiedTransaction,
    rpctypes::{JsonRpcResponse, ParamsValue, ResponseValue},
    ProtoMessage, H256,
};
use log::{debug, error, info, trace, warn};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct ReturnString {
    string: String,
}

#[derive(Serialize, Deserialize)]
struct ReturnUint {
    uint256: String,
}

pub fn parse_json_result(res: JsonRpcResponse) -> Option<String> {
    if let Some(res) = res.result() {
        match res {
            ResponseValue::Singe(v) => match v {
                ParamsValue::String(s) => {
                    let s = remove_0x(&s);
                    return Some(s.to_string());
                }
                _ => {}
            },
            _ => {}
        }
    }
    None
}

pub fn parse_json_result_kv(res: JsonRpcResponse, key: &str) -> Option<String> {
    if let Some(res) = res.result() {
        match res {
            ResponseValue::Map(m) => {
                if let Some(hash) = m.get(key) {
                    match hash {
                        ParamsValue::String(hstr) => {
                            return Some(hstr.to_string());
                        }
                        ParamsValue::Null => {
                            return Some(String::new());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    None
}

static CONTRACT_ADDR: &'static str = "0x8d014ad46d3bdf5f08f16b7d3ca6af7dfc3f568d";
static FUNC_HASH: &'static str = "69ed8049";
static CALL_HASH: &'static str = "0x27f37949";
static SK: &'static str = "6ff0a6e8cd3b19cfc17503a8cdf4b7fc5aafe63ab81749a32b7fb5555eb8771f";

fn main() {
    env_logger::init();

    let private_key = PrivateKey::from_str(SK, Encryption::Secp256k1).unwrap();
    let mut cli = Client::new().set_uri(&"http://127.0.0.1:1337");
    cli.set_private_key(&private_key);
    let mut count: usize = 0;

    loop {
        let res = cli.get_block_number();
        info!("get block number {:?}", res);
        if let Ok(res) = res {
            if let Some(hstr) = parse_json_result(res) {
                if let Ok(h) = u64::from_str_radix(&hstr, 16) {
                    if h > 0 {
                        break;
                    }
                }
            }
        }
        std::thread::sleep(Duration::new(1, 0));
    }

    loop {
        let key = count.to_string();
        let code_str = encode_params(
            &[
                "string".to_string(),
                "string".to_string(),
                "uint256".to_string(),
            ],
            &[key.clone(), key.clone(), key.clone()],
            true,
        )
        .unwrap();

        let mut code = FUNC_HASH.to_string();

        code = code + &code_str;

        //info!("get send code {:?}", code);
        let tx_opt = TransactionOptions::new()
            .set_code(&code)
            .set_address(CONTRACT_ADDR);

        let res = cli.send_raw_transaction(tx_opt);
        info!("get sent tx res {:?}", res);
        if res.is_err() {
            return;
        }
        let hash = parse_json_result_kv(res.unwrap(), "hash").unwrap();
        loop {
            let rpt = cli.get_transaction_receipt(&hash);
            //info!("get reciept {:?}", rpt);
            if let Ok(rpt) = rpt {
                if let Some(msg) = parse_json_result_kv(rpt, "errorMessage") {
                    if msg.is_empty() {
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::new(1, 0));
        }

        let code_str = encode_params(&["string".to_string()], &[key.clone()], true).unwrap();
        let code = CALL_HASH.to_string() + &code_str;
        info!("get call code {:?}", code);
        let res = cli.call(None, CONTRACT_ADDR, Some(&code), "pending");
        info!("get clit call res {:?}", res);

        if let Ok(res) = res {
            if let Some(jres) = parse_json_result(res) {
                if let Ok(call) = decode_params(&["string".to_string(), "uint".to_string()], &jres)
                {
                    let p1: ReturnString = serde_json::from_str(&call[0]).unwrap();
                    let p2: ReturnUint = serde_json::from_str(&call[1]).unwrap();

                    if p1.string != key || usize::from_str_radix(&p2.uint256, 16).unwrap() != count
                    {
                        info!("not get right result get-> {:?} expect-> {}", call, key);
                        return;
                    }
                }
            }
        }

        count += 1;
    }
}
