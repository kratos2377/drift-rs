use std::{
    cell::LazyCell,
    time::{SystemTime, UNIX_EPOCH},
};

use anchor_lang::{AnchorDeserialize, AnchorSerialize, Space};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub use crate::types::SignedMsgOrderParamsMessage as SignedOrder;
use crate::{
    constants::MarketExt,
    types::{Context, MarketId, OrderParams, SdkError, SdkResult},
    DriftClient,
};

/// Swift message discriminator (Anchor)
const SWIFT_MSG_PREFIX: LazyCell<[u8; 8]> = LazyCell::new(|| {
    solana_sdk::hash::hash(b"global:SignedMsgOrderParamsMessage").to_bytes()[..8]
        .try_into()
        .unwrap()
});

pub const SWIFT_DEVNET_WS_URL: &str = "wss://master.swift.drift.trade";
pub const SWIFT_MAINNET_WS_URL: &str = "wss://swift.drift.trade";

const LOG_TARGET: &str = "swift";

#[derive(Clone, Deserialize)]
pub struct OrderNotification<'a> {
    #[allow(dead_code)]
    channel: &'a str,
    order: SignedOrderInfo,
}

#[derive(Deserialize)]
pub struct Heartbeat {
    #[serde(deserialize_with = "deser_int_str", rename = "message")]
    ts: u64,
}

/// Swift order and metadata fresh from the Websocket
///
/// This is an off-chain authorization for a taker order.
/// It may be placed and filled by any willing counter-party, ensuring the time-price bounds
/// are respected.
#[derive(Clone, Debug, Deserialize)]
pub struct SignedOrderInfo {
    /// Swift order uuid
    uuid: String,
    /// Order creation timestamp (unix ms)
    pub ts: u64,
    /// The taker authority pubkey
    #[serde(deserialize_with = "deser_pubkey")]
    pub taker_authority: Pubkey,
    /// The authority pubkey that verifies `signature`
    /// it is either the taker authority or a sub-account delegate
    #[serde(rename = "signing_authority", deserialize_with = "deser_pubkey")]
    pub signer: Pubkey,
    /// hex-ified, borsh encoded signed order message
    /// this is the signed/verified payload for onchain use
    #[serde(rename = "order_message", deserialize_with = "deser_order_message")]
    order: SignedOrder,
    /// Signature over the serialized `order` payload
    #[serde(rename = "order_signature", deserialize_with = "deser_signature")]
    pub signature: Signature,
}

impl SignedOrderInfo {
    /// The order's UUID (stringified)
    pub fn order_uuid_str(&self) -> &str {
        self.uuid.as_ref()
    }
    /// The order's UUID (raw)
    pub fn order_uuid(&self) -> [u8; 8] {
        self.order.uuid
    }
    /// The drift order params of the message
    pub fn order_params(&self) -> OrderParams {
        self.order.signed_msg_order_params
    }
    /// The taker sub-account_id of the order
    pub fn taker_subaccount_id(&self) -> u16 {
        self.order.sub_account_id
    }
    /// serialize the order message for onchain use e.g. signature verification
    pub fn encode_for_signing(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(SignedOrder::INIT_SPACE + 8);
        buf.extend_from_slice(SWIFT_MSG_PREFIX.as_slice());
        self.order
            .serialize(&mut buf)
            .expect("swift msg serialized");
        hex::encode(buf).into_bytes()
    }
    /// convert swift order into anchor ix data
    pub fn to_ix_data(&self) -> Vec<u8> {
        let signed_msg = self.encode_for_signing();
        [
            self.signature.as_ref(),
            self.signer.as_ref(),
            &(signed_msg.len() as u16).to_le_bytes(),
            signed_msg.as_ref(),
        ]
        .concat()
    }
    /// True if the message was signed by an identity other than the authority i.e a delegated
    pub fn using_delegate_signing(&self) -> bool {
        self.taker_authority != self.signer
    }
}

/// Emits swift orders from the Ws server
pub type SwiftOrderStream = ReceiverStream<SignedOrderInfo>;

