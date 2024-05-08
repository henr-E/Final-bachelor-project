use num_bigint::Sign;
use proto::frontend::BigDecimal as ProtoBigDecimal;
use sensor_store::SensorStore as SensorStoreInner;
use sqlx::types::BigDecimal;

mod crud;
mod data_fetching;

#[derive(Clone)]
pub struct SensorStore(SensorStoreInner);

impl SensorStore {
    pub async fn new() -> Self {
        Self(
            SensorStoreInner::new()
                .await
                .expect("Could not create sensor store."),
        )
    }

    pub fn as_inner(&self) -> &SensorStoreInner {
        &self.0
    }
}

impl std::ops::Deref for SensorStore {
    type Target = SensorStoreInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Convert [`sqlx`]'s [`BigDecimal`] type to [`ProtoBigDecimal`] from the protobuf messages.
fn into_proto_big_decimal(decimal: &BigDecimal) -> ProtoBigDecimal {
    let (big_int, exponent) = decimal.as_bigint_and_exponent();
    let (sign, integer) = big_int.to_u32_digits();
    ProtoBigDecimal {
        integer: integer.to_vec(),
        sign: sign == Sign::Minus,
        // Scale/exponent is inverted in the `BigDecimal` type. See
        // [documentation](https://docs.rs/bigdecimal/0.4.3/src/bigdecimal/lib.rs.html#191)
        // for more info.
        exponent: -exponent,
    }
}
