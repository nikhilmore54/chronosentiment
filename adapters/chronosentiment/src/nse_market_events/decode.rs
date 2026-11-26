//! Kite WebSocket full-mode packet decode (184-byte quote body).
//!
//! Layout must match `scripts/rp608_kite_ticker_cache.decode_full_mode_packet`.

use super::event::{DepthLevel, MarketEvent, SourceProvenance};

pub const FULL_MODE_PACKET_LEN: usize = 184;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    TooShort { got: usize, need: usize },
    BadFrame,
}

fn read_u32_be(buf: &[u8], off: usize) -> u32 {
    u32::from_be_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

fn read_u16_be(buf: &[u8], off: usize) -> u16 {
    u16::from_be_bytes([buf[off], buf[off + 1]])
}

fn depth_level(buf: &[u8], off: usize) -> DepthLevel {
    let qty = read_u32_be(buf, off) as f64;
    let price = read_u32_be(buf, off + 4) as f64 / 100.0;
    DepthLevel {
        price,
        quantity: qty,
    }
}

/// Decode one 184-byte full-mode body into a MarketEvent (ingest_seq/fp filled later).
pub fn decode_full_mode_packet(
    pkt: &[u8],
    receipt_utc_ms: u64,
    source: SourceProvenance,
) -> Result<MarketEvent, DecodeError> {
    if pkt.len() < FULL_MODE_PACKET_LEN {
        return Err(DecodeError::TooShort {
            got: pkt.len(),
            need: FULL_MODE_PACKET_LEN,
        });
    }
    let instrument_token = read_u32_be(pkt, 0);
    let last_price = read_u32_be(pkt, 4) as f64 / 100.0;
    let volume = read_u32_be(pkt, 16) as f64;
    let exchange_timestamp_sec = read_u32_be(pkt, 60);

    let mut depth_buy = [DepthLevel {
        price: 0.0,
        quantity: 0.0,
    }; 5];
    for i in 0..5 {
        depth_buy[i] = depth_level(pkt, 64 + i * 12);
    }
    let mut depth_sell = [DepthLevel {
        price: 0.0,
        quantity: 0.0,
    }; 5];
    for i in 0..5 {
        depth_sell[i] = depth_level(pkt, 124 + i * 12);
    }

    let best_bid = depth_buy[0].price;
    let best_ask = depth_sell[0].price;
    let bid_qty = depth_buy[0].quantity;
    let ask_qty = depth_sell[0].quantity;
    let exchange_timestamp_utc_ms =
        MarketEvent::exchange_timestamp_utc_ms_from_sec(exchange_timestamp_sec);
    let quote_age_ms = if receipt_utc_ms >= exchange_timestamp_utc_ms {
        (receipt_utc_ms - exchange_timestamp_utc_ms) as f64
    } else {
        0.0
    };

    Ok(MarketEvent {
        instrument_token,
        last_price,
        volume,
        exchange_timestamp_sec,
        exchange_timestamp_utc_ms,
        receipt_utc_ms,
        quote_age_ms,
        best_bid,
        best_ask,
        bid_qty,
        ask_qty,
        depth_buy,
        depth_sell,
        source,
        ingest_seq: 0,
        content_fp: 0,
    })
}

/// Decode a Kite binary WS frame: `u16 n`, then n × (`u16 len` + payload).
/// Only 184-byte full-mode payloads are returned.
pub fn decode_full_mode_frame(
    frame: &[u8],
    receipt_utc_ms: u64,
    source: SourceProvenance,
) -> Result<Vec<MarketEvent>, DecodeError> {
    if frame.len() < 2 {
        return Err(DecodeError::BadFrame);
    }
    let num_pkts = read_u16_be(frame, 0) as usize;
    let mut offset = 2usize;
    let mut out = Vec::new();
    for _ in 0..num_pkts {
        if offset + 2 > frame.len() {
            break;
        }
        let pkt_len = read_u16_be(frame, offset) as usize;
        offset += 2;
        if offset + pkt_len > frame.len() {
            break;
        }
        let pkt = &frame[offset..offset + pkt_len];
        offset += pkt_len;
        if pkt_len != FULL_MODE_PACKET_LEN {
            continue;
        }
        out.push(decode_full_mode_packet(pkt, receipt_utc_ms, source)?);
    }
    Ok(out)
}

/// FNV-1a 64-bit fingerprint over decoded economic fields (dedupe).
pub fn content_fingerprint(ev: &MarketEvent) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut mix = |x: u64| {
        h ^= x;
        h = h.wrapping_mul(0x100000001b3);
    };
    mix(u64::from(ev.instrument_token));
    mix(u64::from(ev.exchange_timestamp_sec));
    mix(ev.last_price.to_bits());
    mix(ev.volume.to_bits());
    for lvl in ev.depth_buy.iter().chain(ev.depth_sell.iter()) {
        mix(lvl.price.to_bits());
        mix(lvl.quantity.to_bits());
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_packet() {
        let err = decode_full_mode_packet(&[0u8; 10], 0, SourceProvenance::FixtureReplay);
        assert!(matches!(err, Err(DecodeError::TooShort { .. })));
    }
}