/// Subscribe to the Swift WebSocket server, authenticate, and listen to new orders
///
/// `client` Drift client instance
/// `markets` markets to listen on for new swift orders
///
/// Returns a stream of new Swift order messages
pub async fn subscribe_swift_orders(
    client: &DriftClient,
    markets: &[MarketId],
) -> SdkResult<SwiftOrderStream> {
    let base_url = if client.context == Context::MainNet {
        SWIFT_MAINNET_WS_URL
    } else {
        SWIFT_DEVNET_WS_URL
    };
    let maker_pubkey = client.wallet().authority().to_string();
    let (ws_stream, _) = connect_async(format!("{base_url}/ws?pubkey={maker_pubkey}"))
        .await
        .map_err(|err| {
            log::error!(target: LOG_TARGET, "couldn't connect to server: {err:?}");
            SdkError::WsClient(err)
        })?;

    let (mut outgoing, mut incoming) = ws_stream.split();

    // handle authentication and subscription
    while let Some(msg) = incoming.next().await {
        let msg = msg.map_err(|err| {
            log::error!(target: LOG_TARGET, "failed reading swift msg: {err:?}");
            SdkError::WsClient(err)
        })?;

        if let Message::Text(text) = msg {
            log::debug!(target: LOG_TARGET, "msg: {text}");
            let message: Value = serde_json::from_str(&text).expect("Failed to parse message");

            if let Some(err) = message.get("error") {
                log::error!(target: LOG_TARGET, "swift server error: {err:?}");
                return Err(SdkError::WebsocketError);
            }

            // authenticate with Ws server
            if message["channel"] == "auth" && message.get("nonce").is_some() {
                let nonce = message["nonce"].as_str().expect("got nonce");
                let signature = client
                    .wallet()
                    .sign_message(nonce.as_bytes())
                    .expect("infallible");
                let signature_b64 =
                    base64::engine::general_purpose::STANDARD.encode(signature.as_ref());

                let auth_message = json!({
                    "pubkey": maker_pubkey,
                    "signature": signature_b64,
                })
                .to_string();
                outgoing.send(Message::Text(auth_message.into())).await?;
                continue;
            }

            // subscribe to markets
            if message["channel"] == "auth" && message["message"] == "Authenticated" {
                let subscribe_msgs: Vec<Result<Message, _>> = markets
                    .iter()
                    .map(|m| {
                        assert!(m.is_perp(), "only perp markets");
                        let market = client
                            .program_data()
                            .perp_market_config_by_index(m.index())
                            .expect("market exists");
                        let subscribe_msg = json!({
                          "action": "subscribe",
                          "market_type": "perp",
                          "market_name": market.symbol(),
                        })
                        .to_string();
                        Ok(Message::Text(subscribe_msg.into()))
                    })
                    .collect();

                outgoing
                    .send_all(&mut futures_util::stream::iter(subscribe_msgs))
                    .await?;
                break;
            }
        }
    }

    let (tx, rx) = tokio::sync::mpsc::channel(256);

    // handle swift orders
    tokio::spawn(async move {
        while let Some(msg) = incoming.next().await {
            match msg {
                Ok(Message::Text(ref text)) => {
                    match serde_json::from_str::<OrderNotification>(text) {
                        Ok(OrderNotification { channel: _, order }) => {
                            log::debug!(target: LOG_TARGET, "uuid: {}, latency: {}ms", order.uuid, unix_now_ms().saturating_sub(order.ts));

                            if let Err(err) = tx.try_send(order) {
                                log::error!(target: LOG_TARGET, "order chan failed: {err:?}");
                                break;
                            }
                        }
                        Err(err) => {
                            if text.contains("heartbeat") {
                                if let Ok(heartbeat) = serde_json::from_str::<Heartbeat>(text) {
                                    log::debug!(
                                        target: LOG_TARGET,
                                        "heartbeat latency: {}",
                                        unix_now_ms().saturating_sub(heartbeat.ts)
                                    );
                                    continue;
                                }
                            }
                            log::error!(target: LOG_TARGET, "{text}. invalid json: {err:?}");
                            break;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    log::error!(target: LOG_TARGET, "server closed connection");
                    break;
                }
                Ok(_) => continue,
                Err(err) => {
                    log::error!(target: LOG_TARGET, "failed reading swift msg: {err:?}");
                    break;
                }
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn deser_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
    Ok(s.parse().expect("base58 pubkey"))
}

fn deser_signature<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
    Ok(Signature::try_from(base64::engine::general_purpose::STANDARD.decode(s).unwrap()).unwrap())
}

fn deser_order_message<'de, D>(deserializer: D) -> Result<SignedOrder, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let order_message: &str = serde::de::Deserialize::deserialize(deserializer)?;
    let order_message_buf = hex::decode(order_message).expect("valid hex");
    Ok(AnchorDeserialize::deserialize(&mut &order_message_buf[8..])
        .expect("SignedMsgOrderParams deser"))
}

fn deser_int_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
    Ok(s.parse().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MarketType;

    #[test]
    fn test_swift_order_deser() {
        let msg = r#"{
            "channel":"signed_orders_perp_1",
            "order":{
                "market_index":1,
                "market_type":"perp",
                "order_message":"b9c165ffdf70594d0001010080841e00000000000000000000000000010000000000000000013201a4e99abc16000000011ab2f982160000000300900f84150000000072753959424c52740000",
                "order_signature":"FIgxWlW+C0abvtE8esSko7At1YGM8h66T0u5lJpwXirW63CuvEllVWZ68NNVFsaqcj4jqgQInXUnLPjIf/PQDA==",
                "signing_authority":"4rmhwytmKH1XsgGAUyUUH7U64HS5FtT6gM8HGKAfwcFE",
                "taker_authority":"DxoRJ4f5XRMvXU9SGuM4ZziBFUxbhB3ubur5sVZEvue2",
                "ts":1739518796400,
                "uuid":"ru9YBLRt"
            }
        }"#;
        let order_notification: OrderNotification = serde_json::from_str(&msg).unwrap();
        let signed_message = order_notification.order;
        assert_eq!(
            signed_message.signer,
            "4rmhwytmKH1XsgGAUyUUH7U64HS5FtT6gM8HGKAfwcFE"
                .parse()
                .unwrap()
        );
        assert_eq!(
            signed_message.taker_authority,
            "DxoRJ4f5XRMvXU9SGuM4ZziBFUxbhB3ubur5sVZEvue2"
                .parse()
                .unwrap()
        );
        assert_eq!(signed_message.ts, 1739518796400);
        assert_eq!(signed_message.uuid, "ru9YBLRt");
        assert_eq!(signed_message.order_params().market_index, 1);
        assert_eq!(signed_message.order_params().market_type, MarketType::Perp);
    }

    #[test]
    fn test_swift_order_encode_for_signing() {
        let msg = "{\"channel\":\"swift_orders_perp_2\",\"order\":{\"market_index\":2,\"market_type\":\"perp\",\"order_message\":\"c8d5a65e2234f55d0001010080841e0000000000000000000000000002000000000000000001320124c6aa950000000001786b2f94000000000000bb64a9150000000074735730364f6d380000\",\"order_signature\":\"SaOaLJ1i0MqZ2cXdp00jGe2EJFa32eOfiQynFU7mclhT86yhIa4/tWXq7r6l7QPN0Jl6frfsZl0nNOvKZxZpAA==\",\"signing_authority\":\"4rmhwytmKH1XsgGAUyUUH7U64HS5FtT6gM8HGKAfwcFE\",\"taker_authority\":\"4rmhwytmKH1XsgGAUyUUH7U64HS5FtT6gM8HGKAfwcFE\",\"ts\":1740456840770,\"uuid\":\"tsW06Om8\"}}";
        let order_notification: OrderNotification = serde_json::from_str(&msg).unwrap();
        let signed_message = order_notification.order;
        assert_eq!(
            signed_message.encode_for_signing().as_slice(),
            b"c8d5a65e2234f55d0001010080841e0000000000000000000000000002000000000000000001320124c6aa950000000001786b2f94000000000000bb64a9150000000074735730364f6d380000"
        );
    }
}
