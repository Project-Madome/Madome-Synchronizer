// use std::time;

use rdkafka::error::KafkaError;
use rdkafka::message::{OwnedHeaders, ToBytes};
use rdkafka::producer::{BaseProducer, BaseRecord};
use rdkafka::ClientConfig;

pub fn create(servers: Vec<&str>) -> Result<BaseProducer, KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", &servers[..].join(","))
        .set("acks", "all")
        .create()
}

pub fn send<K, P>(
    producer: &BaseProducer,
    topic: &str,
    key: &K,
    payload: &P,
    // headers: Vec<(&str, &HV)>,
) -> Result<(), KafkaError>
where
    K: ToBytes + ?Sized,
    P: ToBytes + ?Sized,
{
    // let mut headers_map = OwnedHeaders::new();
    /* headers.into_iter().for_each(|(k, v)| {
        headers_map.add(k, v);
    }); */

    // let timestamp = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;

    let record = BaseRecord::to(topic).key(key).payload(payload);
    // .headers(headers_map)
    // .timestamp();

    let r = producer.send(record);

    if let Err((err, _)) = r {
        return Err(err);
    }

    Ok(())
}
