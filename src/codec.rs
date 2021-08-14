use std::marker::PhantomData;

use asynchronous_codec::{Bytes, Decoder, Encoder, LengthCodec};
use bincode::{DefaultOptions, Error as BincodeError, Options};
use bytes::BytesMut;
use serde::{Deserialize, Serialize};

/// Bincode codec for asynchronous serialization.
pub struct BincodeCodec<Enc, Dec> {
    options: DefaultOptions,
    inner: LengthCodec,
    enc: PhantomData<Enc>,
    dec: PhantomData<Dec>,
}

impl<Enc, Dec> BincodeCodec<Enc, Dec>
where
    for<'de> Dec: Deserialize<'de> + 'static,
    for<'de> Enc: Serialize + 'static,
{
    /// Creates a new `BincodeCodec` with the associated types
    pub fn new() -> BincodeCodec<Enc, Dec> {
        Self::with_options(bincode::DefaultOptions::new())
    }

    /// Creates a new `BincodeCodec` with the associated types and provided options
    pub fn with_options(options: bincode::DefaultOptions) -> BincodeCodec<Enc, Dec> {
        BincodeCodec {
            options,
            inner: LengthCodec {},
            enc: PhantomData,
            dec: PhantomData,
        }
    }
}

impl<Enc, Dec> Decoder for BincodeCodec<Enc, Dec>
where
    for<'de> Dec: Deserialize<'de> + 'static,
    for<'de> Enc: Serialize + 'static,
{
    type Item = Dec;
    type Error = BincodeError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.inner.decode(buf)? {
            Some(bytes) => Ok(self.options.deserialize(&bytes)?),
            None => Ok(None),
        }
    }
}

/// Encoder impl encodes object streams to bytes
impl<Enc, Dec> Encoder for BincodeCodec<Enc, Dec>
where
    for<'de> Dec: Deserialize<'de> + 'static,
    for<'de> Enc: Serialize + 'static,
{
    type Item = Enc;
    type Error = BincodeError;

    fn encode(&mut self, data: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let size = self.options.serialized_size(&data)?;
        buf.reserve(size as usize);
        let message = self.options.serialize(&data)?;
        self.inner.encode(Bytes::from(message), buf)?;
        Ok(())
    }
}