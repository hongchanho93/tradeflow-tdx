use super::{Market, Standard};
use crate::error::TdxError;
use crate::request::Request;
use crate::types::SecurityCode;
use crate::wire::{ByteReader, ByteWriter};

/// 当日分笔成交。`offset = 0` 从最新一页开始。
#[derive(Clone, Debug)]
pub struct Transactions {
    pub market: Market,
    pub code: SecurityCode,
    pub offset: u16,
    pub count: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transaction {
    /// 北京时间从 00:00 起算的分钟数。TDX 标准行情不提供秒。
    pub minute: u16,
    /// 协议原始整数价格；股票为分、ETF 为厘。
    pub price: i64,
    /// 成交量（手）。
    pub volume: i64,
    /// 该条记录聚合的成交笔数。
    pub transaction_count: i64,
    /// 0 主买、1 主卖、其他值保留为未知方向。
    pub buy_or_sell: u8,
    pub unknown: u8,
}

impl Request for Transactions {
    type Dialect = Standard;
    type Response = Vec<Transaction>;
    const COMMAND: u16 = 0x0fc5;

    fn encode(&self, body: &mut ByteWriter) -> Result<(), TdxError> {
        if self.count == 0 {
            return Err(TdxError::InvalidArgument(
                "transaction request count must be positive".to_string(),
            ));
        }
        body.u8(self.market.code())
            .u8(0)
            .bytes(self.code.as_bytes())
            .u16(self.offset)
            .u16(self.count);
        Ok(())
    }

    fn decode(&self, body: &mut ByteReader<'_>) -> Result<Vec<Transaction>, TdxError> {
        let count = body.u16()?;
        let mut previous_price = 0;
        let mut rows = Vec::with_capacity(usize::from(count));
        for index in 0..count {
            let minute = body.u16()?;
            let encoded_price = body.varint()?;
            let price = if index == 0 {
                encoded_price
            } else {
                previous_price + encoded_price
            };
            let volume = body.varint()?;
            let transaction_count = body.varint()?;
            let buy_or_sell = body.u8()?;
            let unknown = body.u8()?;
            if minute >= 24 * 60 || price <= 0 || volume < 0 || transaction_count < 0 {
                return Err(TdxError::Decode(format!(
                    "invalid transaction row minute={minute} price={price} volume={volume} count={transaction_count}"
                )));
            }
            rows.push(Transaction {
                minute,
                price,
                volume,
                transaction_count,
                buy_or_sell,
                unknown,
            });
            previous_price = price;
        }
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::{Market, Transactions};
    use crate::request::Request;
    use crate::types::SecurityCode;
    use crate::wire::{ByteReader, ByteWriter};

    fn request() -> Transactions {
        Transactions {
            market: Market::Shanghai,
            code: SecurityCode::new("600000").unwrap(),
            offset: 0,
            count: 10,
        }
    }

    #[test]
    fn request_body_matches_live_wire_layout() {
        let mut body = ByteWriter::new();
        request().encode(&mut body).unwrap();
        assert_eq!(
            body.as_slice(),
            &[1, 0, b'6', b'0', b'0', b'0', b'0', b'0', 0, 0, 10, 0]
        );
    }

    #[test]
    fn live_response_fixture_decodes_absolute_then_delta_prices() {
        let bytes = "0a008c02860e100301008c0200050201008c0201280100008c0200010100008c0241b1011001008c02010c0100008c02413b0501008c0201060200008d0241040101008d020017040100"
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let rows = request().decode(&mut ByteReader::new(&bytes)).unwrap();
        assert_eq!(rows.len(), 10);
        assert_eq!(
            (rows[0].minute, rows[0].price, rows[0].volume),
            (652, 902, 16)
        );
        assert_eq!(
            (rows[2].price, rows[2].volume, rows[2].buy_or_sell),
            (903, 40, 0)
        );
        assert_eq!(
            (rows[8].minute, rows[8].price, rows[8].buy_or_sell),
            (653, 902, 1)
        );
        assert!(
            request()
                .decode(&mut ByteReader::new(&bytes[..bytes.len() - 1]))
                .is_err()
        );
    }
}
