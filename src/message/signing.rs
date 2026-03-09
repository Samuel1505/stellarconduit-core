use crate::message::errors::SignError;
use crate::message::types::TransactionEnvelope;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

fn generate_payload_hash(envelope: &TransactionEnvelope) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(envelope.origin_pubkey);
    hasher.update(envelope.timestamp.to_be_bytes());
    hasher.update(envelope.tx_xdr.as_bytes());
    hasher.finalize().into()
}

pub fn sign_envelope(
    keypair: &SigningKey,
    envelope: &mut TransactionEnvelope,
) -> Result<(), SignError> {
    let hash = generate_payload_hash(envelope);
    let signature = keypair.sign(&hash);
    envelope.signature = signature.to_bytes();
    Ok(())
}

pub fn verify_signature(envelope: &TransactionEnvelope) -> Result<bool, SignError> {
    let verifying_key = VerifyingKey::from_bytes(&envelope.origin_pubkey)
        .map_err(|e| SignError::MalformedPublicKey(e.to_string()))?;

    let hash = generate_payload_hash(envelope);

    let signature = Signature::from_bytes(&envelope.signature);

    match verifying_key.verify(&hash, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Err(SignError::InvalidSignature),
    }
}
